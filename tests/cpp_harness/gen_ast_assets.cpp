// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for the asset readers (`crates/zh_assets`), produced by the
// unmodified reference loaders: HTreeClass, HRawAnimClass, HCompressedAnimClass,
// MeshGeometryClass, HLodDefClass, HLodClass, W3DModelDraw's pristine bone and
// barrel code, Targa, DDSFileClass and GameTextManager::parseCSF.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_ast_assets.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/AST_assets.json
//
// All "case" rows use inputs built here. The "retail" rows read the installed
// game's archives in place ($CNC_GENERALS_ZH_PATH or ~/Games/generalszh) and
// record only counts and digests.
//
// Not covered: materials, emitters and collision trees (no headless loader is
// compared), adaptive delta animation, HLODs with more than one LOD.

#include "oracle_common.h"

struct SDL_Window;

#include <algorithm>
#include <dirent.h>
#include <map>
#include <memory>
#include <string>
#include <vector>

#include "WWLib/hashtemplate.h"
#include "WWLib/wwstring.h"

// Reach the loaders' private state; layouts are unchanged.
#define private public
#define protected public
#include "Common/GameState.h"
#include "Common/LocalFileSystem.h"
#include "Common/FileSystem.h"
#include "Common/NameKeyGenerator.h"
#include "Common/RAMFile.h"
#include "GameLogic/GameLogic.h"
#include "../../vendor/GeneralsX/Core/GameEngineDevice/Include/W3DDevice/GameClient/Module/W3DModelDraw.h"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngineDevice/Include/W3DDevice/GameClient/W3DAssetManager.h"
#include "WW3D2/hanim.h"
#include "WW3D2/hcanim.h"
#include "WW3D2/hlod.h"
#include "WW3D2/hrawanim.h"
#include "WW3D2/htree.h"
#include "WW3D2/htreemgr.h"
#include "WW3D2/meshgeometry.h"
#include "WW3D2/ddsfile.h"
#include "WWLib/chunkio.h"
#include "WWLib/ffactory.h"
#include "WWLib/ramfile.h"
#include "WWLib/TARGA.h"
#include "WWMath/wwmath.h"
#include "WWMath/quat.h"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameClient/GameText.cpp"
#undef private
#undef protected

// Application globals the engine and device libraries expect from the executable.
HWND ApplicationHWnd = nullptr;
SDL_Window* TheSDL3Window = nullptr;
int __argc = 0;
char** __argv = nullptr;
const Char* g_strFile = "data/Generals.str";
const Char* g_csfFile = "data/%s/generals.csf";

typedef std::vector<uint8_t> Bytes;

// ---- canonical output ------------------------------------------------------

static std::string g_out;
static void word(uint32_t v) { char b[9]; snprintf(b, sizeof b, "%08x", v); g_out += b; }
static void flt(float f) { uint32_t u; memcpy(&u, &f, 4); word(u); }
static void text(const char* s) { g_out += s; g_out += ';'; }
static void hex(const void* p, size_t n) {
    static const char* d = "0123456789abcdef";
    const uint8_t* b = static_cast<const uint8_t*>(p);
    for (size_t i = 0; i < n; ++i) { g_out += d[b[i] >> 4]; g_out += d[b[i] & 15]; }
}
static std::string hexs(const Bytes& b) { std::string keep; keep.swap(g_out); hex(b.data(), b.size()); keep.swap(g_out); return keep; }
static void mtx(const Matrix3D& m) { for (int r = 0; r < 3; ++r) for (int c = 0; c < 4; ++c) flt(m[r][c]); }
static std::string take() { std::string s; s.swap(g_out); return s; }

// ---- input building --------------------------------------------------------

static uint32_t g_state = 0x5a17c3d1u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13; g_state ^= g_state >> 17; g_state ^= g_state << 5;
    return g_state % n;
}
static float frand() { return (float)((int)pick(4001) - 2000) / 512.0f; }

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

// ---- file access -----------------------------------------------------------

static std::map<std::string, Bytes> g_files;   // lower-case name -> content
static std::string lower(std::string s) { for (char& c : s) c = (char)tolower((unsigned char)c); return s; }

struct MemFactory : FileFactoryClass {
    FileClass* Get_File(const char* name) override {
        auto it = g_files.find(lower(name));
        if (it == g_files.end()) return new RAMFileClass((void*)nullptr, 0);
        return new RAMFileClass(it->second.data(), (int)it->second.size());
    }
    void Return_File(FileClass* f) override { delete f; }
};

struct MemFileSystem : LocalFileSystem {
    void init() override {}
    void reset() override {}
    void update() override {}
    File* openFile(const Char* name, Int, size_t) override {
        auto it = g_files.find(lower(name));
        if (it == g_files.end()) return nullptr;
        RAMFile* f = newInstance(RAMFile);
        f->m_size = (Int)it->second.size();
        f->m_data = MSGNEW("RAMFILE") char[f->m_size ? f->m_size : 1];
        memcpy(f->m_data, it->second.data(), f->m_size);
        f->m_pos = 0;
        f->m_open = TRUE;
        f->m_access = File::READ | File::BINARY;
        f->deleteOnClose();
        return f;
    }
    Bool doesFileExist(const Char* n) const override { return g_files.count(lower(n)) != 0; }
    void getFileListInDirectory(const AsciiString&, const AsciiString&, const AsciiString&, FilenameList&, Bool) const override {}
    Bool getFileInfo(const AsciiString&, FileInfo*) const override { return FALSE; }
    Bool createDirectory(AsciiString) override { return FALSE; }
    AsciiString normalizePath(const AsciiString& p) const override { return p; }
};

// ---- chunk walking ---------------------------------------------------------

struct Top { uint32_t id; const uint8_t* p; uint32_t size; };
static std::vector<Top> tops(const Bytes& b) {
    std::vector<Top> out;
    size_t at = 0;
    while (at + 8 <= b.size()) {
        uint32_t id, size;
        memcpy(&id, &b[at], 4); memcpy(&size, &b[at + 4], 4);
        size &= 0x7fffffffu; at += 8;
        if (size > b.size() - at) break;
        out.push_back({id, b.data() + at - 8, size + 8});
        at += size;
    }
    return out;
}
// Runs fn with a loader positioned inside the top-level chunk.
template <class Fn> static bool with_chunk(const Top& t, Fn fn) {
    RAMFileClass file((void*)t.p, (int)t.size);
    file.Open();
    ChunkLoadClass cload(&file);
    if (!cload.Open_Chunk()) return false;
    return fn(cload);
}

// ---- dumps -----------------------------------------------------------------

static void dump_tree_pose(HTreeClass& t) {
    for (int i = 0; i < t.Num_Pivots(); ++i) {
        mtx(t.Get_Transform(i));
        word(t.Get_Visibility(i) ? 1 : 0);
    }
}
static void dump_hierarchy(HTreeClass& t) {
    word(t.Num_Pivots());
    for (int i = 0; i < t.Num_Pivots(); ++i) { text(t.Get_Bone_Name(i)); word(t.Get_Parent_Index(i)); }
    Matrix3D root(true);
    t.Base_Update(root);
    dump_tree_pose(t);
}
static void dump_anim(HTreeClass& tree, HAnimClass& anim, const std::vector<float>& frames) {
    word(anim.Get_Num_Frames());
    for (float f : frames) {
        HTreeClass copy(tree);
        Matrix3D root(true);
        copy.Anim_Update(root, &anim, f);
        dump_tree_pose(copy);
    }
}
static std::vector<float> sample_frames(int n) {
    return {0.0f, 0.5f, 1.0f, (float)(n / 2) + 0.25f, (float)(n - 1), n > 1 ? (float)(n - 1) - 0.25f : 0.0f};
}
static std::unique_ptr<HAnimClass> load_anim(const Top& t) {
    std::unique_ptr<HAnimClass> a;
    if (t.id == 0x200) {
        HRawAnimClass* r = new HRawAnimClass;
        if (with_chunk(t, [&](ChunkLoadClass& c) { return r->Load_W3D(c) == HRawAnimClass::OK; })) a.reset(r);
        else delete r;
    } else {
        HCompressedAnimClass* r = new HCompressedAnimClass;
        if (with_chunk(t, [&](ChunkLoadClass& c) { return r->Load_W3D(c) == HCompressedAnimClass::OK; })) a.reset(r);
        else delete r;
    }
    return a;
}
static void dump_mesh(MeshGeometryClass& m) {
    text(m.Get_Name());
    const int nv = m.Get_Vertex_Count(), np = m.Get_Polygon_Count();
    word(nv); word(np);
    const Vector3* v = m.Get_Vertex_Array();
    for (int i = 0; i < nv; ++i) { flt(v[i].X); flt(v[i].Y); flt(v[i].Z); }
    const Vector3* n = m.get_vert_normals();
    for (int i = 0; i < nv; ++i) { flt(n[i].X); flt(n[i].Y); flt(n[i].Z); }
    const TriIndex* p = m.Get_Polygon_Array();
    const uint8* s = m.Get_Poly_Surface_Type_Array();
    for (int i = 0; i < np; ++i) { word((uint16_t)p[i].I); word((uint16_t)p[i].J); word((uint16_t)p[i].K); word(s[i]); }
    const uint16* links = m.get_bone_links(false);
    word(links != nullptr);
    if (links) for (int i = 0; i < nv; ++i) word(links[i]);
    const uint32* shade = m.get_shade_indices(false);
    word(shade != nullptr);
    if (shade) for (int i = 0; i < nv; ++i) word(shade[i]);
    flt(m.BoundBoxMin.X); flt(m.BoundBoxMin.Y); flt(m.BoundBoxMin.Z);
    flt(m.BoundBoxMax.X); flt(m.BoundBoxMax.Y); flt(m.BoundBoxMax.Z);
    flt(m.BoundSphereCenter.X); flt(m.BoundSphereCenter.Y); flt(m.BoundSphereCenter.Z); flt(m.BoundSphereRadius);
}
static void dump_hlod(HLodDefClass& d) {
    text(d.Name); text(d.HierarchyTreeName);
    word(d.LodCount);
    auto arr = [](HLodDefClass::SubObjectArrayClass& a) {
        flt(a.MaxScreenSize); word(a.ModelCount);
        for (int i = 0; i < a.ModelCount; ++i) { text(a.ModelName[i]); word(a.BoneIndex[i]); }
    };
    for (int l = 0; l < d.LodCount; ++l) arr(d.Lod[l]);
    word(d.Aggregates.ModelCount);
    for (int i = 0; i < d.Aggregates.ModelCount; ++i) { text(d.Aggregates.ModelName[i]); word(d.Aggregates.BoneIndex[i]); }
}

static void dump_tga(const std::string& name) {
    Targa t;
    const long err = t.Load(name.c_str(), TGAF_IMAGE | TGAF_PAL, false);
    if (err) { text("error"); return; }
    const size_t bpp = (t.Header.PixelDepth + 7) / 8;
    word(t.Header.Width); word(t.Header.Height); word(t.Header.PixelDepth);
    word(t.Header.ImageType); word((uint8_t)t.Header.ImageDescriptor);
    hex(t.GetImage(), (size_t)t.Header.Width * t.Header.Height * bpp);
    if (t.Header.ColorMapType == 1) { text("pal"); hex(t.GetPalette(), t.Header.CMapLength * (t.Header.CMapDepth >> 3)); }
}
static void dump_dds(const std::string& name) {
    DDSFileClass d(name.c_str(), 0);
    if (!d.Load()) { text("error"); return; }
    word(d.Get_Width(0)); word(d.Get_Height(0)); word(d.Get_Mip_Level_Count()); word(d.Get_Type());
    for (unsigned l = 0; l < d.Get_Mip_Level_Count(); ++l) {
        word(d.Get_Level_Size(l));
        // Cube maps keep only face 0 here; the level data is the first face's.
        hex(d.Get_Memory_Pointer(l), d.Get_Level_Size(l));
    }
}
static void dump_csf(const std::string& name) {
    GameTextManager* g = new GameTextManager;
    Int count = 0, longest = 0;
    LanguageID lang = LANGUAGE_ID_US;
    if (!g->getCSFInfo(name.c_str(), count, lang, 0)) { text("noinfo"); return; }
    std::vector<StringInfo> info(count + 1);
    const Bool ok = g->parseCSF(name.c_str(), info.data(), count, longest, 0);
    word(ok); word(count); word(lang);
    for (int i = 0; i < count; ++i) {
        const StringInfo& s = info[i];
        text(s.label.str());
        const WideChar* w = s.text.str();
        word(s.text.getLength());
        for (int k = 0; k < s.text.getLength(); ++k) word((uint32_t)w[k] & 0xffff);
        text(s.speech.str());
    }
}

// ---- hand-built cases ------------------------------------------------------

static const char* const kBones[] = {"TURRET01", "Muzzle01", "MUZZLE02", "FirePoint01", "FIREPOINT02", "WeaponA",
    "WEAPONA01", "recoil01", "Door01", "LAUNCH", "FX01", "muzzlefx01", "MUZZLEFX02", "Barrel01"};
static const char* const kQueries[] = {"muzzle", "firepoint", "weapona", "recoil", "door", "launch", "fx",
    "muzzlefx", "barrel", "turret", "box", "gun", "none", "root", "muzzle01", "weapona01", "flash"};
static const char* const kSubs[] = {"MODEL.GUN01", "MODEL.GUN02", "MODEL.BOX", "MODEL.FLASH01", "MODEL.MUZZLE01",
    "MODEL.LAUNCH", "MODEL.FIREPOINT01", "MODEL.BARREL"};

static Bytes quat_pivot(const std::string& name, int parent) {
    W p;
    float q[4] = {frand(), frand(), frand(), frand()};
    if (pick(5) != 0) {
        const float len = sqrtf(q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]);
        if (len > 0.0f) for (float& v : q) v /= len;
    }
    p.name(name, 16).u32((uint32_t)parent).f(frand() * 7).f(frand() * 7).f(frand() * 7).f(0).f(0).f(0);
    p.f(q[0]).f(q[1]).f(q[2]).f(q[3]);
    return p.b;
}

static float quat_part() { return frand() / 4.0f; }

struct Model { W file; int pivots; int rawFrames; int tcFrames; };

static Model build_model(int index) {
    Model m;
    const std::string tag = std::to_string(index);
    const bool old = index % 11 == 5;  // pre-3.0 hierarchy: the root is implicit
    m.pivots = 3 + pick(9);
    W header, pivots;
    header.u32(old ? 0x00020000 : 0x00040001).name("SKEL" + tag, 16).u32(old ? m.pivots - 1 : m.pivots).f(0).f(0).f(0);
    std::vector<int> names;
    for (int i = old ? 1 : 0; i < m.pivots; ++i) {
        const int parent = i == 0 ? -1 : (int)pick(i) - (old ? 1 : 0);
        std::string name = i == 0 ? "ROOTTRANSFORM" : kBones[pick(14)];
        pivots.raw(quat_pivot(name, parent));
    }
    W h; h.chunk(0x101, header).chunk(0x102, pivots);
    m.file.chunk(0x100, h, true);

    // Raw animation.
    m.rawFrames = 2 + pick(5);
    W ah; ah.u32(old ? 0x00020000 : 0x00040001).name("RAW" + tag, 16).name("SKEL" + tag, 16).u32(m.rawFrames).u32(15);
    W a; a.chunk(0x201, ah);
    const int channels = 3 + pick(14);
    for (int c = 0; c < channels; ++c) {
        const int kind = pick(8) < 4 ? 6 : (int)pick(4);  // 3 is XR, which the engine drops
        const int len = kind == 6 ? 4 : 1;
        const int first = pick(2) ? 0 : pick(m.rawFrames);
        const int last = first + pick(m.rawFrames - first);
        W ch; ch.u16(first).u16(last).u16(len).u16(kind).u16(pick(m.pivots + 1)).u16(0);
        for (int i = 0; i < (last - first + 1) * len; ++i) ch.f(kind == 6 ? quat_part() : frand() * 3);
        ch.u32(0);  // exporter padding
        a.chunk(0x202, ch);
    }
    for (int c = pick(3); c > 0; --c) {
        const int first = pick(m.rawFrames), last = first + pick(m.rawFrames - first);
        W ch; ch.u16(first).u16(last).u16(0).u16(pick(m.pivots)).u8(pick(2));
        for (int i = 0; i < (last - first + 1 + 7) / 8; ++i) ch.u8(pick(256));
        a.chunk(0x203, ch);
    }
    m.file.chunk(0x200, a, true);

    // Time coded animation.
    m.tcFrames = 4 + pick(20);
    W th; th.u32(1).name("TC" + tag, 16).name("SKEL" + tag, 16).u32(m.tcFrames).u16(30).u16(0);
    W t; t.chunk(0x281, th);
    for (int c = 3 + pick(12); c > 0; --c) {
        const int kind = pick(8) < 4 ? 6 : (int)pick(3);
        const int len = kind == 6 ? 4 : 1;
        std::vector<uint32_t> times{0};
        for (uint32_t at = 0;;) { at += 1 + pick(6); if (at >= (uint32_t)m.tcFrames) break; times.push_back(at); }
        W ch; ch.u32((uint32_t)times.size()).u16(pick(m.pivots + 1)).u8(len).u8(kind);
        for (size_t k = 0; k < times.size(); ++k) {
            ch.u32(times[k] | ((k && pick(4) == 0) ? 0x80000000u : 0));
            for (int i = 0; i < len; ++i) ch.f(kind == 6 ? quat_part() : frand() * 3);
        }
        t.chunk(0x282, ch);
    }
    for (int c = pick(3); c > 0; --c) {
        std::vector<uint32_t> codes{pick(2) ? 0x80000000u : 0};
        for (uint32_t at = 0;;) { at += 1 + pick(6); if (at >= (uint32_t)m.tcFrames) break; codes.push_back(at | (pick(2) ? 0x80000000u : 0)); }
        W ch; ch.u32((uint32_t)codes.size()).u16(pick(m.pivots)).u8(0).u8(pick(2));
        for (uint32_t code : codes) ch.u32(code);
        t.chunk(0x283, ch);
    }
    m.file.chunk(0x280, t, true);

    // Boxes used as sub-objects, then the HLOD that places them.
    std::vector<std::string> subs;
    const int nsubs = pick(5);
    for (int s = 0; s < nsubs; ++s) subs.push_back(kSubs[pick(8)]);
    std::vector<std::string> boxed;
    for (const std::string& s : subs) {
        if (std::find(boxed.begin(), boxed.end(), s) != boxed.end()) continue;
        boxed.push_back(s);
        if (s == "MODEL.BARREL" && pick(2)) continue;  // sometimes missing
        std::string unique = "M" + tag + s.substr(5);
        W box; box.u32(0x00010000).u32(0).name(unique, 32).u32(0).f(0).f(0).f(0).f(1).f(1).f(1);
        m.file.chunk(0x740, box);
    }
    // An aggregate is named so that no bone query finds it: the engine's bone
    // lookup for a sub-object past the first LOD reads out of bounds.
    const int agg = nsubs > 1 && pick(3) == 0 ? 1 : 0;
    if (agg) {
        subs.back() = "MODEL.ZZ";
        W box; box.u32(0x00010000).u32(0).name("M" + tag + ".ZZ", 32).u32(0).f(0).f(0).f(0).f(1).f(1).f(1);
        m.file.chunk(0x740, box);
    }
    W lh; lh.u32(0x00010000).u32(1).name("M" + tag, 16).name("SKEL" + tag, 16);
    W hl; hl.chunk(0x701, lh);
    auto array = [&](uint32_t id, int from, int to) {
        W arr, ah2; ah2.u32(to - from).f(0.0f); arr.chunk(0x703, ah2);
        for (int s = from; s < to; ++s) {
            W so; so.u32(pick(m.pivots)).name("M" + tag + subs[s].substr(5), 32);
            arr.chunk(0x704, so);
        }
        hl.chunk(id, arr, true);
    };
    array(0x702, 0, nsubs - agg);
    if (agg) array(0x705, nsubs - agg, nsubs);
    m.file.chunk(0x700, hl, true);
    return m;
}

static void model_case(int index, FILE* f) {
    Model m = build_model(index);
    const std::string tag = std::to_string(index);
    WW3DAssetManager* am = WW3DAssetManager::Get_Instance();
    RAMFileClass file(m.file.b.data(), (int)m.file.b.size());
    if (!am->Load_3D_Assets(file)) exit(3);

    HTreeClass* tree = am->Get_HTree(("SKEL" + tag).c_str());
    if (!tree) exit(4);
    HTreeClass base(*tree);
    dump_hierarchy(base);
    const std::string hier = take();

    HAnimClass* anims[2] = {nullptr, nullptr};
    std::string poses[2];
    const std::string anim_names[2] = {"SKEL" + tag + ".RAW" + tag, "SKEL" + tag + ".TC" + tag};
    for (int a = 0; a < 2; ++a) {
        anims[a] = am->Get_HAnim(anim_names[a].c_str());
        if (!anims[a]) exit(5);
        std::vector<float> frames = sample_frames(anims[a]->Get_Num_Frames());
        for (int i = 0; i < 2; ++i) frames.push_back((float)pick(64 * (anims[a]->Get_Num_Frames() - 1) + 1) / 64.0f);
        for (float fr : frames) flt(fr);
        g_out += '|';
        dump_anim(base, *anims[a], frames);
        poses[a] = take();
    }

    static const float kScales[] = {1.0f, 1.005f, 0.5f, 2.25f, 0.98f, 1.0f};
    const float scale = kScales[pick(6)];
    const int which = pick(3);  // 0 base pose, 1 raw, 2 time coded
    const bool last = pick(2);
    RenderObjClass* robj = static_cast<W3DAssetManager*>(am)->Create_Render_Obj(("M" + tag).c_str(), scale, 0);
    if (!robj) exit(6);
    if (which) robj->Set_Animation(anims[which - 1], 1.0f, RenderObjClass::ANIM_MODE_MANUAL);

    for (int s = 0; s < robj->Get_Num_Sub_Objects(); ++s) {
        RenderObjClass* sub = robj->Get_Sub_Object(s);
        text(sub->Get_Name());
        word(robj->Get_Sub_Object_Bone_Index(sub));
        sub->Release_Ref();
    }
    const std::string subs = take();

    ModelConditionInfo info;
    info.m_modelName = ("M" + tag).c_str();
    info.m_flags = last ? (1 << 4) : 0;  // PRISTINE_BONE_POS_IN_FINAL_FRAME
    std::vector<std::string> queries;
    for (int q = 0; q < 17; ++q) if (pick(3) != 0) queries.push_back(kQueries[q]);
    // Weapon bone fields. Slot 0 favours names that exist; the others are random.
    std::vector<std::string> slots;
    for (int w = 0; w < WEAPONSLOT_COUNT; ++w) {
        for (int field = 0; field < 4; ++field) {
            std::string n = pick(3) == 0 ? "" : kQueries[pick(17)];
            if (n == "none") n = "";
            slots.push_back(n);
            if (!n.empty()) queries.push_back(n);
        }
        info.m_weaponFireFXBoneName[w] = slots[w * 4].c_str();
        info.m_weaponRecoilBoneName[w] = slots[w * 4 + 1].c_str();
        info.m_weaponMuzzleFlashName[w] = slots[w * 4 + 2].c_str();
        info.m_weaponProjectileLaunchBoneName[w] = slots[w * 4 + 3].c_str();
    }
    for (const std::string& q : queries) info.addPublicBone(q.c_str());
    info.m_validStuff = 0;
    info.validateCachedBones(robj, scale);
    info.validateWeaponBarrelInfo();
    if (!(info.m_validStuff & ModelConditionInfo::BARRELS_VALID)) exit(7);

    std::map<std::string, std::string> bones;
    for (const auto& it : info.m_pristineBones) {
        const std::string name = TheNameKeyGenerator->keyToName(it.first).str();
        take();
        word(it.second.boneIndex);
        mtx(it.second.mtx);
        bones[name] = take();
    }
    for (const auto& it : bones) { text(it.first.c_str()); g_out += it.second; }
    const std::string bone_dump = take();
    for (int w = 0; w < WEAPONSLOT_COUNT; ++w) {
        word((uint32_t)info.m_weaponBarrelInfoVec[w].size());
        for (const auto& b : info.m_weaponBarrelInfoVec[w]) {
            word(b.m_recoilBone); word(b.m_muzzleFlashBone); word(b.m_fxBone); mtx(b.m_projectileOffsetMtx);
        }
    }
    const std::string barrels = take();
    REF_PTR_RELEASE(robj);

    auto list = [](const std::vector<std::string>& v) {
        std::string s;
        for (const std::string& x : v) s += x + ",";
        return s;
    };
    uint32_t sc; memcpy(&sc, &scale, 4);
    fprintf(f, "{\"seed\":0,\"case\":\"model\",\"index\":%d,\"w3d\":\"%s\",\"scale\":%u,\"which\":%d,\"last\":%d,"
               "\"names\":\"%s\",\"slots\":\"%s\",\"hierarchy\":\"%s\",\"raw\":\"%s\",\"timecoded\":\"%s\","
               "\"subs\":\"%s\",\"bones\":\"%s\",\"barrels\":\"%s\"},\n",
            index, hexs(m.file.b).c_str(), sc, which, last ? 1 : 0, list(queries).c_str(), list(slots).c_str(),
            hier.c_str(), poses[0].c_str(), poses[1].c_str(), subs.c_str(), bone_dump.c_str(), barrels.c_str());
}

static void mesh_case(int index, FILE* f) {
    const int nv = 3 + pick(5), np = 1 + pick(4);
    const bool skin = pick(2), old = index % 5 == 4, links = skin || pick(2), shade = pick(2);
    const bool named = pick(3) != 0;
    W h;
    h.u32(old ? 0x00020000 : 0x00040002).u32(skin ? 0x00020000 : 0).name("PART" + std::to_string(index), 16)
        .name(named ? "CONT" : "", 16).u32(np).u32(nv).u32(1).u32(0).u32(0).u32(0).u32(0).u32(0).u32(0);
    for (int i = 0; i < 10; ++i) h.f(frand());
    W verts, norms, tris, infl, shd;
    for (int i = 0; i < nv * 3; ++i) { verts.f(frand()); norms.f(frand()); }
    for (int i = 0; i < np; ++i) {
        tris.u32(pick(nv)).u32(pick(nv)).u32(pick(nv)).u32(pick(300));
        for (int k = 0; k < 4; ++k) tris.f(frand());
    }
    for (int i = 0; i < nv; ++i) { infl.u16(pick(40)).u16(0).u32(0); shd.u32(pick(100000)); }
    W body; body.chunk(0x1f, h).chunk(2, verts).chunk(3, norms).chunk(0x20, tris);
    if (links) body.chunk(0x0e, infl);
    if (shade) body.chunk(0x22, shd);
    W file; file.chunk(0, body, true);
    MeshGeometryClass* m = new MeshGeometryClass;
    const bool ok = with_chunk(tops(file.b)[0], [&](ChunkLoadClass& c) { return m->Load_W3D(c) == WW3D_ERROR_OK; });
    if (!ok) exit(8);
    dump_mesh(*m);
    fprintf(f, "{\"seed\":0,\"case\":\"mesh\",\"index\":%d,\"w3d\":\"%s\",\"out\":\"%s\"},\n", index, hexs(file.b).c_str(), take().c_str());
    m->Release_Ref();
}

static void tga_case(int index, FILE* f) {
    static const int kTypes[] = {1, 2, 3, 9, 10};
    const int type = kTypes[index % 5];
    const bool mapped = type == 1 || type == 9;
    const int bpp = mapped || type == 3 ? 1 : (pick(2) ? 3 : 4);
    const int w = 1 + pick(6), h = 1 + pick(6);
    const int idlen = pick(3) ? 0 : pick(5);
    W t;
    t.u8(idlen).u8(mapped).u8(type).u16(0).u16(mapped ? 4 : 0).u8(mapped ? 24 : 0).u16(0).u16(0).u16(w).u16(h)
        .u8(bpp * 8).u8((index / 5 % 4) << 4);
    for (int i = 0; i < idlen; ++i) t.u8('i');
    if (mapped) for (int i = 0; i < 12; ++i) t.u8(pick(256));
    if (type >= 9) {
        for (int left = w * h; left > 0;) {
            const int n = 1 + pick(std::min(left, 7));
            if (pick(2)) { t.u8(0x80 | (n - 1)); for (int i = 0; i < bpp; ++i) t.u8(pick(256)); }
            else { t.u8(n - 1); for (int i = 0; i < n * bpp; ++i) t.u8(pick(256)); }
            left -= n;
        }
    } else {
        for (int i = 0; i < w * h * bpp; ++i) t.u8(pick(256));
    }
    for (int i = 0; i < 26; ++i) t.u8(0);  // no TGA 2.0 footer
    g_files["case.tga"] = t.b;
    dump_tga("case.tga");
    fprintf(f, "{\"seed\":0,\"case\":\"tga\",\"index\":%d,\"data\":\"%s\",\"out\":\"%s\"},\n", index, hexs(t.b).c_str(), take().c_str());
}

static void dds_case(int index, FILE* f) {
    static const char* const kCc[] = {"DXT1", "DXT3", "DXT5", "DXT1"};
    const char* cc = kCc[index % 4];
    const int block = strcmp(cc, "DXT1") == 0 ? 8 : 16;
    const int w = 1 << pick(6), h = 1 << pick(6);
    const int mips = 1 + pick(6);
    W d;
    d.raw({'D', 'D', 'S', ' '}).u32(124).u32(0x21007).u32(h).u32(w).u32(0).u32(0).u32(mips);
    for (int i = 0; i < 11; ++i) d.u32(0);
    d.u32(32).u32(4).raw({(uint8_t)cc[0], (uint8_t)cc[1], (uint8_t)cc[2], (uint8_t)cc[3]});
    for (int i = 0; i < 5; ++i) d.u32(0);
    d.u32(0x401008).u32(0).u32(0).u32(0).u32(0);
    int lw = w, lh = h;
    for (int l = 0; l < mips; ++l) {
        const int n = std::max((lw + 3) / 4, 1) * std::max((lh + 3) / 4, 1) * block;
        for (int i = 0; i < n; ++i) d.u8(pick(256));
        lw = std::max(lw / 2, 1); lh = std::max(lh / 2, 1);
    }
    g_files["case.dds"] = d.b;
    dump_dds("case.tga");
    fprintf(f, "{\"seed\":0,\"case\":\"dds\",\"index\":%d,\"data\":\"%s\",\"out\":\"%s\"},\n", index, hexs(d.b).c_str(), take().c_str());
}

static void csf_case(int index, FILE* f) {
    static const char kAlphabet[] = "ab  \n\t c %d";
    const int labels = 1 + pick(5), version = 1 + pick(3), skip = pick(3) ? 0 : 1 + pick(6);
    W c;
    c.u32(0x43534620).u32(version).u32(labels).u32(labels).u32(skip).u32(pick(8));
    for (int i = 0; i < skip; ++i) c.u8(0xee);
    for (int l = 0; l < labels; ++l) {
        const std::string name = "GUI:Label" + std::to_string(l);
        const int strings = pick(4) == 0 ? pick(3) : 1;
        c.u32(0x4c424c20).u32(strings).u32((uint32_t)name.size());
        for (char ch : name) c.u8(ch);
        for (int s = 0; s < strings; ++s) {
            const bool wave = pick(2);
            const int len = pick(14);
            c.u32(wave ? 0x53545257 : 0x53545220).u32(len);
            for (int i = 0; i < len; ++i) {
                const uint32_t r = pick(40);
                const uint16_t unit = r == 0 ? 0 : r == 1 ? 0xffff : r == 2 ? 0x2603 : (uint16_t)kAlphabet[pick(sizeof kAlphabet - 1)];
                c.u16(r < 2 ? unit : (uint16_t)~unit);
            }
            if (wave) {
                const int n = pick(6);
                c.u32(n);
                for (int i = 0; i < n; ++i) c.u8('A' + pick(26));
            }
        }
    }
    g_files["case.csf"] = c.b;
    dump_csf("case.csf");
    fprintf(f, "{\"seed\":0,\"case\":\"csf\",\"index\":%d,\"data\":\"%s\",\"out\":\"%s\"},\n", index, hexs(c.b).c_str(), take().c_str());
}

// Fast_Slerp on its own: nearly parallel, opposed and far-apart pairs.
static void slerp_case(int index, FILE* f) {
    float p[4], q[4];
    for (float& v : p) v = frand() / 4.0f;
    const float len = sqrtf(p[0] * p[0] + p[1] * p[1] + p[2] * p[2] + p[3] * p[3]);
    if (len > 0.0f) for (float& v : p) v /= len;
    static const float kNudge[] = {0.0f, 0.000001f, 0.00001f, 0.0001f, 0.001f, 0.01f, 0.1f, 0.25f};
    const float nudge = kNudge[index % 8];
    for (int i = 0; i < 4; ++i) q[i] = p[i] * (index % 3 == 1 ? -1.0f : 1.0f);
    q[pick(4)] += nudge;
    if (index % 5 == 4) for (float& v : q) v = frand() / 4.0f;
    // A cosine just short of one.
    if (index % 8 == 0 && index > 0) for (float& v : q) v *= 1.0f - 0.000000015f * (float)index * 8.0f;
    const float alpha = index % 7 == 0 ? 1.0f : (float)pick(1001) / 1000.0f;
    Quaternion r(true);
    Fast_Slerp(r, Quaternion(p[0], p[1], p[2], p[3]), Quaternion(q[0], q[1], q[2], q[3]), alpha);
    for (float v : p) flt(v);
    for (float v : q) flt(v);
    flt(alpha);
    const std::string in = take();
    flt(r.X); flt(r.Y); flt(r.Z); flt(r.W);
    fprintf(f, "{\"seed\":0,\"case\":\"slerp\",\"index\":%d,\"data\":\"%s\",\"out\":\"%s\"},\n", index, in.c_str(), take().c_str());
}

// ---- retail ----------------------------------------------------------------

struct Entry { std::string archive, name; uint32_t offset, size; };

static uint32_t be32(const uint8_t* p) { return (uint32_t)p[0] << 24 | p[1] << 16 | p[2] << 8 | p[3]; }

static std::vector<std::string> archives(const std::string& root) {
    std::vector<std::string> out;
    for (const char* sub : {"", "ZH_Generals"}) {
        const std::string dir = root + (*sub ? "/" + std::string(sub) : "");
        if (DIR* d = opendir(dir.c_str())) {
            while (dirent* e = readdir(d)) {
                const std::string n = e->d_name;
                if (n.size() > 4 && lower(n.substr(n.size() - 4)) == ".big") out.push_back((*sub ? std::string(sub) + "/" : "") + n);
            }
            closedir(d);
        }
    }
    std::sort(out.begin(), out.end());
    return out;
}

// Entries of every archive: archives by path, names (lower case) within each.
static std::vector<Entry> scan(const std::string& root) {
    std::vector<Entry> all;
    for (const std::string& a : archives(root)) {
        FILE* f = fopen((root + "/" + a).c_str(), "rb");
        if (!f) continue;
        uint8_t h[16];
        if (fread(h, 1, 16, f) != 16 || memcmp(h, "BIG", 3) != 0) { fclose(f); continue; }
        std::map<std::string, Entry> found;
        for (uint32_t i = be32(h + 8); i > 0; --i) {
            uint8_t e[8];
            if (fread(e, 1, 8, f) != 8) exit(9);
            std::string name;
            for (int c; (c = fgetc(f)) > 0;) name += (char)c;
            name = lower(name);
            found[name] = {a, name, be32(e), be32(e + 4)};
        }
        fclose(f);
        for (auto& it : found) all.push_back(it.second);
    }
    return all;
}

static Bytes read_entry(const std::string& root, const Entry& e) {
    Bytes b(e.size);
    FILE* f = fopen((root + "/" + e.archive).c_str(), "rb");
    if (!f || fseek(f, e.offset, SEEK_SET) != 0 || (e.size && fread(b.data(), 1, e.size, f) != e.size)) exit(10);
    fclose(f);
    return b;
}

struct Digest {
    uint64_t h = 0xcbf29ce484222325ull;
    uint32_t count = 0, failed = 0;
    void add(const std::string& s) { for (unsigned char c : s) { h ^= c; h *= 0x100000001b3ull; } ++count; }
    void write(FILE* f, const char* what, bool last = false) {
        fprintf(f, "{\"seed\":0,\"retail\":\"%s\",\"count\":%u,\"failed\":%u,\"digest\":\"%016llx\"}%s\n",
                what, count, failed, (unsigned long long)h, last ? "" : ",");
    }
};

static bool ends(const std::string& s, const char* ext) {
    const size_t n = strlen(ext);
    return s.size() >= n && s.compare(s.size() - n, n, ext) == 0;
}

static void retail(FILE* f) {
    const char* env = getenv("CNC_GENERALS_ZH_PATH");
    const std::string root = env ? env : std::string(getenv("HOME")) + "/Games/generalszh";
    const std::vector<Entry> all = scan(root);
    if (all.empty()) { fprintf(stderr, "no retail archives\n"); exit(11); }

    // A fresh manager, so the hand-built assets cannot answer for retail names.
    delete WW3DAssetManager::Get_Instance();
    W3DAssetManager* am = new W3DAssetManager;
    am->Set_WW3D_Load_On_Demand(false);

    Digest hier, anim, mesh, hlod, tga, dds, csf;
    // Every hierarchy first: animations name theirs. The first of a name wins.
    for (const Entry& e : all) {
        if (!ends(e.name, ".w3d")) continue;
        const Bytes b = read_entry(root, e);
        for (const Top& t : tops(b)) {
            if (t.id != 0x100) continue;
            std::unique_ptr<HTreeClass> h(new HTreeClass);
            if (!with_chunk(t, [&](ChunkLoadClass& c) { return h->Load_W3D(c) == HTreeClass::OK; })) { ++hier.failed; continue; }
            dump_hierarchy(*h);
            hier.add(take());
            with_chunk(t, [&](ChunkLoadClass& c) { am->HTreeManager.Load_Tree(c); return true; });
        }
    }
    for (const Entry& e : all) {
        if (ends(e.name, ".w3d")) {
            const Bytes b = read_entry(root, e);
            for (const Top& t : tops(b)) {
                if (t.id == 0x200 || t.id == 0x280) {
                    std::unique_ptr<HAnimClass> a = load_anim(t);
                    if (!a) { ++anim.failed; continue; }
                    // The hierarchy name is the part of the anim name before the dot.
                    const std::string full = a->Get_Name();
                    HTreeClass* tree = am->Get_HTree(a->Get_HName());
                    if (!tree) { ++anim.failed; continue; }
                    HTreeClass copy(*tree);
                    text(full.c_str());
                    dump_anim(copy, *a, sample_frames(a->Get_Num_Frames()));
                    anim.add(take());
                } else if (t.id == 0) {
                    MeshGeometryClass* m = new MeshGeometryClass;
                    if (with_chunk(t, [&](ChunkLoadClass& c) { return m->Load_W3D(c) == WW3D_ERROR_OK; })) {
                        dump_mesh(*m);
                        mesh.add(take());
                    } else {
                        ++mesh.failed;
                    }
                    m->Release_Ref();
                } else if (t.id == 0x700) {
                    HLodDefClass d;
                    if (with_chunk(t, [&](ChunkLoadClass& c) { return d.Load_W3D(c) == WW3D_ERROR_OK; })) {
                        dump_hlod(d);
                        hlod.add(take());
                    } else {
                        ++hlod.failed;
                    }
                }
            }
        } else if (ends(e.name, ".tga")) {
            g_files["retail.tga"] = read_entry(root, e);
            dump_tga("retail.tga");
            const std::string s = take();
            if (s == "error;") ++tga.failed; else tga.add(s);
        } else if (ends(e.name, ".dds")) {
            g_files["retail.dds"] = read_entry(root, e);
            dump_dds("retail.tga");
            const std::string s = take();
            if (s == "error;") ++dds.failed; else dds.add(s);
        } else if (ends(e.name, ".csf")) {
            g_files["retail.csf"] = read_entry(root, e);
            dump_csf("retail.csf");
            csf.add(take());
        }
    }
    hier.write(f, "hierarchy");
    anim.write(f, "animation");
    mesh.write(f, "mesh");
    hlod.write(f, "hlod");
    tga.write(f, "tga");
    dds.write(f, "dds");
    csf.write(f, "csf", true);
}

int main(int argc, char** argv) {
    if (argc < 2) return 2;
    oracle::init_memory();
    TheNameKeyGenerator = NEW NameKeyGenerator;
    TheNameKeyGenerator->init();
    TheFileSystem = NEW FileSystem;
    TheLocalFileSystem = NEW MemFileSystem;
    static MemFactory factory;
    _TheFileFactory = &factory;
    WWMath::Init();
    // validateCachedBones only runs inside a logic update.
    TheGameLogic = NEW GameLogic;
    TheGameLogic->m_isInUpdate = TRUE;
    new W3DAssetManager;

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* f = fopen(tmp.c_str(), "wb");
    if (!f) return 2;
    fprintf(f, "{\"vectors\":[\n");
    for (int i = 0; i < 36; ++i) model_case(i, f);
    for (int i = 0; i < 20; ++i) mesh_case(i, f);
    for (int i = 0; i < 40; ++i) tga_case(i, f);
    for (int i = 0; i < 12; ++i) dds_case(i, f);
    for (int i = 0; i < 25; ++i) csf_case(i, f);
    for (int i = 0; i < 160; ++i) slerp_case(i, f);
    retail(f);
    fprintf(f, "]}\n");
    fclose(f);
    if (rename(tmp.c_str(), argv[1]) != 0) return 2;
    fflush(stdout);
    _exit(0);  // engine singletons are not torn down
}
