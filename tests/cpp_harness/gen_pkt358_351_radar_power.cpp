// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for RadarUpdate (PKT-358) and PowerPlantUpdate (PKT-351),
// run on the unmodified reference modules in a small real world.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt358_351_radar_power.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT358_351_RadarPower.json
//
// Not covered: objects without a drawable, frames above 0x3fffffff (the scheduler
// clamps there), loading a save, and the upgrade modules that call these two.

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/Team.h"
#include "GameClient/Drawable.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/PowerPlantUpdate.h"
#include "GameLogic/Module/RadarUpdate.h"
#include "GameLogic/Object.h"

#include <cstdio>
#include <sstream>
#include <string>
#include <vector>

static const int kVariants = 7;
// Milliseconds, as written in INI. nullptr leaves the field out, giving the default.
static const char* const kTimes[kVariants] = {"0", "1", "100", "599", "1000", "4000", nullptr};

static uint32_t g_state = 0x35800351u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}

template <typename T>
static T* find_behavior(Object* obj) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (T* typed = dynamic_cast<T*>(*m)) return typed;
    }
    return nullptr;
}

static std::string fixture_ini() {
    std::string s;
    for (int i = 0; i < kVariants; ++i) {
        const std::string n = std::to_string(i);
        s += "Object P358_Radar" + n + "\n  KindOf = PRELOAD STRUCTURE\n"
             "  Body = InactiveBody ModuleTag_Body\n  End\n"
             "  Behavior = RadarUpdate ModuleTag_Radar\n" +
             (kTimes[i] ? "    RadarExtendTime = " + std::string(kTimes[i]) + "\n" : std::string()) +
             "  End\nEnd\n";
        s += "Object P351_Power" + n + "\n  KindOf = PRELOAD STRUCTURE\n"
             "  Body = InactiveBody ModuleTag_Body\n  End\n"
             "  Behavior = PowerPlantUpdate ModuleTag_Rods\n" +
             (kTimes[i] ? "    RodsExtendTime = " + std::string(kTimes[i]) + "\n" : std::string()) +
             "  End\nEnd\n";
    }
    return s;
}

// The harness's single-module scheduler step.
static void step(oracle_wp51::World& w, UpdateModule* u, UnsignedInt frame) {
    w.set_frame(frame);
    if (u->friend_getNextCallFrame() > frame) return;
    UpdateSleepTime sleep_len = u->update();
    if (sleep_len < 1) sleep_len = UPDATE_SLEEP_NONE;
    UnsignedInt next = frame + static_cast<UnsignedInt>(sleep_len);
    if (next > UPDATE_SLEEP_FOREVER) next = UPDATE_SLEEP_FOREVER;
    u->friend_setNextCallFrame(next);
}

// "<save hex> <crc> <three or two model condition bits> <update return> <next call frame>"
static std::string row(int kind, Object* o, UpdateModule* m, uint32_t ret) {
    const std::string save = oracle_wp51::World::snapshot_save_hex(m);
    const uint32_t crc = oracle_wp51::World::snapshot_crc(m);
    const Drawable* d = o->getDrawable();
    if (!d) {
        std::fprintf(stderr, "object without a drawable\n");
        std::exit(1);
    }
    const ModelConditionFlags& f = d->getModelConditionFlags();
    const int a = f.test(kind ? MODELCONDITION_POWER_PLANT_UPGRADING : MODELCONDITION_RADAR_EXTENDING);
    const int b = f.test(kind ? MODELCONDITION_POWER_PLANT_UPGRADED : MODELCONDITION_RADAR_UPGRADED);
    std::ostringstream out;
    out << save << ' ' << crc << ' ' << a << b << ' ' << ret << ' ' << m->friend_getNextCallFrame();
    return out.str();
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_pkt358_351_radar_power <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    w.load_ini(fixture_ini());
    w.init_with_neutral_player();

    FILE* out = std::fopen(argv[1], "w");
    if (!out) return 1;
    const int kTotal = 1000;
    std::fprintf(out, "{\"vector_count\":%d,\"vectors\":[", kTotal);
    for (int i = 0; i < kTotal; ++i) {
        const int kind = i % 2;  // 0 radar, 1 power plant
        const uint32_t variant = pick(kVariants);
        // A quarter start above 2^24, where `frame + RadarExtendTime` rounds.
        UnsignedInt frame = pick(4) == 0 ? 0x01000000u + pick(0x3e000000u) : 1 + pick(5000);
        w.set_frame(frame);
        const std::string name = std::string(kind ? "P351_Power" : "P358_Radar") + std::to_string(variant);
        Object* o = w.spawn_object(name.c_str(), w.team_a);
        RadarUpdate* radar = kind ? nullptr : find_behavior<RadarUpdate>(o);
        PowerPlantUpdate* power = kind ? find_behavior<PowerPlantUpdate>(o) : nullptr;
        UpdateModule* m = kind ? static_cast<UpdateModule*>(power) : static_cast<UpdateModule*>(radar);
        if (!m) {
            std::fprintf(stderr, "missing module on %s\n", name.c_str());
            return 1;
        }

        std::fprintf(out, "%s\n{\"kind\":%d,\"variant\":%u,\"frame\":%u,\"spawn\":\"%s\",\"ops\":[",
                     i ? "," : "", kind, variant, frame, row(kind, o, m, 0).c_str());
        std::vector<std::string> rows;
        const int ops = 3 + static_cast<int>(pick(10));
        for (int k = 0; k < ops; ++k) {
            // 0 extend, 1 retract (power) or extend again, 2 scheduled frames, 3 direct update
            uint32_t code = pick(8);
            if (code > 3) code = 2;
            if (k == 0 && pick(3) != 0) code = 0;
            const uint32_t a = code == 2 ? 1 + pick(pick(2) ? 8 : 150) : 0;
            uint32_t ret = 0;
            w.set_frame(frame);
            if (code == 0 || (code == 1 && !kind)) {
                if (kind) power->extendRods(TRUE);
                else radar->extendRadar();
            } else if (code == 1) {
                power->extendRods(FALSE);
            } else if (code == 2) {
                for (uint32_t t = 0; t < a; ++t) step(w, m, ++frame);
            } else {
                ret = static_cast<uint32_t>(m->update());
            }
            std::fprintf(out, "%s[%u,%u]", k ? "," : "", code, a);
            rows.push_back(row(kind, o, m, ret));
        }
        std::fprintf(out, "],\"rows\":[");
        for (size_t r = 0; r < rows.size(); ++r)
            std::fprintf(out, "%s\"%s\"", r ? "," : "", rows[r].c_str());
        std::fprintf(out, "]}");

        TheGameLogic->destroyObject(o);
        w.flush_destroy_list();
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return 0;
}
