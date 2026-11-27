// Golden vectors for slice S4: the goal and position reservations of the
// pathfinder, snapping, the adjust* destination searches, tightenPath,
// updateLayer and moveAllies, all in the unmodified reference library.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s4_reservations.cpp -o gen_s4
//   ./gen_s4 docs/golden_vectors/S4_Reservations.json
//
// The terrain is synthetic and the AI module is a subclass that only records
// aiMoveAwayFromUnit. The map is built by the real newMap. Each row is an input
// to replay or a question with the reference's answer after " = "; "d" rows are
// the cells and units an operation changed. isGoalPosWithinAttackRange answers
// are a tape, checkPathCost (slice S5) is a table over every candidate cell.
//
// Not covered: bridges (the wall layer stands for a layer above the ground),
// the retail-crc layout of tightenPath.
//
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a TerrainLogic.cpp.o __ZN12TerrainLogic22getLayerForDestinationEPK7Coord3D
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a Weapon.cpp.o __ZNK6Weapon26isGoalPosWithinAttackRangeEPK6ObjectPK7Coord3DS2_S5_
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZNK17AIUpdateInterface8isMovingEv

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
#include "GameLogic/Weapon.h"

#include <cmath>
#include <cstdio>
#include <cstring>
#include <map>
#include <string>
#include <vector>

#define S4_TAG(Tag, Type, Class, Member)                   \
    struct Tag { using type = Type Class::*; };            \
    template struct MemberAccess<Tag, &Class::Member>;     \
    inline Type Class::* access_member(Tag);
#define S4_FN(Tag, Signature, Member)                      \
    struct Tag { using type = Signature; };                \
    template struct MemberAccess<Tag, &Member>;            \
    inline Tag::type access_member(Tag);

S4_TAG(LsSurfaces, LocomotorSurfaceTypeMask, LocomotorSet, m_validLocomotorSurfaces)
S4_TAG(ObjLayer, PathfindLayerEnum, Object, m_layer)
S4_TAG(PlayerKind, PlayerType, Player, m_playerType)
S4_FN(PfTighten, void (Pathfinder::*)(Object*, const LocomotorSet&, Coord3D*, const Coord3D*), Pathfinder::tightenPath)
S4_FN(PfCost, Int (Pathfinder::*)(Object*, const LocomotorSet&, const Coord3D*, const Coord3D*), Pathfinder::checkPathCost)
S4_FN(PfAllies, Bool (Pathfinder::*)(Object*, Path*), Pathfinder::moveAllies)
S4_FN(PfAway, void (Pathfinder::*)(Object*, const Coord3D&), Pathfinder::moveAlliesAwayFromDestination)
S4_FN(PfRadius, void (Pathfinder::*)(const Object*, Int&, Bool&), Pathfinder::getRadiusAndCenter)
S4_FN(PfAdjustCoord, void (Pathfinder::*)(Int, Int, Bool, Coord3D&, PathfindLayerEnum), Pathfinder::adjustCoordToCell)

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

class S4Terrain : public oracle_wp51::OracleTerrainLogic {
public:
    int w = 1, h = 1;
    std::string cliff, water, up, bridge_end;
    Region3D logical;
    size_t at(int i, int j) const {
        i = i < 0 ? 0 : (i > w + 1 ? w + 1 : i);
        j = j < 0 ? 0 : (j > h + 1 ? h + 1 : j);
        return static_cast<size_t>(j * (w + 2) + i);
    }
    static int cell(float v) { return static_cast<int>(std::floor(v / 10.0f)); }
    size_t at_pos(float x, float y) const { return at(cell(x), cell(y)); }
    void getExtent(Region3D* out) const override { *out = logical; }
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return x * 0.01f + y * 0.003f;
    }
    Real getLayerHeight(Real x, Real y, PathfindLayerEnum layer, Coord3D* normal = nullptr,
                        Bool = true) const override {
        return getGroundHeight(x, y, normal) + 3.0f * static_cast<float>(layer);
    }
    Bool isCliffCell(Real x, Real y) const override { return cliff[at_pos(x, y)] == '1'; }
    Bool isUnderwater(Real x, Real y, Real* waterZ = nullptr, Real* terrainZ = nullptr) override {
        if (waterZ) *waterZ = 0.0f;
        if (terrainZ) *terrainZ = 0.0f;
        return water[at_pos(x, y)] == '1';
    }
    // Answered from the object's position alone, so that a replay needs no tape.
    Bool objectInteractsWithBridgeEnd(Object* o, Int) const override {
        return bridge_end[at_pos(o->getPosition()->x, o->getPosition()->y)] == '1';
    }
    Bool objectInteractsWithBridgeLayer(Object* o, Int, Bool = true) const override {
        return bridge_end[at_pos(o->getPosition()->x, o->getPosition()->y)] == '1';
    }
};

static S4Terrain* g_terrain = nullptr;
static oracle_wp51::World* g_world = nullptr;
static Pathfinder* pf() { return TheAI->pathfinder(); }

// `getLayerForDestination` answers the wall where the table says so.
PathfindLayerEnum TerrainLogic::getLayerForDestination(const Coord3D* pos) {
    return g_terrain->up[g_terrain->at_pos(pos->x, pos->y)] == '1' ? LAYER_WALL : LAYER_GROUND;
}

static void blobs(std::string& table, int count) {
    S4Terrain& t = *g_terrain;
    for (int n = 0; n < count; ++n) {
        const int x = below(t.w + 2), y = below(t.h + 2), rx = 1 + below(5), ry = 1 + below(5);
        for (int j = y; j < y + ry; ++j)
            for (int i = x; i < x + rx; ++i) table[t.at(i, j)] = '1';
    }
}

// ---- weapon tape ------------------------------------------------------------------

static int g_range_percent = 0;
Bool Weapon::isGoalPosWithinAttackRange(const Object*, const Coord3D* goalPos, const Object*,
                                        const Coord3D*) const {
    const bool in = chance(g_range_percent);
    row("range " + c3(*goalPos) + " " + num(in));
    return in;
}

// ---- AI -------------------------------------------------------------------------

struct Ally {
    bool moving = false, attacking = false, busy = false;
    bool move_when_told = false;  // starts moving once it is ordered away
    Path* path = nullptr;         // asks its own allies to move once ordered away
};
static std::map<unsigned, Ally> g_ally;

class S4AI : public AIUpdateInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(S4AI, "OracleReservationAI", 64, 64)
public:
    S4AI(Thing* thing, const ModuleData* data) : AIUpdateInterface(thing, data) {}
    Bool isAttacking() const override { return g_ally[getObject()->getID()].attacking; }
    Bool isBusy() const override { return g_ally[getObject()->getID()].busy; }
    void aiDoCommand(const AICommandParms* p) override;
};
S4AI::~S4AI() {}
static Module* make_s4_ai(Thing* t, const ModuleData* d) { return newInstance(S4AI)(t, d); }

Bool AIUpdateInterface::isMoving() const { return g_ally[getObject()->getID()].moving; }

void S4AI::aiDoCommand(const AICommandParms* p) {
    if (p->m_cmd != AICMD_MOVE_AWAY_FROM_UNIT) return;
    const unsigned id = getObject()->getID();
    row("away " + num(id) + " " + num(p->m_obj->getID()));
    Ally& a = g_ally[id];
    if (a.move_when_told) a.moving = true;
    if (a.path) {
        const bool ran = (pf()->*access_member(PfAllies()))(getObject(), a.path);
        row("nested " + num(id) + " = " + num(ran));
    }
}

struct ExposedFactory : public ModuleFactory { using ModuleFactory::addModuleInternal; };

// ---- templates and objects ------------------------------------------------------

enum { F_STRUCTURE = 64, F_SEETHROUGH = 128, F_MOBILE = 256, F_WALKWALL = 32, F_DEFWALL = 8 };

static std::string g_ini;
static std::vector<std::string> g_blocks, g_units;
static const char* kSurfaces[] = {"GROUND", "WATER", "CLIFF", "RUBBLE", "AIR", "GROUND WATER",
                                  "GROUND CLIFF", "GROUND RUBBLE", "GROUND CLIFF RUBBLE WATER",
                                  "AIR", "AIR"};
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
    for (int i = 0; i < 11; ++i) {
        const char* look = i == 4 ? "WINGS" : i == 9 ? "HOVER" : i == 10 ? "THRUST" : "TWO_LEGS";
        g_ini += "Locomotor S4Loco" + num(i) + "\n  Surfaces = " + kSurfaces[i] + "\n  Appearance = " +
                 look + "\n  Speed = 30\nEnd\n";
    }
    static const float radii[] = {2.0f, 5.0f, 7.0f, 9.0f, 10.0f, 12.0f, 15.0f, 19.0f, 22.0f, 40.0f};
    for (int i = 0; i < 30; ++i) {
        const std::string name = "S4Unit" + num(i);
        const int crusher = i % 5 == 4 ? 3 : i % 3 == 2 ? 1 : 0;
        std::string kinds = i % 3 == 0 ? "INFANTRY" : "VEHICLE";
        if (i % 11 == 4) kinds += " DOZER";
        if (i % 11 == 7) kinds += " HARVESTER";
        if (i == 13) kinds += " IMMOBILE";
        if (i == 14) kinds += " PRODUCED_AT_HELIPAD";
        if (i == 15) kinds += " PROJECTILE";
        // 24..29 fly: hover, wings and thrust.
        const bool flies = i >= 24;
        if (flies) kinds = i == 25 ? "VEHICLE AIRCRAFT PRODUCED_AT_HELIPAD" : "VEHICLE AIRCRAFT";
        const int loco = !flies ? 0 : (i % 3 == 0 ? 9 : i % 3 == 1 ? 4 : 10);
        std::string extra = "  CrusherLevel = " + num(crusher) + "\n  CrushableLevel = " + num(i % 4) + "\n";
        if (i % 13 != 12) {
            extra += "  Behavior = S4AI ModuleTag_AI\n  End\n  Locomotor = SET_NORMAL S4Loco" +
                     num(i == 14 ? 4 : loco) + "\n";
        }
        object_ini(name, kinds, "CYLINDER", radii[(i * 3) % 10], radii[(i * 3) % 10], extra);
        g_units.push_back(name);
    }
    static const char* geoms[] = {"BOX", "CYLINDER", "SPHERE"};
    for (int i = 0; i < 12; ++i) {
        const std::string name = "S4Block" + num(i);
        object_ini(name, "STRUCTURE IMMOBILE", geoms[i % 3], between(4.0f, 28.0f), between(4.0f, 28.0f), "");
        g_blocks.push_back(name);
    }
    for (int i = 0; i < 3; ++i) {
        const std::string name = "S4Fence" + num(i);
        char extra[64];
        std::snprintf(extra, sizeof(extra), "  FenceWidth = %.4f\n", between(15.0f, 70.0f));
        object_ini(name, "STRUCTURE IMMOBILE", "BOX", 20, 3, extra);
        g_blocks.push_back(name);
    }
    object_ini("S4Wall", "STRUCTURE IMMOBILE WALK_ON_TOP_OF_WALL", "BOX", 30, 18, "");
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
    int w = 0, h = 0;
    float maxx = 0, maxy = 0;
    std::vector<Object*> blocks, units, walls, everything;
};

static Object* pick(const std::vector<Object*>& v) { return v[below(static_cast<int>(v.size()))]; }

static Coord3D point(const Scene& s, int off_percent = 8) {
    Coord3D p;
    const float out = chance(off_percent) ? 40.0f : 0.0f;
    p.x = coord(-out, s.maxx + out);
    p.y = coord(-out, s.maxy + out);
    p.z = chance(70) ? 0.0f : between(0.0f, 20.0f);
    return p;
}

// A point, often near a unit so that footprints meet.
static Coord3D busy_point(const Scene& s) {
    if (chance(45)) return point(s);
    Coord3D p = *pick(s.units)->getPosition();
    p.x += between(-18.0f, 18.0f);
    p.y += between(-18.0f, 18.0f);
    p.z = 0.0f;
    return p;
}

static PathfindLayerEnum some_layer(const Scene& s) {
    return !s.walls.empty() && chance(25) ? LAYER_WALL : LAYER_GROUND;
}

static void unit_rows(const Scene& s) {
    for (Object* o : s.units) {
        AIUpdateInterface* ai = o->getAIUpdateInterface();
        Player* player = o->getControllingPlayer();
        const bool human = !(player && player->getPlayerType() == PLAYER_COMPUTER);
        const bool heli = o->isKindOf(KINDOF_PRODUCED_AT_HELIPAD) && o->isDisabledByType(DISABLED_UNMANNED);
        row("unit " + num(o->getID()) + " " + num(ai != nullptr) + " " +
            num(ai ? ai->getIgnoredObstacleID() : 0) + " " +
            num(ai ? ai->isAircraftThatAdjustsDestination() : 0) + " " + num(o->getCrusherLevel()) + " " +
            num(o->getCrushableLevel()) + " " + f32(o->getGeometryInfo().getBoundingCircleRadius()) + " " +
            num(o->getLayer()) + " " + num(o->isKindOf(KINDOF_INFANTRY) != 0) + " " +
            num(o->isKindOf(KINDOF_IMMOBILE) != 0) + " " + num(ai ? ai->isDoingGroundMovement() : 0) + " " +
            num(heli) + " " +
            num(o->isKindOf(KINDOF_DOZER) || o->isKindOf(KINDOF_HARVESTER)) + " " +
            num(o->isKindOf(KINDOF_AIRCRAFT) != 0) + " " + num(o->isKindOf(KINDOF_PROJECTILE) != 0) + " " +
            num(human) + " " + c3(*o->getPosition()));
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

// ---- state diff -----------------------------------------------------------------

// Everything the operations may write, one entry per cell of each layer.
static std::map<std::string, std::string> g_seen;

static void diff_one(const std::string& key, const std::string& value, std::string& out) {
    std::string& old = g_seen[key];
    if (old.empty()) old = "0.0.0.0.0.0";
    if (old == value) return;
    old = value;
    out += " " + key + "=" + value;
}

static void diff(const Scene& s) {
    std::string out = "d";
    for (int layer = LAYER_GROUND; layer <= LAYER_WALL; layer += LAYER_WALL - LAYER_GROUND) {
        for (int x = 0; x <= s.w + 1; ++x) {
            for (int y = 0; y <= s.h + 1; ++y) {
                PathfindCell* c = pf()->getCell(static_cast<PathfindLayerEnum>(layer), x, y);
                if (!c) continue;
                diff_one(num(layer) + "." + num(x) + "." + num(y),
                         num(c->getFlags()) + "." + num(c->getGoalUnit()) + "." + num(c->getPosUnit()) + "." +
                             num(c->isAircraftGoal()) + "." + num(c->getGoalAircraft()) + "." +
                             num(c->hasInfo()),
                         out);
            }
        }
    }
    for (Object* o : s.units) {
        AIUpdateInterface* ai = o->getAIUpdateInterface();
        std::string v = num(o->getDestinationLayer()) + "." + num(o->getLayer());
        if (ai) {
            v += "." + num(ai->getPathfindGoalCell()->x) + "." + num(ai->getPathfindGoalCell()->y) + "." +
                 num(ai->getCurPathfindCell()->x) + "." + num(ai->getCurPathfindCell()->y);
        }
        diff_one("u" + num(o->getID()), v, out);
    }
    row(out);
}

static LocomotorSet* surface_set(int mask) {
    static LocomotorSet set;
    set.*access_member(LsSurfaces()) = mask;
    return &set;
}

// ---- operations -------------------------------------------------------------------

static void move_unit(Object* u, const Coord3D& p) {
    u->setPosition(&p);
    // The object makes this call itself when its integer position changes.
    pf()->updatePos(u, u->getPosition());
    row("move " + num(u->getID()) + " " + c3(*u->getPosition()));
}

static void reserve(const Scene& s) {
    Object* u = pick(s.units);
    const int k = below(100);
    if (k < 34) {
        const PathfindLayerEnum layer = some_layer(s);
        const Coord3D p = busy_point(s);
        row("goal " + num(u->getID()) + " " + c3(p) + " " + num(layer));
        pf()->updateGoal(u, &p, layer);
    } else if (k < 66) {
        move_unit(u, busy_point(s));
    } else if (k < 73) {
        row("rmgoal " + num(u->getID()));
        pf()->removeGoal(u);
    } else if (k < 78) {
        row("rmpos " + num(u->getID()));
        pf()->removePos(u);
    } else if (k < 82) {
        row("rmunit " + num(u->getID()));
        pf()->removeUnitFromPathfindMap(u);
    } else if (k < 90) {
        const PathfindLayerEnum layer = some_layer(s);
        row("setlayer " + num(u->getID()) + " " + num(layer));
        u->setLayer(layer);
    } else if (k < 95) {
        const PathfindLayerEnum layer = some_layer(s);
        pf()->updateLayer(u, layer);
        row("uplayer " + num(u->getID()) + " " + num(layer) + " = " + num(u->getLayer()));
    } else {
        // The aircraft reservation on its own, also for ground units.
        const Coord3D p = busy_point(s);
        row("airgoal " + num(u->getID()) + " " + c3(p));
        pf()->updateAircraftGoal(u, &p);
    }
    diff(s);
}

static void hold(const Scene& s) {
    for (int n = 0; n < 40; ++n) {
        Object* u = pick(s.units);
        AIUpdateInterface* ai = u->getAIUpdateInterface();
        if (!ai) continue;
        // Often at or over the west or south edge, where a cell can be 0 or less.
        Coord3D p = busy_point(s);
        if (chance(60)) p.x = between(-30.0f, 14.0f);
        else if (chance(50)) p.y = between(-30.0f, 14.0f);
        move_unit(u, p);
        diff(s);
        u->setDisabled(DISABLED_HELD);
        row("ground " + num(u->getID()) + " " + num(ai->isDoingGroundMovement()));
        move_unit(u, chance(50) ? p : busy_point(s));
        diff(s);
        u->clearDisabled(DISABLED_HELD);
        row("ground " + num(u->getID()) + " " + num(ai->isDoingGroundMovement()));
    }
}

// Lines that start on the wall and leave it, to come back or not.
static void along_walls(const Scene& s) {
    if (s.walls.empty()) return;
    for (int n = 0; n < 24; ++n) {
        Object* u = pick(s.units);
        const int mask = chance(60) ? 1 : kMasks[below(9)];
        auto on_wall = [&]() {
            Coord3D p = *pick(s.walls)->getPosition();
            p.x += between(-30.0f, 30.0f);
            p.y += between(-14.0f, 14.0f);
            p.z = 0.0f;
            return p;
        };
        const Coord3D from = on_wall();
        const Coord3D to = chance(75) ? on_wall() : busy_point(s);
        Coord3D tight = from;
        (pf()->*access_member(PfTighten()))(u, *surface_set(mask), &tight, &to);
        row("tighten " + num(u->getID()) + " " + num(mask) + " " + c3(from) + " " + c3(to) + " = " + c3(tight));
        Coord3D d = from;
        const bool ok = pf()->adjustDestination(u, *surface_set(mask), &d, nullptr);
        row("adjust " + num(u->getID()) + " " + num(mask) + " " + c3(from) + " = " + num(ok) + " " + c3(d));
    }
}

static void snaps(const Scene& s) {
    for (int n = 0; n < 20; ++n) {
        Object* u = pick(s.units);
        const Coord3D p = busy_point(s);
        Coord3D a = p;
        pf()->snapPosition(u, &a);
        row("snap " + num(u->getID()) + " " + c3(p) + " = " + c3(a));
        a = p;
        pf()->snapClosestGoalPosition(u, &a);
        row("snapgoal " + num(u->getID()) + " " + c3(p) + " = " + c3(a));
        Coord3D g = p;
        const bool has = pf()->goalPosition(u, &g);
        row("goalpos " + num(u->getID()) + " " + c3(p) + " = " + num(has) + " " + c3(g));
    }
}

// `checkPathCost` for every cell a candidate can be. Runs of equal costs are
// written as cost*count.
static void cost_table(const Scene& s, Object* u, int mask, const Coord3D& from) {
    Int radius;
    Bool center;
    (pf()->*access_member(PfRadius()))(u, radius, center);
    std::string out = "costs " + num(center);
    Int last = 0;
    int run = 0;
    for (int x = 0; x <= s.w + 1; ++x) {
        for (int y = 0; y <= s.h + 1; ++y) {
            Coord3D to;
            (pf()->*access_member(PfAdjustCoord()))(x, y, center, to, LAYER_GROUND);
            const Int cost = (pf()->*access_member(PfCost()))(u, *surface_set(mask), &from, &to);
            if (run && cost == last) {
                ++run;
                continue;
            }
            if (run) out += " " + num(last) + "*" + num(run);
            last = cost;
            run = 1;
        }
    }
    out += " " + num(last) + "*" + num(run);
    row(out);
}

static void searches(const Scene& s, int count, int groups) {
    for (int n = 0; n < count; ++n) {
        Object* u = pick(s.units);
        const int mask = chance(60) ? 1 : kMasks[below(9)];
        const Coord3D p = busy_point(s);
        Coord3D d = p;
        const bool ok = pf()->adjustDestination(u, *surface_set(mask), &d, nullptr);
        row("adjust " + num(u->getID()) + " " + num(mask) + " " + c3(p) + " = " + num(ok) + " " + c3(d));

        d = p;
        const bool possible = pf()->adjustToPossibleDestination(u, *surface_set(mask), &d);
        row("possible " + num(u->getID()) + " " + num(mask) + " " + c3(p) + " = " + num(possible) + " " +
            c3(d));

        Object* lander = pick(s.units);
        const Coord3D q = chance(70) ? busy_point(s) : point(s, 60);
        d = q;
        const bool lands = pf()->adjustToLandingDestination(lander, &d);
        row("land " + num(lander->getID()) + " " + c3(q) + " = " + num(lands) + " " + c3(d));

        Object* shooter = pick(s.units);
        const Coord3D r = busy_point(s);
        d = r;
        g_range_percent = chance(15) ? 0 : chance(50) ? 60 : 4;
        row("target " + num(shooter->getID()) + " " + c3(r));
        static char fake[16];
        const bool in = pf()->adjustTargetDestination(shooter, pick(s.units), &r,
                                                      reinterpret_cast<const Weapon*>(fake), &d);
        row("targeted = " + num(in) + " " + c3(d));

        const Coord3D from = busy_point(s);
        const Coord3D to = busy_point(s);
        Coord3D tight = from;
        (pf()->*access_member(PfTighten()))(u, *surface_set(mask), &tight, &to);
        row("tighten " + num(u->getID()) + " " + num(mask) + " " + c3(from) + " " + c3(to) + " = " + c3(tight));
    }
    for (int n = 0; n < groups; ++n) {
        Object* u = pick(s.units);
        const int mask = chance(75) ? 1 : kMasks[below(9)];
        const Coord3D group = busy_point(s);
        Coord3D p = group;
        const float reach = chance(60) ? 30.0f : 120.0f;
        p.x += between(-reach, reach);
        p.y += between(-reach, reach);
        cost_table(s, u, mask, group);
        Coord3D d = p;
        const bool ok = pf()->adjustDestination(u, *surface_set(mask), &d, &group);
        row("adjustgroup " + num(u->getID()) + " " + num(mask) + " " + c3(p) + " " + c3(group) + " = " +
            num(ok) + " " + c3(d));
        diff(s);
    }
}

static void allies(const Scene& s, int count) {
    for (int n = 0; n < count; ++n) {
        for (Object* o : s.units) {
            Ally& a = g_ally[o->getID()];
            a = Ally();
            a.moving = chance(20);
            a.attacking = chance(15);
            a.busy = chance(10);
            a.move_when_told = chance(40);
            AIUpdateInterface* ai = o->getAIUpdateInterface();
            if (ai && chance(20)) {
                ai->ignoreObstacle(pick(s.units));
                row("ignore " + num(o->getID()) + " " + num(ai->getIgnoredObstacleID()));
            }
            const bool ability = chance(10);
            if (ability) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_IS_USING_ABILITY));
            else o->clearStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_IS_USING_ABILITY));
            row("ally " + num(o->getID()) + " " + num(a.moving) + " " + num(a.attacking) + " " +
                num(ability) + " " + num(a.busy) + " " + num(a.move_when_told));
        }
        // A path through the places units stand in.
        auto make = [&](int index) {
            Path* path = newInstance(Path);
            const bool blocked = chance(60);
            path->setBlockedByAlly(blocked);
            std::string text = "path " + num(index) + " " + num(blocked);
            for (int k = 1 + below(6); k > 0; --k) {
                Coord3D p = busy_point(s);
                const PathfindLayerEnum layer = some_layer(s);
                path->appendNode(&p, layer);
                text += " " + c3(p) + " " + num(layer);
            }
            row(text);
            return path;
        };
        Path* main_path = make(0);
        std::vector<Path*> nested;
        int index = 1;
        for (Object* o : s.units) {
            if (!chance(35)) continue;
            Path* p = make(index);
            g_ally[o->getID()].path = p;
            row("nest " + num(o->getID()) + " " + num(index));
            nested.push_back(p);
            ++index;
        }
        Object* u = pick(s.units);
        row("allies " + num(u->getID()));
        const bool ran = (pf()->*access_member(PfAllies()))(u, main_path);
        row("allied = " + num(ran));
        // The same units, along straight lines.
        for (int k = 0; k < 3; ++k) {
            Object* w = pick(s.units);
            if (!w->getAIUpdateInterface()) continue;
            // Across the map or towards another unit.
            Coord3D to = chance(50) ? point(s) : *pick(s.units)->getPosition();
            row("madest " + num(w->getID()) + " " + c3(to));
            (pf()->*access_member(PfAway()))(w, to);
            row("madested");
        }
        deleteInstance(main_path);
        for (Path* p : nested) deleteInstance(p);
        for (Object* o : s.units) g_ally[o->getID()] = Ally();
        // Move a few units so that the next round meets other cells.
        for (int k = 0; k < 5; ++k) {
            move_unit(pick(s.units), busy_point(s));
            diff(s);
        }
    }
}

static void run_scene(int index) {
    Scene s;
    std::fprintf(stderr, "scene %d at %zu rows\n", index, g_rows.size());
    row("scenario " + num(index));
    s.w = 12 + below(16);
    s.h = 12 + below(12);
    s.maxx = s.w * 10.0f;
    s.maxy = s.h * 10.0f;
    S4Terrain& t = *g_terrain;
    t.w = s.w;
    t.h = s.h;
    t.cliff.assign(static_cast<size_t>((s.w + 2) * (s.h + 2)), '0');
    t.water = t.up = t.bridge_end = t.cliff;
    if (index % 4 != 0) {
        blobs(t.cliff, below(4));
        blobs(t.water, below(4));
    }
    blobs(t.bridge_end, 8);
    t.extent.lo.x = 0.0f;
    t.extent.lo.y = 0.0f;
    t.extent.lo.z = 0.0f;
    t.extent.hi.x = s.maxx;
    t.extent.hi.y = s.maxy;
    t.extent.hi.z = 100.0f;
    t.logical = t.extent;
    const float wall_height = 12.0f;
    const_cast<TAiData*>(TheAI->getAiData())->m_wallHeight = wall_height;
    pf()->reset();
    row("map " + f32(s.maxx) + " " + f32(s.maxy) + " " + f32(wall_height));
    row("terrain " + num(s.w) + " " + num(s.h) + " " + t.cliff + " " + t.water);
    g_world->set_frame(100);
    pf()->newMap();

    for (int n = 2 + below(index % 4 == 0 ? 3 : 8); n > 0; --n) {
        const std::string name = g_blocks[below(static_cast<int>(g_blocks.size()))];
        s.blocks.push_back(spawn(name, g_world->team_b, coord(0.0f, s.maxx), coord(0.0f, s.maxy), 0.0f,
                                 chance(50) ? 0.0f : between(-3.2f, 3.2f)));
    }
    if (index % 3 != 0) {
        float x = coord(30.0f, s.maxx * 0.5f);
        const float y = coord(30.0f, s.maxy - 30.0f);
        // Some walls have gaps.
        const float step = index % 3 == 1 ? 80.0f : 48.0f;
        for (int n = 1 + below(4); n > 0; --n) {
            s.walls.push_back(spawn("S4Wall", g_world->team_a, x, y, 0.0f, 0.0f));
            x += step;
        }
        // The wall is where destinations are said to be above the ground.
        for (Object* o : s.walls) {
            const Coord3D c = *o->getPosition();
            for (float dx = -30.0f; dx <= 30.0f; dx += 5.0f)
                for (float dy = -18.0f; dy <= 18.0f; dy += 5.0f) t.up[t.at_pos(c.x + dx, c.y + dy)] = '1';
        }
    }
    row("tables " + t.up + " " + t.bridge_end);
    for (int n = 9 + below(8); n > 0; --n) {
        // The special units come up more often than their share.
        static const int special[] = {13, 14, 15, 24, 25, 26, 27, 4, 7, 12};
        const int which = chance(40) ? special[below(10)] : below(30);
        Object* o = spawn(g_units[which], chance(65) ? g_world->team_a : g_world->team_b,
                          coord(0.0f, s.maxx), coord(0.0f, s.maxy), chance(15) ? 30.0f : 0.0f, 0.0f);
        if (o->isKindOf(KINDOF_PRODUCED_AT_HELIPAD) && chance(60)) o->setDisabled(DISABLED_UNMANNED);
        s.units.push_back(o);
    }
    for (Object* o : s.units) {
        if (o->getAIUpdateInterface() && !s.blocks.empty() && chance(35)) {
            o->getAIUpdateInterface()->ignoreObstacle(pick(s.blocks));
        }
    }
    s.everything = s.blocks;
    s.everything.insert(s.everything.end(), s.walls.begin(), s.walls.end());
    s.everything.insert(s.everything.end(), s.units.begin(), s.units.end());
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
    // Every unit starts off the map: spawning had already run `updatePos`.
    for (Object* o : s.units) {
        AIUpdateInterface* ai = o->getAIUpdateInterface();
        o->*access_member(ObjLayer()) = LAYER_GROUND;
        o->setDestinationLayer(LAYER_GROUND);
        if (!ai) continue;
        ICoord2D none;
        none.x = none.y = -1;
        ai->setCurPathfindCell(none);
        ai->setPathfindGoalCell(none);
    }
    for (int x = 0; x <= s.w + 1; ++x) {
        for (int y = 0; y <= s.h + 1; ++y) {
            for (int layer = LAYER_GROUND; layer <= LAYER_WALL; layer += LAYER_WALL - LAYER_GROUND) {
                PathfindCell* cell = pf()->getCell(static_cast<PathfindLayerEnum>(layer), x, y);
                if (!cell) continue;
                ICoord2D at;
                at.x = x;
                at.y = y;
                cell->setGoalUnit(INVALID_ID, at);
                cell->setPosUnit(INVALID_ID, at);
                cell->setGoalAircraft(INVALID_ID, at);
            }
        }
    }
    unit_rows(s);
    // A smaller playable area, which only people are held to.
    if (index % 2 == 0) {
        t.logical.lo.x = 20.0f;
        t.logical.lo.y = 30.0f;
        t.logical.hi.x = s.maxx - 30.0f;
        t.logical.hi.y = s.maxy - 20.0f;
        pf()->processPathfindQueue();
        row("logical " + f32(t.logical.lo.x) + " " + f32(t.logical.lo.y) + " " + f32(t.logical.hi.x) + " " +
            f32(t.logical.hi.y));
    }
    g_seen.clear();
    // The cells start clean; the units' own state is the first difference.
    diff(s);

    for (int round = 0; round < 3; ++round) {
        for (int n = 0; n < 45; ++n) reserve(s);
        snaps(s);
        searches(s, 12, index % 3 == 2 ? 3 : 2);
    }
    along_walls(s);
    hold(s);
    allies(s, 14);
    // A crowded map: searches run to the end of the spiral and fail.
    for (int n = 0; n < 220; ++n) {
        Object* u = pick(s.units);
        const Coord3D p = point(s, 0);
        row("goal " + num(u->getID()) + " " + c3(p) + " " + num(LAYER_GROUND));
        pf()->updateGoal(u, &p, LAYER_GROUND);
        // Leave the cells behind taken, as a crowd of units would.
        AIUpdateInterface* ai = u->getAIUpdateInterface();
        if (ai && chance(75)) {
            ICoord2D none;
            none.x = none.y = -1;
            ai->setPathfindGoalCell(none);
            row("forget " + num(u->getID()));
            // A goal two cells off, then let go: only that one is cleared.
            if (chance(50)) {
                Coord3D q = p;
                q.x += chance(50) ? 20.0f : -10.0f;
                q.y += chance(50) ? 0.0f : 10.0f;
                row("goal " + num(u->getID()) + " " + c3(q) + " " + num(LAYER_GROUND));
                pf()->updateGoal(u, &q, LAYER_GROUND);
                diff(s);
                row("rmgoal " + num(u->getID()));
                pf()->removeGoal(u);
            }
        }
        diff(s);
    }
    snaps(s);
    searches(s, 12, 2);

    for (Object* o : s.units) pf()->removeUnitFromPathfindMap(o);
    for (Object* o : s.everything) TheGameLogic->destroyObject(o);
    TheGameLogic->processDestroyList();
    ThePartitionManager->update();
    TheTerrainLogic->reset();
    pf()->reset();
    g_ally.clear();
    row("end");
}

int main(int argc, char** argv) {
    if (argc < 2) return 2;
    oracle_wp51::World world;
    g_world = &world;
    (TheModuleFactory->*(&ExposedFactory::addModuleInternal))(
        make_s4_ai, AIUpdateInterface::friend_newModuleData, MODULETYPE_BEHAVIOR, "S4AI",
        AIUpdateInterface::getInterfaceMask());
    make_templates();
    world.load_ini(g_ini);
    g_terrain = new S4Terrain;
    TheTerrainLogic = g_terrain;
    Player* computer = world.team_b->getControllingPlayer();
    if (!computer) {
        std::fprintf(stderr, "team B has no player\n");
        return 1;
    }
    computer->*access_member(PlayerKind()) = PLAYER_COMPUTER;
    for (int index = 0; index < 8; ++index) run_scene(index);

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
