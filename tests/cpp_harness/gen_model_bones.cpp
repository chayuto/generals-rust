// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for the draw module bone queries logic makes, run on the
// unmodified reference. "spec" rows parse generated `W3DModelDraw` module data and
// dump the condition states; "best" rows run `findBestInfo`. The other rows make
// objects with real drawables over W3D files built here and ask what logic asks:
// `Drawable::getPristineBonePositions`, `getBarrelCount`,
// `getProjectileLaunchOffset`, `Weapon::calcProjectileLaunchPosition` and
// `DockUpdate::loadDockPositions`.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_model_bones.cpp -o /tmp/gen
//   (also with --opt O0 and with --opt O2 --sanitize)
//   /tmp/gen docs/golden_vectors/ModelBones.json
//
// Not covered: transition states (logic reads `findBestInfo`), contained launchers,
// compressed animations and aggregates (see `gen_ast_assets.cpp`).

#include "oracle_common.h"

#define private public
#define protected public
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/DockUpdate.h"
#include "GameLogic/TurretAI.h"
#undef private
#undef protected

#include "oracle_wp51_world.h"

#include <algorithm>
#include <map>
#include <cstdio>
#include <string>
#include <vector>

#include "Common/INIException.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "GameClient/Drawable.h"
#include "GameLogic/Object.h"
#include "GameLogic/Weapon.h"
#include "WWLib/ffactory.h"
#include "WWLib/ramfile.h"
#include "WWMath/wwmath.h"

#include "WWLib/hashtemplate.h"
#include "WWLib/wwstring.h"

struct SDL_Window;

#define private public
#define protected public
#include "../../vendor/GeneralsX/Core/GameEngineDevice/Include/W3DDevice/GameClient/Module/W3DModelDraw.h"
#include "../../vendor/GeneralsX/Core/GameEngineDevice/Include/W3DDevice/Common/W3DModuleFactory.h"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngineDevice/Include/W3DDevice/GameClient/W3DAssetManager.h"
#undef private
#undef protected

// The one member of `W3DDisplay` used here; its header does not stand alone.
class W3DDisplay {
public:
    static W3DAssetManager* m_assetManager;
};

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

// ---- generated models --------------------------------------------------------

typedef std::vector<uint8_t> Bytes;
static std::map<std::string, Bytes> g_files;   // lower-case name -> content

static std::string lower(std::string s) {
    for (char& c : s) c = (char)tolower((unsigned char)c);
    return s;
}

struct MemFactory : FileFactoryClass {
    FileClass* Get_File(const char* name) override {
        auto it = g_files.find(lower(name));
        if (it == g_files.end()) return new RAMFileClass((void*)nullptr, 0);
        return new RAMFileClass(it->second.data(), (int)it->second.size());
    }
    void Return_File(FileClass* f) override { delete f; }
};

struct W {
    Bytes b;
    W& u8(uint32_t v) { b.push_back((uint8_t)v); return *this; }
    W& u16(uint32_t v) { u8(v); return u8(v >> 8); }
    W& u32(uint32_t v) { u16(v); return u16(v >> 16); }
    W& f(float v) { uint32_t u; memcpy(&u, &v, 4); return u32(u); }
    W& name(const std::string& s, size_t w) { for (size_t i = 0; i < w; ++i) u8(i < s.size() ? s[i] : 0); return *this; }
    W& raw(const Bytes& x) { b.insert(b.end(), x.begin(), x.end()); return *this; }
    W& chunk(uint32_t id, const W& body, bool sub = false) {
        u32(id); u32((uint32_t)body.b.size() | (sub ? 0x80000000u : 0)); return raw(body.b);
    }
};

static float frand() { return (float)((int)pick(4001) - 2000) / 512.0f; }

static const char* const kPivots[] = {"MUZZLE01", "MUZZLE02", "FIREPOINT01", "FIREPOINT02", "FIREPOINT03",
    "WEAPONA01", "WEAPONA02", "LAUNCH01", "LAUNCH02", "FX01", "FX02", "RECOIL01", "DOCKSTART", "DOCKACTION",
    "DOCKEND", "DOCKWAITING01", "DOCKWAITING02", "DOCKWAITING03", "TURRET", "TURRETEL", "TURRET01", "MOUNT01",
    "STANDARD", "Muzzle"};
static const char* const kSubs[] = {"GUN01", "GUN02", "MUZZLEFX01", "LAUNCH", "FIREPOINT01", "BARREL"};

static const int kFrames = 4;

// "m<tag>.w3d": a hierarchy (or "sk<tag>.w3d" of its own), boxes and an HLOD.
static void model_files(const std::string& tag) {
    const std::string skel = "SK" + tag;
    const int pivots = 10 + pick(16);
    W header, body;
    header.u32(0x00040001).name(skel, 16).u32(pivots).f(0).f(0).f(0);
    for (int i = 0; i < pivots; ++i) {
        float q[4] = {frand(), frand(), frand(), frand()};
        const float len = sqrtf(q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]);
        if (len > 0.0f) for (float& v : q) v /= len;
        body.name(i == 0 ? "ROOTTRANSFORM" : kPivots[pick(24)], 16);
        body.u32((uint32_t)(i == 0 ? -1 : (int)pick(i))).f(frand() * 7).f(frand() * 7).f(frand() * 7);
        body.f(0).f(0).f(0).f(q[0]).f(q[1]).f(q[2]).f(q[3]);
    }
    W hierarchy, h;
    h.chunk(0x101, header).chunk(0x102, body);
    hierarchy.chunk(0x100, h, true);

    W file;
    if (chance(20)) g_files[lower(skel) + ".w3d"] = hierarchy.b;
    else file.raw(hierarchy.b);

    std::vector<std::string> subs;
    for (int s = pick(4); s > 0; --s) subs.push_back(kSubs[pick(6)]);
    std::vector<std::string> boxed;
    for (const std::string& s : subs) {
        if (std::find(boxed.begin(), boxed.end(), s) != boxed.end()) continue;
        boxed.push_back(s);
        if (chance(10)) continue;  // sometimes missing
        W box;
        box.u32(0x00010000).u32(0).name("M" + tag + "." + s, 32).u32(0).f(0).f(0).f(0).f(1).f(1).f(1);
        file.chunk(0x740, box);
    }
    W lh;
    lh.u32(0x00010000).u32(1).name("M" + tag, 16).name(skel, 16);
    W hl, arr, ah;
    hl.chunk(0x701, lh);
    ah.u32((uint32_t)subs.size()).f(0.0f);
    arr.chunk(0x703, ah);
    for (const std::string& s : subs) {
        W so;
        so.u32(pick(pivots)).name("M" + tag + "." + s, 32);
        arr.chunk(0x704, so);
    }
    hl.chunk(0x702, arr, true);
    file.chunk(0x700, hl, true);
    g_files["m" + lower(tag) + ".w3d"] = file.b;
}

// "<anim>.w3d": a raw animation "SK<tag>.<anim>".
static void anim_file(const std::string& tag, const std::string& anim) {
    W ah;
    ah.u32(0x00040001).name(anim, 16).name("SK" + tag, 16).u32(kFrames).u32(15);
    W a;
    a.chunk(0x201, ah);
    for (int c = 6 + pick(20); c > 0; --c) {
        const int kind = pick(8) < 3 ? 6 : (int)pick(3);
        const int len = kind == 6 ? 4 : 1;
        W ch;
        ch.u16(0).u16(kFrames - 1).u16(len).u16(kind).u16(pick(20)).u16(0);
        for (int i = 0; i < kFrames * len; ++i) ch.f(kind == 6 ? frand() / 4.0f : frand() * 3);
        ch.u32(0);  // exporter padding
        a.chunk(0x202, ch);
    }
    W file;
    file.chunk(0x200, a, true);
    g_files[lower(anim) + ".w3d"] = file.b;
}

static const char* const kWeaponBoneNames[] = {"Muzzle", "WeaponA", "Launch", "FX", "Recoil", "MuzzleFX",
    "Gun", "Barrel", "Launch01", "NONE", "FirePoint"};

// A state of an object's draw module: `tags` name the models it may use.
static std::string object_state(const std::vector<std::string>& tags, const std::string& anim_tag,
                                bool has_model) {
    std::string out;
    if (!has_model && chance(35)) {
        const int m = pick((uint32_t)tags.size() + 1);
        out += "      Model = " + (m < (int)tags.size() ? "M" + tags[m] : std::string("NONE")) + "\n";
    }
    for (int slot = 0; slot < 3; ++slot) {
        if (!chance(slot == 0 ? 80 : 35)) continue;
        for (int field = 0; field < 4; ++field) {
            if (!chance(55)) continue;
            out += std::string("      ") + kWeaponFields[field] + " = " + kSlots[slot] + " "
                + kWeaponBoneNames[pick(11)] + "\n";
        }
    }
    if (chance(45)) out += "      Turret = Turret\n";
    if (chance(35)) out += "      TurretPitch = TurretEL\n";
    if (chance(15)) out += "      AltTurret = Mount01\n";
    if (chance(15)) out += "      AltTurretPitch = Missing\n";
    if (chance(40)) out += "      TurretArtAngle = " + std::to_string((int)pick(361) - 180) + "\n";
    if (chance(30)) out += "      TurretArtPitch = " + std::to_string((int)pick(91) - 45) + "\n";
    if (chance(15)) out += "      AltTurretArtAngle = " + std::to_string((int)pick(361) - 180) + "\n";
    if (chance(30)) {
        const char* kind = chance(80) ? "A" : "B";  // B is sometimes missing
        out += "      Animation = SK" + anim_tag + "." + kind + anim_tag + "\n";
    }
    if (chance(25)) out += "      Flags = PRISTINE_BONE_POS_IN_FINAL_FRAME\n";
    return out;
}

static std::string draw_module(const std::vector<std::string>& tags, const std::string& anim_tag,
                               int index, bool attach) {
    std::string d = std::string("  Draw = ") + (index ? "W3DTankDraw" : "W3DModelDraw") + " ModuleTag_Draw"
        + std::to_string(index) + "\n";
    if (chance(70)) d += "    ExtraPublicBone = DockStart DockAction DockEnd DockWaiting\n";
    if (chance(80)) d += "    ExtraPublicBone = FirePoint Mount Gun MuzzleFX Barrel\n";
    if (attach) d += "    AttachToBoneInAnotherModule = " + std::string(chance(80) ? "Muzzle01" : "Missing") + "\n";
    d += "    DefaultConditionState\n      Model = M" + tags[0] + "\n"
        + object_state(tags, anim_tag, true) + "    End\n";
    if (chance(70)) d += "    ConditionState = DAMAGED\n" + object_state(tags, anim_tag, false) + "    End\n";
    if (chance(60)) d += "    ConditionState = REALLYDAMAGED\n" + object_state(tags, anim_tag, false) + "    End\n";
    if (chance(40)) d += "    ConditionState = MOVING DAMAGED\n" + object_state(tags, anim_tag, false) + "    End\n";
    d += "  End\n";
    return d;
}

// ---- queries -----------------------------------------------------------------

static const char* const kQueries[] = {"Muzzle", "FirePoint", "WeaponA", "Launch", "FX", "Recoil", "DockStart",
    "DockAction", "DockEnd", "DockWaiting", "Turret", "TurretEL", "Mount", "Gun", "MuzzleFX", "Barrel",
    "none", "Standard", "muzzle01", "Missing"};

static std::string mtx_text(const Matrix3D& m) {
    std::string out;
    for (int r = 0; r < 3; ++r)
        for (int c = 0; c < 4; ++c) out += flt(m[r][c]);
    return out;
}
static std::string pos_text(const Coord3D& p) { return flt(p.x) + flt(p.y) + flt(p.z); }

static std::string pristine_query(const Drawable* draw) {
    const char* name = kQueries[pick(20)];
    const Int start = pick(3);
    const int maxes[] = {1, 2, 3, 10, 70};
    const Int most = maxes[pick(5)];
    static Matrix3D transforms[160];
    const Int count = draw->getPristineBonePositions(name, start, nullptr, transforms, most);
    std::string out = std::string("P:") + name + ":" + std::to_string(start) + ":" + std::to_string(most) + "=";
    out += std::to_string(count) + ":";
    for (int i = 0; i < count; ++i) out += mtx_text(transforms[i]);
    return out;
}

static void object_cases(oracle_wp51::World& w) {
    g_rows.push_back("{\"case\":\"standard\",\"names\":\"Standard,Muzzle\"}");
    for (int i = 0; i < 60; ++i) {
        const std::string n = std::to_string(i);
        const std::vector<std::string> tags = {"O" + n, "P" + n};
        for (const std::string& t : tags) model_files(t);
        anim_file("O" + n, "AO" + n);
        if (chance(50)) anim_file("O" + n, "BO" + n);

        const std::string name = "BoneO" + n;
        std::string ini = "Object " + name + "\n";
        if (chance(15)) ini += "  KindOf = IGNORE_DOCKING_BONES\n";
        const char* scales[] = {"1.0", "1.005", "0.75", "1.6", "1.0"};
        const std::string scale = scales[pick(5)];
        ini += "  Scale = " + scale + "\n";  // without one the scale is 0
        const bool two = chance(45);
        ini += draw_module(tags, "O" + n, 0, !two && chance(15));
        if (two) ini += draw_module({"P" + n, "O" + n}, "O" + n, 1, chance(50));
        const int approach = chance(20) ? -1 : (int)pick(11);
        ini += "  Behavior = RepairDockUpdate ModuleTag_Dock\n    NumberApproachPositions = "
            + std::to_string(approach) + "\n  End\n";
        // Some launchers have turrets, to be turned and pitched about their bones.
        const bool turreted = chance(50);
        if (turreted) {
            ini += "  Behavior = AIUpdateInterface ModuleTag_AI\n    Turret\n"
                   "      ControlledWeaponSlots = PRIMARY SECONDARY\n    End\n"
                   "    AltTurret\n      ControlledWeaponSlots = TERTIARY\n    End\n  End\n";
        }
        ini += "End\n";
        try {
            w.load_ini(ini);
        } catch (const INIException& e) {
            fprintf(stderr, "%s\n%s", e.mFailureMessage ? e.mFailureMessage : "INI error", ini.c_str());
            exit(4);
        }
        TheThingFactory->postProcessLoad();

        const Coord3D at = {(Real)(10 + pick(500)), (Real)(10 + pick(500)), (Real)pick(20)};
        const Real angle = frand();
        Object* obj = w.spawn_object(name.c_str(), w.team_a, at, angle);
        if (!obj || !obj->getDrawable()) exit(5);
        const Drawable* draw = obj->getDrawable();
        g_rows.push_back("{\"case\":\"object\",\"name\":\"" + name + "\",\"approach\":\""
            + std::to_string(approach) + "\",\"at\":\"" + pos_text(at) + flt(angle) + "\",\"ini\":\""
            + hex_text(ini) + "\"}");

        DockUpdate* update = nullptr;
        for (BehaviorModule** b = obj->getBehaviorModules(); *b; ++b)
            if (!update) update = dynamic_cast<DockUpdate*>(*b);
        if (!update) exit(6);

        for (int c = 0; c < 6; ++c) {
            ModelConditionFlags set, clear;
            const char* bits[] = {"DAMAGED", "REALLYDAMAGED", "MOVING", "ATTACKING"};
            for (int b = 0; b < 4; ++b) {
                const int bit = ModelConditionFlags::getSingleBitFromName(bits[b]);
                clear.set(bit);
                if (c && chance(35)) set.set(bit);
            }
            obj->clearAndSetModelConditionFlags(clear, set);

            std::string turn;
            if (AIUpdateInterface* ai = obj->getAIUpdateInterface()) {
                for (int t = 0; t < MAX_TURRETS; ++t) {
                    if (!ai->m_turretAI[t]) exit(7);
                    ai->m_turretAI[t]->m_angle = frand();
                    ai->m_turretAI[t]->m_pitch = frand() / 4.0f;
                    turn += flt(ai->m_turretAI[t]->m_angle) + flt(ai->m_turretAI[t]->m_pitch);
                }
            }
            std::string out;
            for (int q = 0; q < 24; ++q) out += pristine_query(draw) + ";";
            for (int slot = 0; slot < 3; ++slot) {
                out += "B:" + std::to_string(slot) + "="
                    + std::to_string(draw->getBarrelCount((WeaponSlotType)slot)) + ";";
            }
            for (int q = 0; q < 8; ++q) {
                const int slot = q < 3 ? q : (int)pick(3);
                const int barrel = (int)pick(5);
                const int tur = (int)pick(3) - 1;
                Matrix3D launch(true);
                Coord3D rot = {7, 7, 7}, pitch = {7, 7, 7};
                const Bool ok = draw->getProjectileLaunchOffset((WeaponSlotType)slot, barrel, &launch,
                    (WhichTurretType)tur, &rot, &pitch);
                out += "L:" + std::to_string(slot) + ":" + std::to_string(barrel) + ":" + std::to_string(tur)
                    + "=" + std::to_string(ok ? 1 : 0) + ":" + (ok ? mtx_text(launch) : std::string()) + ":"
                    + pos_text(rot) + pos_text(pitch) + ";";
            }
            for (int q = 0; q < 4; ++q) {
                const int slot = q < 2 ? 0 : q == 2 ? 2 : (int)pick(3);
                const int barrel = (int)pick(3);
                Matrix3D world(true);
                Coord3D where;
                Weapon::calcProjectileLaunchPosition(obj, (WeaponSlotType)slot, barrel, world, where);
                out += "W:" + std::to_string(slot) + ":" + std::to_string(barrel) + "=" + mtx_text(world) + ";";
            }
            {
                update->m_enterPosition.zero();
                update->m_dockPosition.zero();
                update->m_exitPosition.zero();
                for (Coord3D& p : update->m_approachPositions) p.zero();
                update->m_numberApproachPositionBones = -1;
                update->loadDockPositions();
                out += "D=" + pos_text(update->m_enterPosition) + pos_text(update->m_dockPosition)
                    + pos_text(update->m_exitPosition) + ":"
                    + std::to_string(update->m_numberApproachPositionBones) + ":";
                for (const Coord3D& p : update->m_approachPositions) out += pos_text(p);
                out += ";";
            }
            g_rows.push_back("{\"case\":\"query\",\"name\":\"" + name + "\",\"conditions\":\""
                + bits_text(draw->getModelConditionFlags()) + "\",\"turrets\":\"" + turn
                + "\",\"results\":\"" + out + "\"}");
        }
        TheGameLogic->destroyObject(obj);
        w.flush_destroy_list();
    }
    for (const auto& file : g_files) {
        g_rows.push_back("{\"case\":\"file\",\"name\":\"" + file.first + "\",\"hex\":\""
            + hex_text(std::string(file.second.begin(), file.second.end())) + "\"}");
    }
}

struct InUpdateTag { using type = Bool GameLogic::*; };
template struct MemberAccess<InUpdateTag, &GameLogic::m_isInUpdate>;
inline Bool GameLogic::*access_member(InUpdateTag);

int main(int argc, char** argv) {
    if (argc < 2) return 2;
    oracle_wp51::World w;
    TheModuleFactory = NEW W3DModuleFactory;
    TheModuleFactory->init();
    spec_cases(w);

    // The models: files from memory, loaded on demand as the game loads them.
    static MemFactory factory;
    _TheFileFactory = &factory;
    WWMath::Init();
    W3DDisplay::m_assetManager = new W3DAssetManager;
    W3DDisplay::m_assetManager->Set_WW3D_Load_On_Demand(true);
    TheWritableGlobalData->m_standardPublicBones.push_back("Standard");
    TheWritableGlobalData->m_standardPublicBones.push_back("Muzzle");
    // Bones are only validated inside a logic update.
    TheGameLogic->*access_member(InUpdateTag()) = TRUE;
    object_cases(w);

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
