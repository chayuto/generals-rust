// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for the edges of `BaseHeightMapRenderObjClass::isClearLineOfSight`
// and `isCliffCell`, run on the unmodified reference over maps built here: flat,
// stepped, ramped, cliffed and rough, with and without a border, down to 2 by 2
// samples. Lines and points are aimed at cell boundaries, the border, both sides of
// the map, lines of one cell, the fudge of 0.5 and the maximum height.
// `gen_s1_terrain.cpp` covers the same calls over loaded maps with random queries.
//
//   CPATH=vendor/GeneralsX/GeneralsMD/Code/GameEngineDevice/Include:vendor/GeneralsX/Core/GameEngineDevice/Include \
//     python3 scripts/build_oracle.py tests/cpp_harness/gen_terrain_los.cpp -o /tmp/gen
//   (also with --opt O0 and with --opt O2 --sanitize)
//   /tmp/gen docs/golden_vectors/TerrainLos.json
//
// The height map and the render object are zeroed memory with the members the two
// calls read; the maximum height is scanned as `initHeightData` scans it.

// The world header is only included for the globals the libraries need to link.
#include "oracle_wp51_world.h"

#define private public
#define protected public
#include "W3DDevice/GameClient/BaseHeightMap.h"
#include "W3DDevice/GameClient/WorldHeightMap.h"
#undef private
#undef protected

#include "Common/MapObject.h"

#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <string>
#include <vector>

static uint32_t g_state = 0x7E551A1Eu;
static uint32_t pick(uint32_t n) { return oracle::splitmix32(g_state) % n; }
static bool chance(uint32_t percent) { return pick(100) < percent; }
static float between(float lo, float hi) {
    return lo + (hi - lo) * (static_cast<float>(pick(1000001)) / 1000000.0f);
}

static std::string hex8(float v) {
    char buf[9];
    snprintf(buf, sizeof buf, "%08x", oracle::fbits(v));
    return buf;
}

static std::vector<std::string> g_rows;

struct Map {
    WorldHeightMap* map;
    BaseHeightMapRenderObjClass* render;
    int w, h, border;
};

// kind: 0 flat, 1 stairs, 2 ramp, 3 cliffs, 4 rough.
static Map make_map(int index, int w, int h, int border, int kind) {
    Map m;
    m.w = w;
    m.h = h;
    m.border = border;
    m.map = static_cast<WorldHeightMap*>(calloc(1, sizeof(WorldHeightMap)));
    m.map->m_width = w;
    m.map->m_height = h;
    m.map->m_borderSize = border;
    m.map->m_dataSize = w * h;
    m.map->m_data = static_cast<UnsignedByte*>(calloc(1, w * h));
    m.map->m_flipStateWidth = (w + 7) / 8;
    m.map->m_cellCliffState = static_cast<UnsignedByte*>(calloc(1, m.map->m_flipStateWidth * h));
    const int level = 8 + pick(40);
    const int slope = 1 + pick(14);
    for (int y = 0; y < h; ++y) {
        for (int x = 0; x < w; ++x) {
            int v = level;
            if (kind == 1) v = level + ((x / 3 + y / 4) % 5) * 16;
            if (kind == 2) v = level + x * slope + y * (slope / 3);
            // 15 samples are a slope of 9.375, 16 are 10: either side of the cliff limit.
            if (kind == 3) v = level + (chance(20) ? 15 + static_cast<int>(pick(2)) : 0) + (x > w / 2 ? 90 : 0);
            if (kind == 4) v = static_cast<int>(pick(chance(50) ? 256 : 24));
            m.map->m_data[y * w + x] = static_cast<UnsignedByte>(v > 255 ? 255 : v);
        }
    }
    m.map->initCliffFlagsFromHeights();

    m.render = static_cast<BaseHeightMapRenderObjClass*>(calloc(1, 1 << 16));
    m.render->m_map = m.map;
    int low = 255, high = 0;
    for (int j = 0; j < h; ++j) {
        for (int i = 0; i < w; ++i) {
            const int cur = m.map->getHeight(i, j);
            if (cur < low) low = cur;
            if (high < cur) high = cur;
        }
    }
    m.render->m_maxHeight = high * MAP_HEIGHT_SCALE;

    static const char* const digits = "0123456789abcdef";
    std::string data, cliffs;
    for (int i = 0; i < w * h; ++i) {
        data += digits[m.map->m_data[i] >> 4];
        data += digits[m.map->m_data[i] & 15];
    }
    for (int y = 0; y < h; ++y)
        for (int x = 0; x < w; ++x) cliffs += m.map->getCliffState(x, y) ? '1' : '0';
    g_rows.push_back("{\"case\":\"map\",\"id\":\"" + std::to_string(index) + "\",\"size\":\""
        + std::to_string(w) + " " + std::to_string(h) + " " + std::to_string(border) + "\",\"data\":\""
        + data + "\",\"cliffs\":\"" + cliffs + "\",\"max\":\"" + hex8(m.render->m_maxHeight) + "\"}");
    return m;
}

// A coordinate along one axis of `extent` samples: mostly near the map, often on or
// next to a cell boundary, sometimes far outside.
static float axis(const Map& m, int extent) {
    const int cell = static_cast<int>(pick(extent + 6)) - 3 - m.border;
    const float edge = cell * 10.0f;
    switch (pick(8)) {
        case 0: return edge;
        case 1: return std::nextafterf(edge, -1.0e9f);
        case 2: return std::nextafterf(edge, 1.0e9f);
        case 3: return -between(0.0f, 10.0f);
        case 4: return chance(50) ? between(-1.0e6f, 1.0e6f) : edge + 5.0f;
        default: return edge + between(0.0f, 10.0f);
    }
}

static void cliff_row(int index, const Map& m, float x, float y) {
    g_rows.push_back("{\"case\":\"cliff\",\"map\":\"" + std::to_string(index) + "\",\"at\":\"" + hex8(x) + hex8(y)
        + "\",\"out\":\"" + (m.render->isCliffCell(x, y) ? "1" : "0") + "\"}");
}

static void los_row(int index, const Map& m, const Coord3D& a, const Coord3D& b) {
    g_rows.push_back("{\"case\":\"los\",\"map\":\"" + std::to_string(index) + "\",\"line\":\"" + hex8(a.x)
        + hex8(a.y) + hex8(a.z) + hex8(b.x) + hex8(b.y) + hex8(b.z) + "\",\"out\":\""
        + (m.render->isClearLineOfSight(a, b) ? "1" : "0") + "\"}");
}

// The highest corner of the cell holding (x, y), clamped into the map, in world units.
static float cell_top(const Map& m, float x, float y) { return m.render->getMaxCellHeight(x, y); }

static void queries(int index, const Map& m) {
    for (int i = 0; i < 150; ++i) {
        const float x = axis(m, m.w), y = axis(m, m.h);
        cliff_row(index, m, x, y);
    }
    const float max = m.render->m_maxHeight;
    for (int i = 0; i < 260; ++i) {
        Coord3D a = {axis(m, m.w), axis(m, m.h), 0}, b = {axis(m, m.w), axis(m, m.h), 0};
        // Keep the walk short enough to be cheap.
        if (std::fabs(a.x) > 5000.0f && std::fabs(b.x) < 5000.0f) b.x = a.x + between(-50.0f, 50.0f);
        if (std::fabs(b.x) > 5000.0f) a.x = b.x + between(-50.0f, 50.0f);
        if (std::fabs(a.y) > 5000.0f && std::fabs(b.y) < 5000.0f) b.y = a.y + between(-50.0f, 50.0f);
        if (std::fabs(b.y) > 5000.0f) a.y = b.y + between(-50.0f, 50.0f);
        const float top = cell_top(m, a.x, a.y);
        switch (pick(8)) {
            case 0:  // one cell, or none at all
                b.x = a.x + (chance(50) ? 0.0f : between(-10.0f, 10.0f));
                b.y = a.y + (chance(50) ? 0.0f : between(-10.0f, 10.0f));
                a.z = top + between(-2.0f, 2.0f);
                b.z = between(0.0f, 200.0f);
                break;
            case 1: {  // on the fudge, level with the first cell
                const float z = top - 0.5f;
                a.z = chance(50) ? z : std::nextafterf(z, chance(50) ? -1.0e9f : 1.0e9f);
                b.z = chance(60) ? a.z : a.z + between(-30.0f, 30.0f);
                break;
            }
            case 2:  // at or around the maximum height, level or rising
                a.z = chance(60) ? max : max + between(-1.0f, 1.0f);
                b.z = chance(30) ? a.z : a.z + between(0.0f, 40.0f);
                break;
            case 3:  // rising from below the maximum through it
                a.z = max - between(0.0f, 6.0f);
                b.z = max + between(0.0f, 60.0f);
                break;
            case 4:  // falling
                a.z = max + between(-5.0f, 40.0f);
                b.z = between(-20.0f, 40.0f);
                break;
            case 5:  // straight along one axis, or a diagonal
                if (chance(33)) b.x = a.x;
                else if (chance(50)) b.y = a.y;
                else b.y = a.y + (b.x - a.x);
                a.z = top + between(0.0f, 25.0f);
                b.z = cell_top(m, b.x, b.y) + between(0.0f, 25.0f);
                break;
            default:
                a.z = top + between(-3.0f, 40.0f);
                b.z = cell_top(m, b.x, b.y) + between(-3.0f, 40.0f);
                break;
        }
        los_row(index, m, a, b);
    }
}

int main(int argc, char** argv) {
    if (argc < 2) return 2;
    oracle::init_memory();
    struct Shape { int w, h, border, kind; };
    const Shape shapes[] = {
        {24, 20, 0, 0}, {24, 20, 3, 0}, {31, 27, 4, 1}, {17, 33, 0, 1}, {29, 22, 5, 2},
        {26, 26, 1, 2}, {33, 25, 3, 3}, {20, 20, 0, 3}, {23, 31, 7, 4}, {40, 36, 2, 4},
        {2, 2, 0, 4}, {9, 3, 1, 3},
    };
    int index = 0;
    for (const Shape& s : shapes) {
        const Map m = make_map(index, s.w, s.h, s.border, s.kind);
        queries(index, m);
        ++index;
    }
    std::string out = "{\"vectors\":[\n";
    for (size_t i = 0; i < g_rows.size(); ++i) out += g_rows[i] + (i + 1 < g_rows.size() ? ",\n" : "\n");
    out += "]}\n";
    oracle::write_file_atomic(argv[1], out);
    return 0;
}
