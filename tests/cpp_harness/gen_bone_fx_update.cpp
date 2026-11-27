// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for `BoneFXUpdate` and `BoneFXDamage`, run on the unmodified
// reference: update frames, damage and healing across body states, and
// `stopAllBoneFX`. Each row holds the effect frames of the current body state,
// both random streams, the objects its creation lists made and the CRC of the
// module's saved bytes.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_bone_fx_update.cpp -o /tmp/gen
//   (also with --opt O0 and with --opt O2 --sanitize)
//   /tmp/gen docs/golden_vectors/BoneFXUpdate.json
//
// Not covered: bone positions (headless objects here have no drawable), FX lists
// and particle systems (client).

#include "oracle_wp51_world.h"

#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameClient/ClientRandomValue.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/BoneFXUpdate.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"

#include <cstdio>
#include <sstream>
#include <string>
#include <vector>

static uint32_t g_state = 0xb0ef0123u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}

static const char* const kStates[4] = {"Pristine", "Damaged", "ReallyDamaged", "Rubble"};
static const int kVariants = 4;

static std::string asset_ini() {
    return "FXList FX_B\nEnd\n"
           "ObjectCreationList OCL_B\n  CreateObject\n    ObjectNames = G_Mark\n    Disposition = LIKE_EXISTING\n  End\nEnd\n";
}

static std::string effect(int state, const char* kind, int slot, bool once, int low, int high) {
    std::string label = std::string(kind) == "FXList" ? "fxlist"
                        : std::string(kind) == "OCL"  ? "ocl"
                                                      : "psys";
    const char* name = std::string(kind) == "FXList" ? "FX_B"
                       : std::string(kind) == "OCL"  ? "OCL_B"
                                                     : "None";
    return std::string("    ") + kStates[state] + kind + std::to_string(slot) + " = bone:B" +
           std::to_string(slot) + " onlyonce:" + (once ? "Yes" : "No") + " " + std::to_string(low) +
           " " + std::to_string(high) + " " + label + ":" + name + "\n";
}

static std::string objects_ini() {
    std::string s =
        "Object G_Mark\n  KindOf = INERT\n  Body = InactiveBody ModuleTag_Body\n  End\nEnd\n";
    static const char* const kKinds[3] = {"FXList", "OCL", "ParticleSystem"};
    for (int v = 0; v < kVariants; ++v) {
        s += "Object G_Bone" + std::to_string(v) +
             "\n  KindOf = STRUCTURE\n  Body = ActiveBody ModuleTag_Body\n"
             "    MaxHealth = 1000.0\n    InitialHealth = 1000.0\n  End\n"
             "  Behavior = BoneFXDamage ModuleTag_Damage\n  End\n"
             "  Behavior = BoneFXUpdate ModuleTag_Fx\n";
        for (int state = 0; state < 4; ++state) {
            for (int kind = 0; kind < 3; ++kind) {
                for (int slot = 1; slot <= 8; ++slot) {
                    if (pick(v == 0 ? 3 : 9) != 0) continue;
                    // Milliseconds; some ranges are empty or reversed.
                    const int low = static_cast<int>(pick(5)) * 100;
                    const int high = pick(4) == 0 ? low : static_cast<int>(pick(12)) * 100;
                    s += effect(state, kKinds[kind], slot, pick(3) == 0, low, high);
                }
            }
        }
        if (v == 2) {
            s += "    DamageOCLTypes = NONE +FLAME\n    DamageFXTypes = ALL -CRUSH\n";
        }
        if (v == 3) {
            s += "    DamageOCLTypes = ALL -EXPLOSION\n";
        }
        s += "  End\nEnd\n";
    }
    return s;
}

static std::string escaped(const std::string& text) {
    std::string out;
    for (char c : text) {
        if (c == '\n') out += "\\n";
        else out += c;
    }
    return out;
}

static int count_marks() {
    const ThingTemplate* mark = TheThingFactory->findTemplate("G_Mark");
    int n = 0;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        if (o->getTemplate() == mark && !o->isDestroyed()) ++n;
    }
    return n;
}

static std::string state_text(BehaviorModule* module) {
    uint32_t crc = 0;
    const std::string hex = oracle_wp51::World::snapshot_save_hex(module, &crc);
    // The saved bytes end with: the arrays, the state, four flags, active.
    const size_t arrays = 3 * 4 * 8 * 4 + 3 * 4 * 8 * 12 + 4 + 4 + 1;
    const size_t start = hex.size() / 2 - arrays;
    auto word = [&](size_t at) {
        uint32_t v = 0;
        for (int i = 3; i >= 0; --i) {
            v = v * 256 + std::stoul(hex.substr((at + i) * 2, 2), nullptr, 16);
        }
        return static_cast<int32_t>(v);
    };
    const size_t tail = start + 3 * 4 * 8 * 4 + 3 * 4 * 8 * 12;
    const int body = word(tail);
    std::ostringstream s;
    s << body << " " << std::stoul(hex.substr((tail + 8) * 2, 2), nullptr, 16) << " ";
    for (int kind = 0; kind < 3; ++kind) {
        for (int slot = 0; slot < 8; ++slot) {
            s << word(start + (kind * 32 + body * 8 + slot) * 4) << ",";
        }
    }
    s << " " << crc << " " << (hex.size() / 2);
    return s.str();
}

int main(int argc, char** argv) {
    if (argc < 2) return 2;
    oracle_wp51::World w;
    w.load_ini(asset_ini());
    const std::string objects = objects_ini();
    w.load_ini(objects);
    TheThingFactory->postProcessLoad();
    w.init_with_neutral_player();

    std::vector<std::string> rows;
    rows.push_back("assets " + escaped(asset_ini()));
    rows.push_back("objects " + escaped(objects));
    const NameKeyType key = NAMEKEY("BoneFXUpdate");
    for (int trial = 0; trial < 60; ++trial) {
        const int variant = trial % kVariants;
        const uint32_t seed = 1 + pick(0x7fffffffu);
        UnsignedInt frame = 5 + pick(1000);
        w.set_frame(frame);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        InitRandom(seed);
        const std::string name = "G_Bone" + std::to_string(variant);
        Object* o = TheThingFactory->newObject(TheThingFactory->findTemplate(name.c_str()), w.team_a);
        if (!o) return 1;
        UpdateModule* update = o->findUpdateModule(key);
        if (!update) return 1;
        BoneFXUpdate* fx = static_cast<BoneFXUpdate*>(update);
        {
            std::ostringstream row;
            row << "trial " << variant << " " << seed << " " << frame << " " << state_text(update);
            rows.push_back(row.str());
        }
        for (int k = 0; k < 24; ++k) {
            int code = static_cast<int>(pick(10));
            code = code < 5 ? 0 : code < 8 ? 1 : code - 6;
            int amount = 100 + static_cast<int>(pick(500));
            const int types[4] = {DAMAGE_UNRESISTABLE, DAMAGE_FLAME, DAMAGE_CRUSH, DAMAGE_EXPLOSION};
            const int type = types[pick(4)];
            frame += pick(3) == 0 ? 1 + pick(30) : 1;
            w.set_frame(frame);
            if (code == 1 && o->getBodyModule()->getHealth() - amount < 5.0f) code = 2;
            if (code == 0) {
                static_cast<UpdateModuleInterface*>(update)->update();
            } else if (code == 1 || code == 2) {
                DamageInfo info;
                info.in.m_damageType = static_cast<DamageType>(code == 1 ? type : DAMAGE_HEALING);
                info.in.m_amount = static_cast<Real>(amount);
                if (code == 1) o->attemptDamage(&info);
                else o->attemptHealing(static_cast<Real>(amount), nullptr);
            } else {
                fx->stopAllBoneFX();
            }
            std::ostringstream row;
            row << "op " << code << " " << type << " " << amount << " " << frame << " "
                << state_text(update) << " " << count_marks() << " " << GetGameLogicRandomSeedCRC()
                << " " << GameClientRandomValue(0, 1000000);
            rows.push_back(row.str());
        }
        std::vector<Object*> doomed;
        for (Object* it = TheGameLogic->getFirstObject(); it; it = it->getNextObject()) {
            doomed.push_back(it);
        }
        for (Object* it : doomed) TheGameLogic->destroyObject(it);
        TheGameLogic->processDestroyList();
    }

    FILE* out = std::fopen(argv[1], "w");
    if (!out) return 1;
    std::fputs("[\n", out);
    for (size_t i = 0; i < rows.size(); ++i) {
        std::fprintf(out, " \"%s\"%s\n", rows[i].c_str(), i + 1 < rows.size() ? "," : "");
    }
    std::fputs("]\n", out);
    std::fclose(out);
    std::printf("%zu rows\n", rows.size());
    return 0;
}
