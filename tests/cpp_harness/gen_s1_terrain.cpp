// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for slice S1: the reference map loaders (W3DTerrainLogic::loadMap,
// WorldHeightMap) and terrain queries (TerrainLogic, W3DTerrainLogic,
// BaseHeightMapRenderObjClass, W3DTerrainVisual) over maps this harness writes
// with the reference DataChunkOutput.
//
//   CPATH=vendor/GeneralsX/GeneralsMD/Code/GameEngineDevice/Include:vendor/GeneralsX/Core/GameEngineDevice/Include \
//     python3 scripts/build_oracle.py tests/cpp_harness/gen_s1_terrain.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/S1_Terrain.json
//
// Retail mode, local only: a list file of extracted map files, one
// "<key> <path>" per line. Only digests are written.
//
//   /tmp/gen --retail <list> <digests.json>
//
// The render object is zeroed memory whose virtual calls do nothing, with its
// height map and maximum height set as initHeightData sets them. The logic height
// map is made as W3DTerrainVisual::load makes it. Nothing else of the device is
// run.
//
// Not covered: the water grid, bridge damage and repair, setActiveBoundary's
// effects, walls with pieces (only the wall height), loading a save.

#include "oracle_wp51_world.h"

#include "Common/DataChunk.h"
#include "Common/FileSystem.h"
#include "Common/MapObject.h"
#include "Common/TerrainTypes.h"
#include "Common/WellKnownKeys.h"
#include "GameClient/MapUtil.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/PolygonTrigger.h"
#include "GameLogic/TerrainLogic.h"
#include "StdDevice/Common/StdLocalFileSystem.h"
#include "W3DDevice/GameClient/BaseHeightMap.h"
#include "W3DDevice/GameClient/HeightMap.h"
#include "W3DDevice/GameClient/W3DTerrainVisual.h"
#include "W3DDevice/GameClient/WorldHeightMap.h"
#include "W3DDevice/GameLogic/W3DTerrainLogic.h"

#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>

// ---- private members ---------------------------------------------------------

#define S1_TAG(Tag, Type, Class, Member)                   \
    struct Tag { using type = Type Class::*; };            \
    template struct MemberAccess<Tag, &Class::Member>;     \
    inline Type Class::* access_member(Tag);

S1_TAG(HmWidth, Int, WorldHeightMap, m_width)
S1_TAG(HmHeight, Int, WorldHeightMap, m_height)
S1_TAG(HmBorder, Int, WorldHeightMap, m_borderSize)
S1_TAG(HmDataSize, Int, WorldHeightMap, m_dataSize)
S1_TAG(HmData, UnsignedByte*, WorldHeightMap, m_data)
S1_TAG(BhmMap, WorldHeightMap*, BaseHeightMapRenderObjClass, m_map)
S1_TAG(BhmMaxHeight, Real, BaseHeightMapRenderObjClass, m_maxHeight)
S1_TAG(VisLogicMap, WorldHeightMap*, W3DTerrainVisual, m_logicHeightMap)
S1_TAG(VisRender, BaseHeightMapRenderObjClass*, W3DTerrainVisual, m_terrainRenderObject)
S1_TAG(TlBoundaries, VecICoord2D, TerrainLogic, m_boundaries)
S1_TAG(PfWallHeight, Real, Pathfinder, m_wallHeight)

struct RectTag { using type = void (TerrainLogic::*)(const WaterHandle*, Region3D*); };
template struct MemberAccess<RectTag, &TerrainLogic::findAxisAlignedBoundingRect>;
inline void (TerrainLogic::* access_member(RectTag))(const WaterHandle*, Region3D*);

extern "C" void* _ZTV23HeightMapRenderObjClass[];

// The one virtual call that reaches the render object, staticLightingChanged,
// updates rendering buffers that do not exist here.
static void no_rendering() {}
static void* g_render_vtable[256];

static void** render_vtable() {
    g_render_vtable[0] = _ZTV23HeightMapRenderObjClass[0];  // offset to top
    g_render_vtable[1] = _ZTV23HeightMapRenderObjClass[1];  // type info
    for (int i = 2; i < 256; ++i) g_render_vtable[i] = reinterpret_cast<void*>(&no_rendering);
    return &g_render_vtable[2];
}

// ---- formatting --------------------------------------------------------------

static std::string f32(float v) {
    uint32_t bits;
    std::memcpy(&bits, &v, 4);
    char buf[16];
    std::snprintf(buf, sizeof(buf), "%08x", bits);
    return buf;
}
static std::string c3(const Coord3D& c) { return f32(c.x) + " " + f32(c.y) + " " + f32(c.z); }
static std::string num(long long v) { return std::to_string(v); }
static std::string word(const AsciiString& s) { return s.isEmpty() ? "~" : s.str(); }

static uint64_t fnv(uint64_t h, const void* data, size_t len) {
    const unsigned char* p = static_cast<const unsigned char*>(data);
    for (size_t i = 0; i < len; ++i) {
        h ^= p[i];
        h *= 0x100000001b3ULL;
    }
    return h;
}
static const uint64_t kFnv = 0xcbf29ce484222325ULL;
static std::string hex64(uint64_t v) {
    char buf[24];
    std::snprintf(buf, sizeof(buf), "%016llx", static_cast<unsigned long long>(v));
    return buf;
}

static uint32_t g_state = 0x51000001u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
static float unit() { return static_cast<float>(pick(1000000)) / 1000000.0f; }
static float between(float lo, float hi) { return lo + unit() * (hi - lo); }

// ---- synthetic maps ----------------------------------------------------------

struct MemoryStream : public OutputStream {
    std::string bytes;
    Int write(const void* data, Int len) override {
        bytes.append(static_cast<const char*>(data), len);
        return len;
    }
};

struct Spec {
    int width, height, border;
    int height_version, blend_version, objects_version, trigger_version;
    int boundaries;
    int terrain;  // 0 noise, 1 hills, 2 plateaus
    bool water_grid;
};

static std::vector<unsigned char> make_samples(const Spec& s) {
    std::vector<unsigned char> d(static_cast<size_t>(s.width) * s.height);
    for (int y = 0; y < s.height; ++y) {
        for (int x = 0; x < s.width; ++x) {
            int h = 0;
            if (s.terrain == 0) {
                h = static_cast<int>(pick(256));
            } else if (s.terrain == 1) {
                h = 40 + x * 3 + y * 2 + static_cast<int>(pick(7));
                if (pick(40) == 0) h += 60;
            } else {
                h = ((x / 5 + y / 4) % 4) * 23 + 10;
                if (pick(6) == 0) h += static_cast<int>(pick(20));
            }
            d[static_cast<size_t>(y) * s.width + x] = static_cast<unsigned char>(h > 255 ? 255 : h);
        }
    }
    return d;
}

static void write_height_chunk(DataChunkOutput& out, const Spec& s) {
    std::vector<unsigned char> data = make_samples(s);
    out.openDataChunk("HeightMapData", s.height_version);
    out.writeInt(s.width);
    out.writeInt(s.height);
    if (s.height_version >= 3) out.writeInt(s.border);
    if (s.height_version >= 4) {
        out.writeInt(s.boundaries);
        const int w = s.width - 2 * s.border, h = s.height - 2 * s.border;
        for (int i = 0; i < s.boundaries; ++i) {
            out.writeInt(w - i * 2);
            out.writeInt(h - i * 3);
        }
    }
    out.writeInt(static_cast<Int>(data.size()));
    out.writeArrayOfBytes(reinterpret_cast<char*>(data.data()), static_cast<Int>(data.size()));
    out.closeDataChunk();
}

static void write_blend_chunk(DataChunkOutput& out, const Spec& s) {
    const int v = s.blend_version;
    const int n = s.width * s.height;
    std::vector<char> shorts(static_cast<size_t>(n) * 2, 0);
    out.openDataChunk("BlendTileData", v);
    out.writeInt(n);
    int arrays = 2 + (v >= 6 ? 1 : 0) + (v >= 5 ? 1 : 0);
    for (int i = 0; i < arrays; ++i) out.writeArrayOfBytes(shorts.data(), n * 2);
    if (v >= 7) {
        const int row = v == 7 ? (s.width + 1) / 8 : (s.width + 7) / 8;
        std::vector<char> flags(static_cast<size_t>(row) * s.height);
        for (char& b : flags) b = static_cast<char>(pick(4) == 0 ? pick(256) : 0);
        out.writeArrayOfBytes(flags.data(), static_cast<Int>(flags.size()));
    }
    out.writeInt(1);               // bitmap tiles
    out.writeInt(1);               // blended tiles
    if (v >= 5) out.writeInt(1);   // cliff infos
    out.writeInt(0);               // texture classes
    if (v >= 4) {
        out.writeInt(0);           // edge tiles
        out.writeInt(0);           // edge texture classes
    }
    out.closeDataChunk();
}

static void write_objects(DataChunkOutput& out, const Spec& s) {
    const float wx = (s.width - 2 * s.border) * 10.0f, wy = (s.height - 2 * s.border) * 10.0f;
    const int v = s.objects_version;
    out.openDataChunk("ObjectsList", v);
    const int count = 22;
    int waypoint = 0;
    for (int i = 0; i < count; ++i) {
        out.openDataChunk("Object", v);
        out.writeReal(between(-20.0f, wx + 20.0f));
        out.writeReal(between(-20.0f, wy + 20.0f));
        float z = between(-30.0f, 30.0f);
        if (pick(9) == 0) z = pick(2) ? -1000.5f : 1593.8f;
        if (pick(11) == 0) z = pick(2) ? -1000.0f : 1593.75f;
        out.writeReal(z);
        out.writeReal(between(-20.0f, 20.0f));
        out.writeInt(static_cast<Int>(pick(512)));
        out.writeAsciiString(AsciiString(("Thing" + num(i)).c_str()));
        if (v >= 2) {
            Dict d;
            const int kind = static_cast<int>(pick(6));
            if (kind <= 3) {
                ++waypoint;
                const int id = pick(8) == 0 ? 1 + static_cast<int>(pick(4)) : waypoint;
                d.setInt(NAMEKEY("waypointID"), id);
                std::string name = pick(8) == 0 ? std::string("Spot") : "Way" + num(id);
                if (s.water_grid && waypoint == 2) name = "WaveGuide1";
                d.setAsciiString(NAMEKEY("waypointName"), AsciiString(name.c_str()));
                static const char* labels[] = {"Attack", "attack", "Patrol", "Flank", ""};
                static const char* keys[] = {"waypointPathLabel1", "waypointPathLabel2",
                                             "waypointPathLabel3"};
                for (int l = 0; l < 3; ++l) {
                    if (pick(3) == 0) {
                        d.setAsciiString(NAMEKEY(keys[l]), AsciiString(labels[pick(5)]));
                    }
                }
                if (pick(2) == 0) d.setBool(NAMEKEY("waypointPathBiDirectional"), pick(3) != 0);
            } else if (kind == 4) {
                d.setReal(NAMEKEY("lightHeightAboveTerrain"), between(0.0f, 5.0f));
            } else {
                d.setInt(NAMEKEY("scorchType"), static_cast<int>(pick(4)));
            }
            out.writeDict(d);
        }
        out.closeDataChunk();
    }
    out.closeDataChunk();
}

static void write_waypoint_links(DataChunkOutput& out) {
    out.openDataChunk("WaypointsList", 1);
    const int count = 30;
    out.writeInt(count);
    for (int i = 0; i < count; ++i) {
        // One waypoint gets more links than it can hold.
        out.writeInt(i < 11 ? 1 : 1 + static_cast<int>(pick(17)));
        out.writeInt(i < 11 ? 2 + i : 1 + static_cast<int>(pick(17)));
    }
    out.closeDataChunk();
}

static void write_triggers(DataChunkOutput& out, const Spec& s) {
    const int wx = (s.width - 2 * s.border) * 10, wy = (s.height - 2 * s.border) * 10;
    const int v = s.trigger_version;
    out.openDataChunk("PolygonTriggers", v);
    const int count = 7;
    out.writeInt(count);
    for (int i = 0; i < count; ++i) {
        out.writeAsciiString(AsciiString(("Area" + num(i == 5 ? 1 : i)).c_str()));
        if (v >= 4) out.writeAsciiString(AsciiString(""));
        out.writeInt(10 + i * 3);
        if (v >= 2) out.writeByte(i % 4 != 3);
        if (v >= 3) {
            out.writeByte(0);
            out.writeInt(0);
        }
        const int points = i == 2 ? 1 : 3 + static_cast<int>(pick(5));
        out.writeInt(points);
        const int cx = static_cast<int>(pick(wx + 1)), cy = static_cast<int>(pick(wy + 1));
        const int z = i == 4 ? -5 : static_cast<int>(pick(90));
        for (int p = 0; p < points; ++p) {
            out.writeInt(cx + static_cast<int>(pick(wx + 1)) - wx / 2);
            out.writeInt(cy + static_cast<int>(pick(wy + 1)) - wy / 2);
            out.writeInt(p == 0 ? z : z + static_cast<int>(pick(5)));
        }
    }
    out.closeDataChunk();
}

static std::string make_map(const Spec& s) {
    MemoryStream stream;
    {
        DataChunkOutput out(&stream);
        write_height_chunk(out, s);
        write_blend_chunk(out, s);
        out.openDataChunk("WorldInfo", 1);
        Dict world;
        world.setInt(NAMEKEY("weather"), 1);
        out.writeDict(world);
        out.closeDataChunk();
        write_objects(out, s);
        write_triggers(out, s);
        write_waypoint_links(out);
    }
    return stream.bytes;
}

// ---- the world ---------------------------------------------------------------

struct Terrain {
    W3DTerrainLogic* logic = nullptr;
    W3DTerrainVisual* visual = nullptr;
    BaseHeightMapRenderObjClass* render = nullptr;

    void create() {
        TheWritableGlobalData->m_mapName = "s1_oracle";
        TheWritableGlobalData->m_waterExtentX = 70.0f;
        TheWritableGlobalData->m_waterExtentY = 90.0f;
        TheFileSystem = NEW FileSystem;
        TheLocalFileSystem = NEW StdLocalFileSystem;
        TheTerrainTypes = NEW TerrainTypeCollection;
        static void* map_cache[64];
        TheMapCache = reinterpret_cast<MapCache*>(map_cache);

        logic = NEW W3DTerrainLogic;
        TheTerrainLogic = logic;
        visual = new W3DTerrainVisual;
        TheTerrainVisual = visual;
        void* memory = std::calloc(1, sizeof(HeightMapRenderObjClass));
        *static_cast<void**>(memory) = render_vtable();
        render = static_cast<HeightMapRenderObjClass*>(memory);
        TheTerrainRenderObject = render;
        visual->*access_member(VisRender()) = render;
    }

    WorldHeightMap* map() { return visual->*access_member(VisLogicMap()); }

    void load(const std::string& path) {
        logic->reset();
        visual->*access_member(VisLogicMap()) = nullptr;
        render->*access_member(BhmMap()) = nullptr;
        if (!logic->loadMap(AsciiString(path.c_str()), TRUE)) {
            std::fprintf(stderr, "loadMap failed: %s\n", path.c_str());
            std::exit(1);
        }
        CachedFileInputStream stream;
        if (!stream.open(AsciiString(path.c_str()))) std::exit(1);
        WorldHeightMap* map = NEW WorldHeightMap(&stream, false);
        visual->*access_member(VisLogicMap()) = map;
        render->*access_member(BhmMap()) = map;
        // BaseHeightMapRenderObjClass::initHeightData.
        Int low = map->getMaxHeightValue(), high = 0;
        for (Int j = 0; j < map->getYExtent(); j++) {
            for (Int i = 0; i < map->getXExtent(); i++) {
                Short cur = map->getHeight(i, j);
                if (cur < low) low = cur;
                if (high < cur) high = cur;
            }
        }
        render->*access_member(BhmMaxHeight()) = high * MAP_HEIGHT_SCALE;
        logic->TerrainLogic::newMap(false);
    }
};

// ---- rows --------------------------------------------------------------------

static std::string cliff_digest(WorldHeightMap* map) {
    uint64_t h = kFnv;
    for (Int y = -1; y <= map->getYExtent(); ++y) {
        for (Int x = -1; x <= map->getXExtent(); ++x) {
            const unsigned char bit = map->getCliffState(x, y) ? 1 : 0;
            h = fnv(h, &bit, 1);
        }
    }
    return hex64(h);
}

static std::string sample_digest(WorldHeightMap* map) {
    const Int size = map->*access_member(HmDataSize());
    return hex64(fnv(kFnv, map->*access_member(HmData()), static_cast<size_t>(size)));
}

static std::string row_map(Terrain& t) {
    WorldHeightMap* m = t.map();
    std::string r = "HM " + num(m->getXExtent()) + " " + num(m->getYExtent()) + " " +
                    num(m->getBorderSizeInline()) + " " + num(m->*access_member(HmDataSize())) + " " +
                    sample_digest(m) + " " + cliff_digest(m);
    const VecICoord2D& b = t.logic->*access_member(TlBoundaries());
    r += " " + num(static_cast<long long>(b.size()));
    for (const ICoord2D& c : b) r += " " + num(c.x) + " " + num(c.y);
    r += " " + f32(t.render->getMaxHeight());
    return r;
}

static std::string row_extents(Terrain& t) {
    Region3D e, b, p;
    t.logic->getExtent(&e);
    b.lo.z = b.hi.z = 0.0f;
    t.logic->getExtentIncludingBorder(&b);
    t.logic->getMaximumPathfindExtent(&p);
    return "EXT " + c3(e.lo) + " " + c3(e.hi) + " " + c3(b.lo) + " " + c3(b.hi) + " " + c3(p.lo) +
           " " + c3(p.hi);
}

static void rows_lists(Terrain& t, std::vector<std::string>& rows) {
    for (MapObject* o = MapObject::getFirstMapObject(); o; o = o->getNext()) {
        rows.push_back("OBJ " + word(o->getName()) + " " + c3(*o->getLocation()) + " " +
                       f32(o->getAngle()) + " " + num(o->getFlags()) + " " + num(o->isWaypoint()) +
                       " " + num(o->isLight()) + " " + num(o->isScorch()));
    }
    for (Waypoint* w = t.logic->getFirstWaypoint(); w; w = w->getNext()) {
        std::string r = "WP " + num(w->getID()) + " " + word(w->getName()) + " " +
                        c3(*w->getLocation()) + " " + word(w->getPathLabel1()) + " " +
                        word(w->getPathLabel2()) + " " + word(w->getPathLabel3()) + " " +
                        num(w->getBiDirectional()) + " " + num(w->getNumLinks());
        for (Int i = 0; i < w->getNumLinks(); ++i) r += " " + num(w->getLink(i)->getID());
        rows.push_back(r);
    }
    for (PolygonTrigger* p = PolygonTrigger::getFirstPolygonTrigger(); p; p = p->getNext()) {
        std::string r = "TR " + num(p->getID()) + " " + word(p->getTriggerName()) + " " +
                        num(p->isWaterArea()) + " " + num(p->getNumPoints());
        for (Int i = 0; i < p->getNumPoints(); ++i) {
            const ICoord3D* c = p->getPoint(i);
            r += " " + num(c->x) + " " + num(c->y) + " " + num(c->z);
        }
        rows.push_back(r);
    }
}

static std::string row_height(Terrain& t, float x, float y) {
    Coord3D n;
    n.x = n.y = n.z = -7.0f;
    const Real with = TheTerrainLogic->getGroundHeight(x, y, &n);
    const Real without = TheTerrainLogic->getGroundHeight(x, y);
    return "H " + f32(x) + " " + f32(y) + " = " + f32(with) + " " + c3(n) + " " + f32(without) + " " +
           f32(t.render->getMaxCellHeight(x, y)) + " " + num(TheTerrainLogic->isCliffCell(x, y) ? 1 : 0);
}

static std::string row_los(const Coord3D& a, const Coord3D& b) {
    return "LOS " + c3(a) + " " + c3(b) + " = " + num(TheTerrainLogic->isClearLineOfSight(a, b));
}

static std::string row_water(float x, float y) {
    Real water = -12345.0f, ground = -12345.0f;
    const Bool under = TheTerrainLogic->isUnderwater(x, y, &water, &ground);
    return "UW " + f32(x) + " " + f32(y) + " = " + num(under) + " " + f32(water) + " " + f32(ground);
}

// The wall and bridge answers, so the other side can replay them.
static std::string elevated_tape(const Coord3D& at, PathfindLayerEnum layer, Bool clip) {
    std::string r = f32(TheAI->pathfinder()->getWallHeight()) + " " +
                    num(TheAI->pathfinder()->isPointOnWall(&at));
    int decks = 0;
    std::string list;
    for (Bridge* b = TheTerrainLogic->getFirstBridge(); b; b = b->getNext()) {
        if (!b->isPointOnBridge(&at)) continue;
        ++decks;
        list += " " + num(b->getLayer()) + " " + f32(b->getBridgeHeight(&at, nullptr)) + " " +
                num(b->peekBridgeInfo()->curDamageState == BODY_RUBBLE);
    }
    r += " " + num(decks) + list;
    Coord3D on = at;
    on.z = TheTerrainLogic->getGroundHeight(at.x, at.y);
    Bridge* found = nullptr;
    if (layer != LAYER_GROUND && layer != LAYER_WALL) {
        found = TheTerrainLogic->findBridgeLayerAt(&on, layer, clip);
    }
    if (found) {
        Coord3D n;
        n.x = n.y = n.z = -9.0f;
        const Real h = found->getBridgeHeight(&on, &n);
        r += " 1 " + f32(h) + " " + c3(n);
    } else {
        r += " 0";
    }
    return r;
}

static std::string row_layer_height(float x, float y, PathfindLayerEnum layer, Bool clip) {
    Coord3D at;
    at.x = x;
    at.y = y;
    at.z = 0.0f;
    const std::string tape = elevated_tape(at, layer, clip);
    Coord3D n;
    n.x = n.y = n.z = -7.0f;
    const Real h = TheTerrainLogic->getLayerHeight(x, y, layer, &n, clip);
    return "LH " + f32(x) + " " + f32(y) + " " + num(layer) + " " + num(clip) + " | " + tape + " = " +
           f32(h) + " " + c3(n);
}

static std::string row_layers(const Coord3D& at) {
    const std::string tape = elevated_tape(at, LAYER_GROUND, TRUE);
    return "LAY " + c3(at) + " | " + tape + " = " + num(TheTerrainLogic->getLayerForDestination(&at)) +
           " " + num(TheTerrainLogic->getHighestLayerForDestination(&at, FALSE)) + " " +
           num(TheTerrainLogic->getHighestLayerForDestination(&at, TRUE));
}

static std::string row_align(float angle, const Coord3D& at, Bool stick) {
    const PathfindLayerEnum layer = TheTerrainLogic->getLayerForDestination(&at);
    const std::string tape = elevated_tape(at, layer, TRUE);
    Matrix3D m(true);
    TheTerrainLogic->alignOnTerrain(angle, at, stick, m);
    std::string r = "AL " + f32(angle) + " " + c3(at) + " " + num(stick) + " | " + tape + " = " +
                    num(layer);
    for (int i = 0; i < 3; ++i) {
        for (int j = 0; j < 4; ++j) r += " " + f32(m[i][j]);
    }
    return r;
}

static std::string row_edges(const Coord3D& at) {
    const Coord3D c = TheTerrainLogic->findClosestEdgePoint(&at);
    const Coord3D f = TheTerrainLogic->findFarthestEdgePoint(&at);
    return "EDGE " + c3(at) + " = " + c3(c) + " " + c3(f);
}

static std::string id_of(Waypoint* w) { return w ? num(w->getID()) + " " + word(w->getName()) : "-"; }

static std::string water_name(const WaterHandle* h) {
    if (!h) return "none";
    if (h == TheTerrainLogic->getWaterHandleByName(AsciiString("Water Grid"))) return "grid";
    return num(h->m_polygon->getID());
}

// ---- synthetic rows ----------------------------------------------------------

static void query_rows(Terrain& t, const Spec& s, std::vector<std::string>& rows, int count) {
    const float lo = -(s.border * 10.0f) - 25.0f;
    const float hx = s.width * 10.0f - s.border * 10.0f + 25.0f;
    const float hy = s.height * 10.0f - s.border * 10.0f + 25.0f;
    auto rx = [&]() {
        const uint32_t k = pick(8);
        if (k == 0) return static_cast<float>(static_cast<int>(between(lo, hx) / 10.0f) * 10);
        if (k == 1) return static_cast<float>(static_cast<int>(between(lo, hx) / 5.0f) * 5);
        return between(lo, hx);
    };
    auto ry = [&]() {
        const uint32_t k = pick(8);
        if (k == 0) return static_cast<float>(static_cast<int>(between(lo, hy) / 10.0f) * 10);
        if (k == 1) return static_cast<float>(static_cast<int>(between(lo, hy) / 5.0f) * 5);
        return between(lo, hy);
    };
    auto point = [&](float zlo, float zhi) {
        Coord3D c;
        c.x = rx();
        c.y = ry();
        c.z = between(zlo, zhi);
        return c;
    };

    for (int i = 0; i < count; ++i) {
        const float x = rx(), y = ry();
        rows.push_back(row_height(t, x, y));
    }
    for (int i = 0; i < count / 2; ++i) {
        Coord3D a = point(0.0f, 170.0f), b = point(0.0f, 170.0f);
        if (pick(3) == 0) {
            // Skimming the ground, where the comparison with the cell corners matters.
            a.z = TheTerrainLogic->getGroundHeight(a.x, a.y) + between(0.0f, 30.0f);
            b.z = TheTerrainLogic->getGroundHeight(b.x, b.y) + between(0.0f, 30.0f);
        }
        if (pick(10) == 0) {
            b.x = a.x;
            b.y = a.y;
        }
        if (pick(8) == 0) {
            a.z = between(150.0f, 200.0f);
            b.z = a.z + between(0.0f, 20.0f);
        }
        rows.push_back(row_los(a, b));
    }
    for (int i = 0; i < count / 2; ++i) {
        float x = rx(), y = ry();
        PolygonTrigger* p = PolygonTrigger::getFirstPolygonTrigger();
        if (p && pick(3) == 0) {
            // At a vertex, give or take the rounding.
            for (uint32_t skip = pick(5); skip && p->getNext(); --skip) p = p->getNext();
            const ICoord3D* v = p->getPoint(static_cast<Int>(pick(p->getNumPoints())));
            x = v->x + between(-1.0f, 1.0f);
            y = v->y + between(-1.0f, 1.0f);
        }
        rows.push_back(row_water(x, y));
    }
    const float wall = TheAI->pathfinder()->getWallHeight();
    for (int i = 0; i < count / 4; ++i) {
        const PathfindLayerEnum layer = static_cast<PathfindLayerEnum>(
            pick(4) == 0 ? LAYER_WALL : 1 + pick(3));
        const Bool clip = pick(2) == 0;
        float x = rx(), y = ry();
        if (pick(2) == 0) {
            x = between(20.0f, 120.0f);
            y = between(20.0f, 60.0f);
        }
        rows.push_back(row_layer_height(x, y, layer, clip));
    }
    for (int i = 0; i < count / 4; ++i) {
        Coord3D at = point(-10.0f, 200.0f);
        if (pick(2) == 0) {
            at.x = between(20.0f, 120.0f);
            at.y = between(20.0f, 60.0f);
        }
        if (pick(3) == 0) at.z = TheTerrainLogic->getGroundHeight(at.x, at.y) + between(-3.0f, wall);
        rows.push_back(row_layers(at));
    }
    for (int i = 0; i < count / 4; ++i) {
        Coord3D at = point(-10.0f, 200.0f);
        if (pick(2) == 0) {
            at.x = between(20.0f, 120.0f);
            at.y = between(20.0f, 60.0f);
        }
        rows.push_back(row_align(between(-7.0f, 7.0f), at, pick(2) == 0));
    }
    for (int i = 0; i < 24; ++i) {
        Coord3D at = point(-10.0f, 100.0f);
        if (pick(4) == 0) at.x = at.y;
        rows.push_back(row_edges(at));
    }

    static const char* names[] = {"Way1", "Way2", "Way3", "Way7", "Spot", "WaveGuide1", "way1", "No"};
    for (const char* n : names) {
        rows.push_back(std::string("WPN ") + n + " = " +
                       id_of(TheTerrainLogic->getWaypointByName(AsciiString(n))));
    }
    for (int id = 0; id < 20; ++id) {
        rows.push_back("WPI " + num(id) + " = " + id_of(TheTerrainLogic->getWaypointByID(id)));
    }
    static const char* labels[] = {"Attack", "ATTACK", "attack", "Patrol", "flank", "~", "Nothing"};
    for (int i = 0; i < 60; ++i) {
        const char* l = labels[pick(7)];
        const AsciiString label(l[0] == '~' ? "" : l);
        Coord3D at = point(0.0f, 10.0f);
        rows.push_back("NEAR " + std::string(l) + " " + c3(at) + " = " +
                       id_of(TheTerrainLogic->getClosestWaypointOnPath(&at, label)));
        Waypoint* w = TheTerrainLogic->getFirstWaypoint();
        const uint32_t skip = pick(8);
        for (uint32_t k = 0; k < skip && w; ++k) w = w->getNext();
        if (w) {
            rows.push_back("PUR " + num(skip) + " " + l + " = " +
                           num(TheTerrainLogic->isPurposeOfPath(w, label)));
        }
    }
    static const char* areas[] = {"Area0", "Area1", "Area2", "Area3", "Area4", "Area6", "Water Grid",
                                  "area0"};
    for (const char* a : areas) {
        std::string key = a;
        for (char& c : key) {
            if (c == ' ') c = '_';
        }
        PolygonTrigger* trig = TheTerrainLogic->getTriggerAreaByName(AsciiString(a));
        const WaterHandle* water = TheTerrainLogic->getWaterHandleByName(AsciiString(a));
        std::string r = "AREA " + key + " = " + (trig ? num(trig->getID()) : std::string("-")) + " " +
                        water_name(water) + " ";
        if (water && water_name(water) != "grid") {
            r += f32(TheTerrainLogic->getWaterHeight(water));
            Region3D rect;
            rect.zero();
            (TheTerrainLogic->*access_member(RectTag()))(water, &rect);
            r += " " + c3(rect.lo) + " " + c3(rect.hi);
        } else {
            r += "-";
        }
        rows.push_back(r);
    }
}

// ---- water changes -----------------------------------------------------------

static const int kVictims = 24;

struct Water {
    std::vector<Object*> victims;

    void spawn(oracle_wp51::World& w, const Spec& s) {
        const float wx = (s.width - 2 * s.border) * 10.0f, wy = (s.height - 2 * s.border) * 10.0f;
        for (int i = 0; i < kVictims; ++i) {
            Coord3D at;
            at.x = between(1.0f, wx - 1.0f);
            at.y = between(1.0f, wy - 1.0f);
            at.z = 0.0f;
            victims.push_back(w.spawn_object("S1Victim", nullptr, at, 0.0f));
        }
    }

    std::string positions() const {
        std::string r = "VICTIMS " + num(kVictims);
        for (Object* o : victims) r += " " + f32(o->getPosition()->x) + " " + f32(o->getPosition()->y);
        return r;
    }

    // Which victims took damage since the last call.
    std::string damaged() {
        std::string r;
        for (Object* o : victims) {
            BodyModuleInterface* body = o->getBodyModule();
            const bool hurt = body->getHealth() < body->getMaxHealth();
            r += hurt ? '1' : '0';
            if (hurt) {
                DamageInfo heal;
                heal.in.m_damageType = DAMAGE_HEALING;
                heal.in.m_amount = body->getMaxHealth();
                o->attemptHealing(heal.in.m_amount, nullptr);
            }
        }
        return r;
    }
};

// Whether the pathfind map was reclassified: a planted cell type is overwritten.
static void plant() {
    TheAI->pathfinder()->getCell(LAYER_GROUND, 1, 1)->setType(PathfindCell::CELL_RUBBLE);
}
static int replanted() {
    return TheAI->pathfinder()->getCell(LAYER_GROUND, 1, 1)->getType() != PathfindCell::CELL_RUBBLE;
}

static std::vector<const WaterHandle*> water_handles() {
    std::vector<const WaterHandle*> out;
    for (PolygonTrigger* p = PolygonTrigger::getFirstPolygonTrigger(); p; p = p->getNext()) {
        if (p->isWaterArea()) {
            out.push_back(TheTerrainLogic->getWaterHandleByName(p->getTriggerName()));
        }
    }
    return out;
}

static std::string water_state(Terrain& t, Water& water) {
    std::string r = num(replanted()) + " " + water.damaged();
    for (const WaterHandle* h : water_handles()) {
        r += " " + water_name(h) + ":" + f32(TheTerrainLogic->getWaterHeight(h));
        for (Int i = 0; i < h->m_polygon->getNumPoints(); ++i) r += "," + num(h->m_polygon->getPoint(i)->z);
    }
    r += " " + oracle_wp51::World::snapshot_save_hex(t.logic);
    return r;
}

static std::string victims_under(const Water& water) {
    std::string r;
    for (Object* o : water.victims) {
        r += TheTerrainLogic->isUnderwater(o->getPosition()->x, o->getPosition()->y) ? '1' : '0';
    }
    return r;
}

static void water_rows(Terrain& t, oracle_wp51::World& w, Water& water,
                       std::vector<std::string>& rows, int steps) {
    UnsignedInt frame = 1;
    rows.push_back(water.positions());
    for (int i = 0; i < steps; ++i) {
        std::vector<const WaterHandle*> handles = water_handles();
        if (handles.empty()) return;
        const WaterHandle* h = handles[pick(static_cast<uint32_t>(handles.size()))];
        const uint32_t op = pick(10);
        plant();
        if (op == 0) {
            float height = between(-10.0f, 160.0f);
            if (pick(3) == 0) height = TheTerrainLogic->getWaterHeight(h) + between(0.0f, 0.9f);
            const float damage = pick(2) ? between(1.0f, 5.0f) : 0.0f;
            const Bool force = pick(4) == 0;
            const std::string name = water_name(h);
            TheTerrainLogic->setWaterHeight(h, height, damage, force);
            rows.push_back("SETW " + name + " " + f32(height) + " " + f32(damage) + " " + num(force) +
                           " = " + water_state(t, water));
        } else if (op <= 2) {
            const float final_height = between(-20.0f, 170.0f);
            const float seconds = pick(4) == 0 ? 0.05f : between(0.1f, 2.0f);
            const float damage = pick(2) ? between(1.0f, 5.0f) : 0.0f;
            const std::string name = water_name(h);
            TheTerrainLogic->changeWaterHeightOverTime(h, final_height, seconds, damage);
            rows.push_back("CHG " + name + " " + f32(final_height) + " " + f32(seconds) + " " +
                           f32(damage) + " = " + water_state(t, water));
        } else {
            if (pick(3) == 0) frame += 30 - frame % 30;
            w.set_frame(frame);
            t.logic->update();
            rows.push_back("UPD " + num(frame) + " = " + water_state(t, water));
            ++frame;
        }
        if (pick(5) == 0) rows.push_back("UNDER = " + victims_under(water));
    }
}

// ---- flattening --------------------------------------------------------------

static void shape_rows(Terrain& t, oracle_wp51::World& w, const Spec& s,
                       std::vector<std::string>& rows, int count) {
    const float wx = (s.width - 2 * s.border) * 10.0f, wy = (s.height - 2 * s.border) * 10.0f;
    Coord3D at;
    at.x = at.y = 5.0f;
    at.z = 0.0f;
    Object* o = w.spawn_object("S1Shape", nullptr, at, 0.0f);
    for (int i = 0; i < count; ++i) {
        const int type = static_cast<int>(pick(3));
        const Bool small = pick(10) == 0;
        float major = between(1.0f, 45.0f), minor = between(1.0f, 30.0f);
        if (pick(8) == 0) major = between(0.0f, 3.0f);
        if (pick(8) == 0) major = static_cast<float>(10 * (1 + pick(3)));
        o->setGeometryInfo(GeometryInfo(static_cast<GeometryType>(type), small, 10.0f, major, minor));
        at.x = between(-30.0f, wx + 30.0f);
        at.y = between(-30.0f, wy + 30.0f);
        if (pick(4) == 0) {
            at.x = static_cast<float>(10 * static_cast<int>(at.x / 10.0f));
            at.y = static_cast<float>(10 * static_cast<int>(at.y / 10.0f));
        }
        o->setPosition(&at);
        o->setOrientation(pick(4) == 0 ? 0.0f : between(-4.0f, 4.0f));
        const std::string args = num(type) + " " + num(small) + " " + c3(*o->getPosition()) + " " +
                                 f32(o->getOrientation()) + " " + f32(major) + " " + f32(minor);
        const bool crater = pick(3) == 0;
        if (crater) {
            t.logic->createCraterInTerrain(o);
        } else {
            t.logic->flattenTerrain(o);
        }
        rows.push_back(std::string(crater ? "CRATER " : "FLAT ") + args + " = " + sample_digest(t.map()) +
                       " " + f32(TheTerrainLogic->getGroundHeight(at.x, at.y)));
    }
}

// ---- retail digests ----------------------------------------------------------

struct Digest {
    uint64_t h = kFnv;
    size_t rows = 0;
    void add(const std::string& row) {
        h = fnv(h, row.data(), row.size());
        const char newline = '\n';
        h = fnv(h, &newline, 1);
        ++rows;
    }
    std::string text() const { return num(static_cast<long long>(rows)) + ":" + hex64(h); }
};

static const int kGrid = 96;

// Grid point `i` of `kGrid` along an axis of `samples` height samples.
static float grid_at(int i, int samples, int border) {
    const long long span = samples * 100LL + 300;
    const long long tenths = i * span / (kGrid - 1) - border * 100LL - 150 + (i % 7) * 3;
    return static_cast<float>(tenths) / 10.0f;
}

static std::string retail_digests(Terrain& t) {
    WorldHeightMap* m = t.map();
    const int w = m->getXExtent(), h = m->getYExtent(), border = m->getBorderSizeInline();
    Digest head, objects, heights, cliffs, water, sight, align;
    head.add(row_map(t));
    head.add(row_extents(t));
    std::vector<std::string> lists;
    rows_lists(t, lists);
    size_t waypoints = 0, triggers = 0;
    for (const std::string& r : lists) {
        objects.add(r);
        if (r.compare(0, 3, "WP ") == 0) ++waypoints;
        if (r.compare(0, 3, "TR ") == 0) ++triggers;
    }
    for (Int y = -1; y <= h; ++y) {
        std::string bits;
        for (Int x = -1; x <= w; ++x) bits += m->getCliffState(x, y) ? '1' : '0';
        cliffs.add(bits);
    }
    for (int j = 0; j < kGrid; ++j) {
        for (int i = 0; i < kGrid; ++i) {
            const float x = grid_at(i, w, border), y = grid_at(j, h, border);
            heights.add(row_height(t, x, y));
            water.add(row_water(x, y));
        }
    }
    uint32_t saved = g_state;
    g_state = 0x51e7a11u;
    for (int i = 0; i < 600; ++i) {
        Coord3D a, b;
        a.x = grid_at(static_cast<int>(pick(kGrid)), w, border);
        a.y = grid_at(static_cast<int>(pick(kGrid)), h, border);
        b.x = a.x + between(-300.0f, 300.0f);
        b.y = a.y + between(-300.0f, 300.0f);
        a.z = TheTerrainLogic->getGroundHeight(a.x, a.y) + between(0.0f, 30.0f);
        b.z = TheTerrainLogic->getGroundHeight(b.x, b.y) + between(0.0f, 30.0f);
        sight.add(row_los(a, b));
        a.z = 0.0f;
        align.add(row_align(between(-3.0f, 3.0f), a, TRUE));
        align.add(row_edges(a));
    }
    g_state = saved;
    return "w=" + num(w) + " h=" + num(h) + " wp=" + num(static_cast<long long>(waypoints)) +
           " tr=" + num(static_cast<long long>(triggers)) + " map=" + head.text() +
           " lists=" + objects.text() + " heights=" + heights.text() + " cliffs=" + cliffs.text() +
           " water=" + water.text() + " sight=" + sight.text() + " align=" + align.text();
}

// ---- main --------------------------------------------------------------------

static void write_file(const std::string& path, const std::string& bytes) {
    std::ofstream f(path, std::ios::binary);
    f.write(bytes.data(), static_cast<std::streamsize>(bytes.size()));
}

static std::string hex_of(const std::string& bytes) {
    static const char* digits = "0123456789abcdef";
    std::string out;
    out.reserve(bytes.size() * 2);
    for (unsigned char c : bytes) {
        out += digits[c >> 4];
        out += digits[c & 15];
    }
    return out;
}

int main(int argc, char** argv) {
    const bool retail = argc == 4 && std::strcmp(argv[1], "--retail") == 0;
    if (!retail && argc != 2) {
        std::fprintf(stderr, "usage: gen out.json | gen --retail list out.json\n");
        return 1;
    }
    static oracle_wp51::World w;
    w.load_ini(
        "Object S1Victim\n  KindOf = SELECTABLE\n  Geometry = CYLINDER\n  GeometryMajorRadius = 1\n"
        "  GeometryHeight = 5\n  GeometryIsSmall = Yes\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100000\n    InitialHealth = 100000\n  End\nEnd\n"
        "Object S1Shape\n  KindOf = STRUCTURE\n  Geometry = BOX\n  GeometryMajorRadius = 20\n"
        "  GeometryMinorRadius = 10\n  GeometryHeight = 10\n  GeometryIsSmall = No\n"
        "  Body = InactiveBody ModuleTag_Body\n  End\nEnd\n");
    static Terrain t;
    t.create();

    if (retail) {
        std::ifstream list(argv[2]);
        std::string key, path, out = "{\n \"maps\": [";
        int count = 0;
        while (list >> key >> path) {
            t.load(path);
            out += std::string(count ? "," : "") + "\n  \"" + key + " " + retail_digests(t) + "\"";
            ++count;
            std::fprintf(stderr, "map %d done\n", count);
        }
        out += "\n ]\n}\n";
        write_file(argv[3], out);
        return count ? 0 : 1;
    }

    static const Spec specs[] = {
        // width height border height blend objects triggers bounds terrain grid
        {48, 40, 5, 4, 8, 3, 4, 3, 1, false},
        {41, 37, 4, 4, 7, 3, 3, 1, 2, true},
        {33, 29, 0, 4, 6, 2, 2, 2, 0, false},
        {30, 34, 6, 3, 5, 3, 1, 1, 2, false},
        {26, 22, 0, 2, 3, 2, 4, 1, 1, false},
        {40, 32, 3, 4, 1, 1, 4, 2, 2, false},
        {44, 36, 0, 1, 1, 3, 4, 1, 1, false},
        {64, 64, 8, 4, 4, 3, 4, 2, 0, false},
    };
    const int maps = static_cast<int>(sizeof(specs) / sizeof(specs[0]));
    const std::string base = std::string(argv[1]) + ".map";
    std::string out = "{\n \"maps\": [";
    size_t total = 0;
    for (int i = 0; i < maps; ++i) {
        const Spec& s = specs[i];
        g_state = 0x51000001u + static_cast<uint32_t>(i) * 0x9e3779b9u;
        const std::string bytes = make_map(s);
        const std::string path = base + num(i);
        write_file(path, bytes);
        // A bridge or wall from the previous map would answer on this one.
        t.load(path);
        std::remove(path.c_str());

        std::vector<std::string> rows;
        rows.push_back(row_map(t));
        rows.push_back(row_extents(t));
        rows_lists(t, rows);

        // The pathfinder and partition over this map.
        TheAI->pathfinder()->reset();
        TheAI->pathfinder()->newMap();
        ThePartitionManager->reset();
        ThePartitionManager->init();

        if (i == 0 || i == 4) {
            BridgeInfo bridge;
            bridge.from.x = 20.0f;
            bridge.from.y = 40.0f;
            bridge.from.z = 60.0f;
            bridge.to.x = 120.0f;
            bridge.to.y = 40.0f;
            bridge.to.z = 95.0f;
            bridge.bridgeWidth = 30.0f;
            // The deck: a sloped rectangle from x 20 to 120 and y 25 to 55.
            bridge.fromLeft.x = 20.0f;
            bridge.fromLeft.y = 25.0f;
            bridge.fromLeft.z = 60.0f;
            bridge.fromRight.x = 20.0f;
            bridge.fromRight.y = 55.0f;
            bridge.fromRight.z = 60.0f;
            bridge.toLeft.x = 120.0f;
            bridge.toLeft.y = 25.0f;
            bridge.toLeft.z = 95.0f;
            bridge.toRight.x = 120.0f;
            bridge.toRight.y = 55.0f;
            bridge.toRight.z = 95.0f;
            Dict props;
            t.logic->TerrainLogic::addBridgeToLogic(&bridge, &props, "S1_NoSuchBridge");
        }
        TheAI->pathfinder()->*access_member(PfWallHeight()) = i % 2 ? 40.0f : 0.0f;

        query_rows(t, s, rows, 220);
        if (i != 5 && i != 6) {
            static Water water;
            water.victims.clear();
            water.spawn(w, s);
            ThePartitionManager->update();  // puts the victims in their cells
            water_rows(t, w, water, rows, 90);
            // A fresh grid of queries against the water where it ended up.
            for (int k = 0; k < 40; ++k) {
                const float x = between(0.0f, 300.0f), y = between(0.0f, 300.0f);
                rows.push_back(row_water(x, y));
            }
            for (Object* o : water.victims) TheGameLogic->destroyObject(o);
            TheGameLogic->processDestroyList();
        }
        shape_rows(t, w, s, rows, 60);
        // Queries over the reshaped ground.
        for (int k = 0; k < 60; ++k) {
            const float x = between(-40.0f, 400.0f), y = between(-40.0f, 350.0f);
            rows.push_back(row_height(t, x, y));
        }

        total += rows.size();
        out += std::string(i ? "," : "") + "\n  {\"wall\": \"" + num(i % 2) + "\", \"hex\": \"" +
               hex_of(bytes) + "\",\n   \"rows\": [";
        for (size_t r = 0; r < rows.size(); ++r) {
            out += std::string(r ? "," : "") + "\n    \"" + rows[r] + "\"";
        }
        out += "\n   ]}";
        t.logic->reset();
        std::fprintf(stderr, "map %d: %zu rows\n", i, rows.size());
    }
    out += "\n ]\n}\n";
    write_file(argv[1], out);
    std::fprintf(stderr, "%zu rows\n", total);
    return 0;
}
