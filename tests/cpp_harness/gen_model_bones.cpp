// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for the draw module bone queries logic makes, run on the
// unmodified reference. "spec" rows parse generated `W3DModelDraw` module data and
// dump the condition states; "best" rows run `findBestInfo`.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_model_bones.cpp -o /tmp/gen
//   (also with --opt O0 and with --opt O2 --sanitize)
//   /tmp/gen docs/golden_vectors/ModelBones.json
//
// Not covered: transition states (logic reads `findBestInfo`).

#include "oracle_wp51_world.h"

#include <algorithm>
#include <cstdio>
#include <string>
#include <vector>

#include "Common/INIException.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "GameLogic/Object.h"

#define private public
#define protected public
#include "../../vendor/GeneralsX/Core/GameEngineDevice/Include/W3DDevice/GameClient/Module/W3DModelDraw.h"
#include "../../vendor/GeneralsX/Core/GameEngineDevice/Include/W3DDevice/Common/W3DModuleFactory.h"
#undef private
#undef protected

static uint32_t g_state = 0x0b0e5a11u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
static bool chance(uint32_t percent) { return pick(100) < percent; }

static std::string hex_text(const std::string& s) {
    static const char* d = "0123456789abcdef";
    std::string out;
    for (unsigned char c : s) {
        out += d[c >> 4];
        out += d[c & 15];
    }
    return out;
}
static std::string word(uint32_t v) {
    char b[9];
    snprintf(b, sizeof b, "%08x", v);
    return b;
}
static std::string flt(float f) {
    uint32_t u;
    memcpy(&u, &f, 4);
    return word(u);
}

static std::vector<std::string> g_rows;

// ---- generated module data ---------------------------------------------------

static const char* const kStateBits[] = {"DAMAGED", "REALLYDAMAGED", "RUBBLE", "NIGHT", "SNOW", "MOVING",
    "ATTACKING", "DOOR_1_OPENING", "CAPTURED", "SOLD", "FIRING_A", "RELOADING_A"};
static const char* const kIgnoreBits[] = {"GARRISONED", "ENEMYNEAR", "WEAPONSET_PLAYER_UPGRADE",
    "CONSTRUCTION_COMPLETE"};
static const char* const kSlots[] = {"PRIMARY", "SECONDARY", "TERTIARY"};
static const char* const kWeaponFields[] = {"WeaponFireFXBone", "WeaponRecoilBone", "WeaponMuzzleFlash",
    "WeaponLaunchBone", "WeaponHideShowBone"};
static const char* const kNames[] = {"Muzzle", "FirePoint", "WeaponA", "TURRET", "TurretEL", "Launch", "fx",
    "NONE", "Recoil", "MuzzleFX", "Barrel"};

static std::string pick_name() { return kNames[pick(11)]; }

// A default state never says `Model = none`, which the states copying it would keep.
static std::string state_body(bool needs_model, bool force_once, bool is_default = false) {
    std::string out;
    if (needs_model || chance(50)) {
        const char* models[] = {"ModelA", "MODELB", "\"ModelC\"", "none"};
        out += std::string("      Model = ") + models[pick(is_default ? 3 : 4)] + "\n";
    }
    if (chance(20)) out += "      Turret = " + pick_name() + "\n";
    if (chance(10)) out += "      TurretPitch = " + pick_name() + "\n";
    if (chance(10)) out += "      AltTurret = " + pick_name() + "\n";
    if (chance(5)) out += "      AltTurretPitch = " + pick_name() + "\n";
    if (chance(15)) out += "      TurretArtAngle = " + std::to_string((int)pick(721) - 360) + "\n";
    if (chance(15)) out += "      TurretArtPitch = " + std::to_string((int)pick(91) - 45) + ".5\n";
    if (chance(8)) out += "      AltTurretArtAngle = " + std::to_string((int)pick(721) - 360) + "\n";
    if (chance(8)) out += "      AltTurretArtPitch = " + std::to_string((int)pick(91) - 45) + "\n";
    for (int i = pick(4); i > 0; --i) {
        out += std::string("      ") + kWeaponFields[pick(5)] + " = " + kSlots[pick(3)];
        if (chance(90)) out += " " + pick_name();
        out += "\n";
    }
    for (int i = chance(45) ? 1 + pick(2) : 0; i > 0; --i) {
        const char* anims[] = {"SK.AnimA", "SK.ANIMB", "NONE", "sk.animc"};
        out += std::string("      Animation = ") + anims[pick(4)];
        if (chance(30)) out += " 0.0 " + std::to_string(pick(3));
        out += "\n";
    }
    if (chance(20)) {
        const char* flags[] = {"PRISTINE_BONE_POS_IN_FINAL_FRAME", "NONE", "RANDOMSTART",
            "START_FRAME_LAST PRISTINE_BONE_POS_IN_FINAL_FRAME", "+PRISTINE_BONE_POS_IN_FINAL_FRAME",
            "-PRISTINE_BONE_POS_IN_FINAL_FRAME"};
        out += std::string("      Flags = ") + flags[pick(6)] + "\n";
    }
    if (force_once) out += "      AnimationMode = ONCE\n";
    return out;
}

// A set of state bits not used yet, as text; "NONE" for the empty set.
static std::string fresh_conditions(std::vector<std::string>& used, bool allow_none) {
    for (int attempt = 0; attempt < 100; ++attempt) {
        std::vector<int> bits;
        for (int i = pick(4); i > 0; --i) {
            const int b = pick(12);
            if (std::find(bits.begin(), bits.end(), b) == bits.end()) bits.push_back(b);
        }
        if (bits.empty() && !allow_none) continue;
        std::vector<int> sorted = bits;
        std::sort(sorted.begin(), sorted.end());
        std::string key;
        for (int b : sorted) key += std::to_string(b) + ",";
        if (std::find(used.begin(), used.end(), key) != used.end()) continue;
        used.push_back(key);
        std::string out;
        for (int b : bits) out += std::string(out.empty() ? "" : " ") + kStateBits[b];
        return out.empty() ? "NONE" : out;
    }
    return "";
}

static std::string spec_ini(const std::string& name) {
    std::string d = "  Draw = W3DModelDraw ModuleTag_01\n";
    if (chance(30)) d += "    ExtraPublicBone = " + pick_name() + " " + pick_name() + "\n";
    if (chance(40)) {
        d += "    IgnoreConditionStates =";
        for (int i = 1 + pick(3); i > 0; --i) d += std::string(" ") + kIgnoreBits[pick(4)];
        d += "\n";
    }
    if (chance(15)) d += "    AttachToBoneInAnotherModule = " + pick_name() + "01\n";
    std::vector<std::string> used;
    const bool has_default = chance(60);
    if (has_default) {
        d += "    DefaultConditionState\n" + state_body(true, false, true) + "    End\n";
        used.push_back("");
    } else {
        d += "    ConditionState = NONE\n" + state_body(true, false, true) + "    End\n";
        used.push_back("");
    }
    for (int s = pick(9); s > 0; --s) {
        const int what = pick(10);
        const std::string c = fresh_conditions(used, false);
        if (c.empty()) break;
        if (what == 0) {
            d += "    AliasConditionState = " + c + "\n";
        } else if (what == 1) {
            d += "    TransitionState = TransA TransB" + std::to_string(s) + "\n"
                + state_body(!has_default, true) + "    End\n";
        } else {
            d += "    ConditionState = " + c + "\n" + state_body(!has_default, false) + "    End\n";
        }
    }
    if (chance(20)) d += "    ExtraPublicBone = " + pick_name() + "\n";
    d += "    OkToChangeModelColor = Yes\n";
    d += "  End\n";
    return "Object " + name + "\n" + d + "End\n";
}

// ---- dumps -------------------------------------------------------------------

static std::string bits_text(const ModelConditionFlags& f) {
    std::string out;
    for (int i = 0; i < f.size(); ++i)
        if (f.test(i)) out += std::string(out.empty() ? "" : ".") + std::to_string(i);
    return out;
}
static std::string key_text(NameKeyType key) {
    return key == NAMEKEY_INVALID ? "" : TheNameKeyGenerator->keyToName(key).str();
}
static std::string list_text(const std::vector<AsciiString>& v) {
    std::string out;
    for (const AsciiString& s : v) out += std::string(s.str()) + ",";
    return out;
}

static const W3DModelDrawModuleData* draw_data(const ThingTemplate* t, int i) {
    return static_cast<const W3DModelDrawModuleData*>(t->getDrawModuleInfo().getNthData(i));
}

static std::string dump_spec(const W3DModelDrawModuleData* d) {
    std::string out = "extra=" + list_text(d->m_extraPublicBones);
    out += ";ignore=" + bits_text(d->m_ignoreConditionStates);
    out += ";default=" + std::to_string(d->m_defaultState);
    out += std::string(";attach=") + d->m_attachToDrawableBone.str();
    for (const ModelConditionInfo& s : d->m_conditionStates) {
        out += "#c=";
        for (const ModelConditionFlags& f : s.m_conditionsYesVec) out += bits_text(f) + "|";
        out += std::string(";m=") + s.m_modelName.str();
        out += std::string(";a=") + (s.m_animations.empty() ? "" : s.m_animations.front().getName().str());
        out += ";f=" + std::to_string((s.m_flags >> 4) & 1);
        out += ";p=" + list_text(s.m_publicBones);
        out += ";w=";
        for (int w = 0; w < WEAPONSLOT_COUNT; ++w) {
            out += std::string(s.m_weaponFireFXBoneName[w].str()) + ":" + s.m_weaponRecoilBoneName[w].str()
                + ":" + s.m_weaponMuzzleFlashName[w].str() + ":" + s.m_weaponProjectileLaunchBoneName[w].str()
                + "/";
        }
        out += ";t=";
        for (int t = 0; t < MAX_TURRETS; ++t) {
            const ModelConditionInfo::TurretInfo& tur = s.m_turrets[t];
            out += key_text(tur.m_turretAngleNameKey) + ":" + key_text(tur.m_turretPitchNameKey) + ":"
                + flt(tur.m_turretArtAngle) + ":" + flt(tur.m_turretArtPitch) + "/";
        }
    }
    return out;
}

static void spec_cases(oracle_wp51::World& w) {
    for (int i = 0; i < 80; ++i) {
        const std::string name = "SpecT" + std::to_string(i);
        const std::string ini = spec_ini(name);
        try {
            w.load_ini(ini);
        } catch (const INIException& e) {
            fprintf(stderr, "%s\n%s", e.mFailureMessage ? e.mFailureMessage : "INI error", ini.c_str());
            exit(4);
        }
        const ThingTemplate* t = TheThingFactory->findTemplate(name.c_str());
        if (!t || t->getDrawModuleInfo().getCount() != 1) exit(3);
        const W3DModelDrawModuleData* d = draw_data(t, 0);
        g_rows.push_back("{\"case\":\"spec\",\"name\":\"" + name + "\",\"ini\":\"" + hex_text(ini)
            + "\",\"dump\":\"" + dump_spec(d) + "\"}");

        std::string queries;
        for (int q = 0; q < 48; ++q) {
            ModelConditionFlags f;
            // Mostly the states' own bits, plus ignored and unrelated ones.
            const int count = pick(6);
            for (int k = 0; k < count; ++k) {
                const int kind = pick(10);
                const char* bit = kind < 7 ? kStateBits[pick(12)] : kIgnoreBits[pick(4)];
                f.set(ModelConditionFlags::getSingleBitFromName(bit));
            }
            if (chance(10)) f.set(pick(f.size()));
            const ModelConditionInfo* best = d->findBestInfo(f);
            const int index = best ? (int)(best - &d->m_conditionStates[0]) : -1;
            queries += bits_text(f) + "=" + std::to_string(index) + ";";
        }
        g_rows.push_back("{\"case\":\"best\",\"name\":\"" + name + "\",\"queries\":\"" + queries + "\"}");
    }
}

int main(int argc, char** argv) {
    if (argc < 2) return 2;
    oracle_wp51::World w;
    TheModuleFactory = NEW W3DModuleFactory;
    TheModuleFactory->init();
    spec_cases(w);

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* f = fopen(tmp.c_str(), "wb");
    if (!f) return 2;
    fprintf(f, "{\"vectors\":[\n");
    for (size_t i = 0; i < g_rows.size(); ++i)
        fprintf(f, "%s%s\n", g_rows[i].c_str(), i + 1 < g_rows.size() ? "," : "");
    fprintf(f, "]}\n");
    fclose(f);
    if (rename(tmp.c_str(), argv[1]) != 0) return 2;
    fflush(stdout);
    _exit(0);  // engine singletons are not torn down
}
