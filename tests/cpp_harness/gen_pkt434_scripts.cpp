// PKT-434 (Scripts, ScriptList, ScriptGroup, Script, OrCondition, Condition, ScriptAction, Parameter, Template)
// (WP-6.4) C++ Golden Vector Generator.
// Compiles unmodified vendor/GeneralsX Scripts.cpp + DataChunk.cpp + Dict.cpp + NameKeyGenerator.cpp
// + KindOf.cpp + ObjectStatusTypes.cpp + BitFlags.cpp + Xfer*.cpp + crc.cpp.
//
// Canonical build & run command (macOS arm64, Release-aligned macros):
//   clang++ -std=c++17 -O1 -ffp-contract=off -URTS_DEBUG -U_ALLOW_DEBUG_CHEATS_IN_RELEASE \
//     -DRTS_ZEROHOUR=1 -DRTS_GENERALS=0 -DRETAIL_COMPATIBLE_CRC=0 -DRETAIL_COMPATIBLE_XFER_SAVE=1 \
//     -Wno-macro-redefined -Wno-switch \
//     -Itests/cpp_harness/stubs_pkt434 \
//     -Ivendor/GeneralsX/GeneralsMD/Code/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/Libraries/Source/Compression \
//     tests/cpp_harness/gen_pkt434_scripts.cpp \
//     -o /tmp/gen_pkt434 && /tmp/gen_pkt434 docs/golden_vectors/PKT-434.json
//
// Determinism & UB verification builds (must produce byte-identical JSON):
//   -O0, -O1, -O2, and -O1 -g -fsanitize=address,undefined -fno-sanitize=alignment,enum.

#include "PreRTS.h"
#include "Compression.h"
#include "Common/crc.h"
#include "Common/Dict.h"
#include "Common/DataChunk.h"
#include "Common/Snapshot.h"
#include "Common/Xfer.h"
#include "Common/XferCRC.h"
#include "Common/XferSave.h"
#include "Common/XferLoad.h"
#include "Common/KindOf.h"
#include "Common/ObjectStatusTypes.h"
#include "Common/Science.h"
#include "Common/Upgrade.h"
#include "Common/BorderColors.h"
#include "GameClient/ShellHooks.h"
#include "GameClient/View.h"
#include "GameLogic/Scripts.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/SidesList.h"

#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/NameKeyGenerator.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/Dict.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/DataChunk.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/KindOf.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/ObjectStatusTypes.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/BitFlags.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/Xfer.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferCRC.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferSave.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferLoad.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/crc.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/ScriptEngine/Scripts.cpp"

const AsciiString AsciiString::TheEmptyString("");
const UnicodeString UnicodeString::TheEmptyString;

static DynamicMemoryAllocator g_dma;
DynamicMemoryAllocator* TheDynamicMemoryAllocator = &g_dma;

static GlobalData g_global_data;
GlobalData* TheGlobalData = &g_global_data;

static GameState g_game_state;
GameState* TheGameState = &g_game_state;

static FileSystem g_fs;
FileSystem* TheFileSystem = &g_fs;

static ScienceStoreStub g_science_store;
ScienceStoreStub* TheScienceStore = &g_science_store;

static UpgradeCenterStub g_upgrade_center;
UpgradeCenterStub* TheUpgradeCenter = &g_upgrade_center;

static SidesListStub g_sides_list;
SidesListStub* TheSidesList = &g_sides_list;

static ScriptEngineStub g_script_engine;
ScriptEngineStub* TheScriptEngine = &g_script_engine;

Bool CompressionManager::isDataCompressed(const void*, Int) { return FALSE; }
Int CompressionManager::getUncompressedSize(const void*, Int len) { return len; }
Int CompressionManager::decompressData(void*, Int, void*, Int) { return 0; }

class MemOutputStream : public OutputStream {
public:
    std::vector<uint8_t> bytes;
    Int write(const void* pData, Int numBytes) override {
        if (numBytes <= 0 || !pData) return 0;
        const uint8_t* p = static_cast<const uint8_t*>(pData);
        bytes.insert(bytes.end(), p, p + numBytes);
        return numBytes;
    }
};

class SliceChunkInputStream : public ChunkInputStream {
    std::vector<uint8_t> m_buf;
    UnsignedInt m_pos = 0;
public:
    explicit SliceChunkInputStream(std::vector<uint8_t> buf) : m_buf(std::move(buf)) {}
    Int read(void* pData, Int numBytes) override {
        if (!pData || numBytes <= 0 || m_pos >= m_buf.size()) return 0;
        size_t avail = m_buf.size() - m_pos;
        size_t take = std::min(avail, static_cast<size_t>(numBytes));
        std::memcpy(pData, m_buf.data() + m_pos, take);
        m_pos += static_cast<UnsignedInt>(take);
        return static_cast<Int>(take);
    }
    UnsignedInt tell() override { return m_pos; }
    Bool absoluteSeek(UnsignedInt pos) override {
        if (pos > m_buf.size()) return FALSE;
        m_pos = pos;
        return TRUE;
    }
    Bool eof() override { return m_pos >= m_buf.size() ? TRUE : FALSE; }
};

static std::string sha256_file(const char* path) {
    static const uint32_t K[64] = {
        0x428a2f98u, 0x71374491u, 0xb5c0fbcfu, 0xe9b5dba5u, 0x3956c25bu, 0x59f111f1u, 0x923f82a4u, 0xab1c5ed5u,
        0xd807aa98u, 0x12835b01u, 0x243185beu, 0x550c7dc3u, 0x72be5d74u, 0x80deb1feu, 0x9bdc06a7u, 0xc19bf174u,
        0xe49b69c1u, 0xefbe4786u, 0x0fc19dc6u, 0x240ca1ccu, 0x2de92c6fu, 0x4a7484aau, 0x5cb0a9dcu, 0x76f988dau,
        0x983e5152u, 0xa831c66du, 0xb00327c8u, 0xbf597fc7u, 0xc6e00bf3u, 0xd5a79147u, 0x06ca6351u, 0x14292967u,
        0x27b70a85u, 0x2e1b2138u, 0x4d2c6dfcu, 0x53380d13u, 0x650a7354u, 0x766a0abbu, 0x81c2c92eu, 0x92722c85u,
        0xa2bfe8a1u, 0xa81a664bu, 0xc24b8b70u, 0xc76c51a3u, 0xd192e819u, 0xd6990624u, 0xf40e3585u, 0x106aa070u,
        0x19a4c116u, 0x1e376c08u, 0x2748774cu, 0x34b0bcb5u, 0x391c0cb3u, 0x4ed8aa4au, 0x5b9cca4fu, 0x682e6ff3u,
        0x748f82eeu, 0x78a5636fu, 0x84c87814u, 0x8cc70208u, 0x90befffau, 0xa4506cebu, 0xbef9a3f7u, 0xc67178f2u
    };
    FILE* f = std::fopen(path, "rb");
    if (!f) { std::perror(path); std::exit(1); }
    std::vector<uint8_t> data;
    std::fseek(f, 0, SEEK_END);
    long sz = std::ftell(f);
    std::fseek(f, 0, SEEK_SET);
    if (sz > 0) {
        data.resize(static_cast<size_t>(sz));
        if (std::fread(data.data(), 1, static_cast<size_t>(sz), f) != static_cast<size_t>(sz)) {
            std::perror("fread");
            std::exit(1);
        }
    }
    std::fclose(f);
    uint64_t bit_len = static_cast<uint64_t>(data.size()) * 8ULL;
    data.push_back(0x80u);
    while ((data.size() % 64u) != 56u) data.push_back(0x00u);
    for (int shift = 56; shift >= 0; shift -= 8) {
        data.push_back(static_cast<uint8_t>((bit_len >> shift) & 0xFFu));
    }
    uint32_t H[8] = {
        0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
        0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u
    };
    auto rotr = [](uint32_t x, uint32_t n) -> uint32_t { return (x >> n) | (x << (32u - n)); };
    for (size_t off = 0; off < data.size(); off += 64u) {
        uint32_t w[64]{};
        for (int i = 0; i < 16; ++i) {
            w[i] = (static_cast<uint32_t>(data[off + i * 4]) << 24) |
                   (static_cast<uint32_t>(data[off + i * 4 + 1]) << 16) |
                   (static_cast<uint32_t>(data[off + i * 4 + 2]) << 8) |
                   static_cast<uint32_t>(data[off + i * 4 + 3]);
        }
        for (int i = 16; i < 64; ++i) {
            uint32_t s0 = rotr(w[i - 15], 7) ^ rotr(w[i - 15], 18) ^ (w[i - 15] >> 3);
            uint32_t s1 = rotr(w[i - 2], 17) ^ rotr(w[i - 2], 19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16] + s0 + w[i - 7] + s1;
        }
        uint32_t a = H[0], b = H[1], c = H[2], d = H[3];
        uint32_t e = H[4], f0 = H[5], g = H[6], h = H[7];
        for (int i = 0; i < 64; ++i) {
            uint32_t S1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
            uint32_t ch = (e & f0) ^ ((~e) & g);
            uint32_t temp1 = h + S1 + ch + K[i] + w[i];
            uint32_t S0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
            uint32_t maj = (a & b) ^ (a & c) ^ (b & c);
            uint32_t temp2 = S0 + maj;
            h = g; g = f0; f0 = e; e = d + temp1;
            d = c; c = b; b = a; a = temp1 + temp2;
        }
        H[0] += a; H[1] += b; H[2] += c; H[3] += d;
        H[4] += e; H[5] += f0; H[6] += g; H[7] += h;
    }
    char hex[65];
    for (int i = 0; i < 8; ++i) std::snprintf(hex + i * 8, 9, "%08x", H[i]);
    hex[64] = '\0';
    return std::string(hex);
}

static uint32_t splitmix32(uint32_t& s) {
    uint32_t z = (s += 0x9e3779b9u);
    z = (z ^ (z >> 16)) * 0x85ebca6bu;
    z = (z ^ (z >> 13)) * 0xc2b2ae35u;
    return z ^ (z >> 16);
}

static std::vector<uint8_t> read_file_bytes(const char* path) {
    FILE* fp = std::fopen(path, "rb");
    if (!fp) { std::perror(path); std::exit(1); }
    std::fseek(fp, 0, SEEK_END);
    long sz = std::ftell(fp);
    std::fseek(fp, 0, SEEK_SET);
    std::vector<uint8_t> buf(static_cast<size_t>(sz));
    if (sz > 0 && std::fread(buf.data(), 1, buf.size(), fp) != buf.size()) {
        std::perror("fread");
        std::exit(1);
    }
    std::fclose(fp);
    return buf;
}

static std::string hex_encode(const std::vector<uint8_t>& bytes) {
    static const char* kHex = "0123456789abcdef";
    std::string out;
    out.reserve(bytes.size() * 2);
    for (uint8_t b : bytes) {
        out.push_back(kHex[b >> 4]);
        out.push_back(kHex[b & 0x0f]);
    }
    return out;
}

static uint32_t crc_str(const char* s) {
    if (!s) return 0;
    CRC c;
    c.computeCRC(s, static_cast<Int>(std::strlen(s)));
    return c.get();
}

static uint32_t rot_crc(uint32_t acc, uint32_t v) {
    return ((acc << 1) | (acc >> 31)) + v;
}

static uint32_t f32_bits(Real v) {
    uint32_t u = 0;
    std::memcpy(&u, &v, 4);
    return u;
}

// Deterministic template table setup used across both C++ oracle and Rust test harness.
// Sets up all Condition::NUM_ITEMS (113) and ScriptAction::NUM_ITEMS (345) templates with
// deterministic internalNameKey, parameter counts/types, and UI strings, including exact
// parameter counts for all legacy healing cases in Scripts.cpp!
static void init_deterministic_templates() {
    for (Int i = 0; i < Condition::NUM_ITEMS; ++i) {
        ConditionTemplate& ct = TheScriptEngine->m_condTemplates[i];
        char iname[64], u0[32], u1[32], u2[32];
        std::snprintf(iname, sizeof(iname), "COND_INTERNAL_%d", i);
        std::snprintf(u0, sizeof(u0), "C%d(", i);
        std::snprintf(u1, sizeof(u1), ",");
        std::snprintf(u2, sizeof(u2), ")");
        ct.m_internalName = iname;
        ct.m_internalNameKey = NAMEKEY(ct.m_internalName);
        ct.m_uiName = iname;

        if (i == Condition::CONDITION_FALSE || i == Condition::CONDITION_TRUE) {
            ct.m_numParameters = 0;
            ct.m_numUiStrings = 1;
            ct.m_uiStrings[0] = (i == Condition::CONDITION_FALSE) ? "FALSE_COND" : "TRUE_COND";
        } else if (i == Condition::TEAM_INSIDE_AREA_PARTIALLY ||
                   i == Condition::TEAM_INSIDE_AREA_ENTIRELY ||
                   i == Condition::TEAM_OUTSIDE_AREA_ENTIRELY ||
                   i == Condition::TEAM_ENTERED_AREA_PARTIALLY ||
                   i == Condition::TEAM_ENTERED_AREA_ENTIRELY ||
                   i == Condition::TEAM_EXITED_AREA_ENTIRELY ||
                   i == Condition::TEAM_EXITED_AREA_PARTIALLY) {
            ct.m_numParameters = 3;
            ct.m_parameters[0] = Parameter::TEAM;
            ct.m_parameters[1] = Parameter::TRIGGER_AREA;
            ct.m_parameters[2] = Parameter::SURFACES_ALLOWED;
            ct.m_numUiStrings = 3;
            ct.m_uiStrings[0] = u0; ct.m_uiStrings[1] = u1; ct.m_uiStrings[2] = u2;
        } else if (i == Condition::SKIRMISH_SPECIAL_POWER_READY) {
            ct.m_numParameters = 2;
            ct.m_parameters[0] = Parameter::SIDE;
            ct.m_parameters[1] = Parameter::SPECIAL_POWER;
            ct.m_numUiStrings = 3;
            ct.m_uiStrings[0] = u0; ct.m_uiStrings[1] = u1; ct.m_uiStrings[2] = u2;
        } else {
            Int np = 1 + (i % 3);
            ct.m_numParameters = np;
            for (Int p = 0; p < np; ++p) {
                ct.m_parameters[p] = static_cast<Parameter::ParameterType>((i * 7 + p * 13) % Parameter::NUM_ITEMS);
            }
            ct.m_numUiStrings = np;
            ct.m_uiStrings[0] = u0;
            if (np >= 2) ct.m_uiStrings[1] = u1;
            if (np >= 3) ct.m_uiStrings[2] = u2;
        }
    }

    for (Int i = 0; i < ScriptAction::NUM_ITEMS; ++i) {
        ActionTemplate& at = TheScriptEngine->m_actTemplates[i];
        char iname[64], u0[32], u1[32], u2[32];
        std::snprintf(iname, sizeof(iname), "ACT_INTERNAL_%d", i);
        std::snprintf(u0, sizeof(u0), "A%d[", i);
        std::snprintf(u1, sizeof(u1), ";");
        std::snprintf(u2, sizeof(u2), "]");
        at.m_internalName = iname;
        at.m_internalNameKey = NAMEKEY(at.m_internalName);
        at.m_uiName = iname;

        switch (i) {
            case ScriptAction::NO_OP:
            case ScriptAction::SKIRMISH_BUILD_BASE_DEFENSE_FRONT:
            case ScriptAction::SKIRMISH_BUILD_BASE_DEFENSE_FLANK:
                at.m_numParameters = 0;
                at.m_numUiStrings = 1;
                at.m_uiStrings[0] = u0;
                break;
            case ScriptAction::MAP_REVEAL_ALL:
            case ScriptAction::MAP_REVEAL_ALL_PERM:
            case ScriptAction::MAP_REVEAL_ALL_UNDO_PERM:
            case ScriptAction::MAP_SHROUD_ALL:
                at.m_numParameters = 1;
                at.m_parameters[0] = Parameter::SIDE;
                at.m_numUiStrings = 1;
                at.m_uiStrings[0] = u0;
                break;
            case ScriptAction::SKIRMISH_FIRE_SPECIAL_POWER_AT_MOST_COST:
                at.m_numParameters = 2;
                at.m_parameters[0] = Parameter::SIDE;
                at.m_parameters[1] = Parameter::SPECIAL_POWER;
                at.m_numUiStrings = 2;
                at.m_uiStrings[0] = u0; at.m_uiStrings[1] = u2;
                break;
            case ScriptAction::NAMED_SET_ATTITUDE:
            case ScriptAction::TEAM_SET_ATTITUDE:
                at.m_numParameters = 2;
                at.m_parameters[0] = (i == ScriptAction::NAMED_SET_ATTITUDE) ? Parameter::UNIT : Parameter::TEAM;
                at.m_parameters[1] = Parameter::AI_MOOD;
                at.m_numUiStrings = 2;
                at.m_uiStrings[0] = u0; at.m_uiStrings[1] = u2;
                break;
            case ScriptAction::SPEECH_PLAY:
                at.m_numParameters = 2;
                at.m_parameters[0] = Parameter::DIALOG;
                at.m_parameters[1] = Parameter::BOOLEAN;
                at.m_numUiStrings = 2;
                at.m_uiStrings[0] = u0; at.m_uiStrings[1] = u2;
                break;
            case ScriptAction::TEAM_FOLLOW_WAYPOINTS:
                at.m_numParameters = 3;
                at.m_parameters[0] = Parameter::TEAM;
                at.m_parameters[1] = Parameter::WAYPOINT_PATH;
                at.m_parameters[2] = Parameter::BOOLEAN;
                at.m_numUiStrings = 3;
                at.m_uiStrings[0] = u0; at.m_uiStrings[1] = u1; at.m_uiStrings[2] = u2;
                break;
            case ScriptAction::MAP_REVEAL_AT_WAYPOINT:
            case ScriptAction::MAP_SHROUD_AT_WAYPOINT:
                at.m_numParameters = 3;
                at.m_parameters[0] = Parameter::WAYPOINT;
                at.m_parameters[1] = Parameter::REAL;
                at.m_parameters[2] = Parameter::SIDE;
                at.m_numUiStrings = 3;
                at.m_uiStrings[0] = u0; at.m_uiStrings[1] = u1; at.m_uiStrings[2] = u2;
                break;
            case ScriptAction::CAMERA_MOD_SET_FINAL_ZOOM:
            case ScriptAction::CAMERA_MOD_SET_FINAL_PITCH:
                at.m_numParameters = 3;
                at.m_parameters[0] = Parameter::REAL;
                at.m_parameters[1] = Parameter::PERCENT;
                at.m_parameters[2] = Parameter::PERCENT;
                at.m_numUiStrings = 3;
                at.m_uiStrings[0] = u0; at.m_uiStrings[1] = u1; at.m_uiStrings[2] = u2;
                break;
            case ScriptAction::RESET_CAMERA:
            case ScriptAction::ZOOM_CAMERA:
            case ScriptAction::PITCH_CAMERA:
            case ScriptAction::ROTATE_CAMERA:
                at.m_numParameters = 4;
                at.m_parameters[0] = Parameter::WAYPOINT;
                at.m_parameters[1] = Parameter::REAL;
                at.m_parameters[2] = Parameter::REAL;
                at.m_parameters[3] = Parameter::REAL;
                at.m_numUiStrings = 3;
                at.m_uiStrings[0] = u0; at.m_uiStrings[1] = u1; at.m_uiStrings[2] = u2;
                break;
            case ScriptAction::MOVE_CAMERA_TO:
            case ScriptAction::MOVE_CAMERA_ALONG_WAYPOINT_PATH:
            case ScriptAction::CAMERA_LOOK_TOWARD_OBJECT:
            case ScriptAction::CAMERA_LOOK_TOWARD_WAYPOINT:
                at.m_numParameters = 5;
                at.m_parameters[0] = Parameter::WAYPOINT;
                at.m_parameters[1] = Parameter::REAL;
                at.m_parameters[2] = Parameter::REAL;
                at.m_parameters[3] = Parameter::REAL;
                at.m_parameters[4] = (i == ScriptAction::CAMERA_LOOK_TOWARD_WAYPOINT) ? Parameter::BOOLEAN : Parameter::REAL;
                at.m_numUiStrings = 3;
                at.m_uiStrings[0] = u0; at.m_uiStrings[1] = u1; at.m_uiStrings[2] = u2;
                break;
            default: {
                Int np = 1 + (i % 3);
                at.m_numParameters = np;
                for (Int p = 0; p < np; ++p) {
                    at.m_parameters[p] = static_cast<Parameter::ParameterType>((i * 5 + p * 11) % Parameter::NUM_ITEMS);
                }
                at.m_numUiStrings = np;
                at.m_uiStrings[0] = u0;
                if (np >= 2) at.m_uiStrings[1] = u1;
                if (np >= 3) at.m_uiStrings[2] = u2;
                break;
            }
        }
    }
}

static void populate_parameter(Parameter* p, Int ptype_idx, uint32_t& rng) {
    Parameter::ParameterType pt = p->getParameterType();
    uint32_t r = splitmix32(rng);
    switch (pt) {
        case Parameter::COORD3D: {
            Coord3D c;
            c.x = static_cast<Real>(static_cast<Int>(r % 2001u) - 1000) * 0.25f;
            c.y = static_cast<Real>(static_cast<Int>((r >> 11) % 2001u) - 1000) * 0.25f;
            c.z = static_cast<Real>(static_cast<Int>((r >> 21) % 401u) - 200) * 0.5f;
            p->friend_setCoord3D(&c);
            break;
        }
        case Parameter::REAL:
        case Parameter::ANGLE:
        case Parameter::PERCENT: {
            Real val = static_cast<Real>(static_cast<Int>(r % 1001u) - 500) * 0.125f;
            p->friend_setReal(val);
            p->friend_setInt(0);
            break;
        }
        case Parameter::INT:
        case Parameter::COLOR: {
            p->friend_setInt(static_cast<Int>(r & 0x00FFFFFFu));
            break;
        }
        case Parameter::BOOLEAN: {
            p->friend_setInt(static_cast<Int>(r & 1u));
            break;
        }
        case Parameter::COMPARISON: {
            p->friend_setInt(static_cast<Int>(r % 6u));
            break;
        }
        case Parameter::RELATION: {
            p->friend_setInt(static_cast<Int>(r % 3u));
            break;
        }
        case Parameter::AI_MOOD: {
            p->friend_setInt(static_cast<Int>(r % 5u) - 2);
            break;
        }
        case Parameter::RADAR_EVENT_TYPE: {
            static const Int kRadarEvents[5] = {
                RADAR_EVENT_CONSTRUCTION, RADAR_EVENT_UPGRADE, RADAR_EVENT_UNDER_ATTACK,
                RADAR_EVENT_INFORMATION, RADAR_EVENT_INFILTRATION
            };
            p->friend_setInt(kRadarEvents[r % 5u]);
            break;
        }
        case Parameter::LEFT_OR_RIGHT: {
            p->friend_setInt(static_cast<Int>(r % 4u));
            break;
        }
        case Parameter::BOUNDARY: {
            p->friend_setInt(static_cast<Int>(r % 16u));
            break;
        }
        case Parameter::BUILDABLE: {
            p->friend_setInt(static_cast<Int>(r % 5u) - 1);
            break;
        }
        case Parameter::SURFACES_ALLOWED: {
            p->friend_setInt(static_cast<Int>(r % 5u));
            break;
        }
        case Parameter::SHAKE_INTENSITY: {
            p->friend_setInt(static_cast<Int>(r % 7u));
            break;
        }
        case Parameter::KIND_OF_PARAM: {
            p->friend_setInt(static_cast<Int>(r % KINDOF_COUNT));
            p->friend_setString(KindOfMaskType::getNameFromSingleBit(p->getInt()));
            break;
        }
        case Parameter::OBJECT_STATUS: {
            if ((r % 7u) == 0) {
                p->friend_setString("");
            } else {
                Int bit = static_cast<Int>(r % OBJECT_STATUS_COUNT);
                p->friend_setString(ObjectStatusMaskType::getBitNames()[bit]);
            }
            break;
        }
        case Parameter::OBJECT_TYPE: {
            static const char* kObjTypes[4] = {
                "AmericaTankCrusader", "FundamentalistTankScorpion", "ChinaTankOverlord", "GLATankScorpion"
            };
            p->friend_setString(kObjTypes[(r + static_cast<uint32_t>(ptype_idx)) % 4u]);
            break;
        }
        case Parameter::UPGRADE: {
            static const char* kUpgrades[5] = {
                "Upgrade_AmericaRangerCaptureBuilding",
                "Upgrade_ChinaRedguardCaptureBuilding",
                "Upgrade_GLARebelCaptureBuilding",
                "Upgrade_InfantryCaptureBuilding",
                "Upgrade_ChinaNuclearTanks"
            };
            p->friend_setString(kUpgrades[r % 5u]);
            break;
        }
        case Parameter::SIDE: {
            static const char* kSides[4] = { "PlyrAmerica", "PlyrChina", "PlyrGLA", THIS_PLAYER };
            p->friend_setString(kSides[r % 4u]);
            break;
        }
        case Parameter::TEAM: {
            static const char* kTeams[4] = { THIS_TEAM, ANY_TEAM, "teamPlyrAmerica", "AttackTeam1" };
            p->friend_setString(kTeams[r % 4u]);
            break;
        }
        default: {
            if ((r % 9u) == 0) {
                p->friend_setString("");
            } else {
                char buf[32];
                std::snprintf(buf, sizeof(buf), "P_%d_%u", static_cast<int>(pt), r % 100u);
                p->friend_setString(buf);
            }
            break;
        }
    }
}

static uint32_t hash_script_list_tree(ScriptList* list) {
    if (!list) return 0;
    uint32_t h = 0x811c9dc5u;
    auto hash_script = [&](Script* scr) {
        for (Script* s = scr; s; s = s->getNext()) {
            h = rot_crc(h, crc_str(s->getName().str()));
            h = rot_crc(h, crc_str(s->getComment().str()));
            h = rot_crc(h, crc_str(s->getConditionComment().str()));
            h = rot_crc(h, crc_str(s->getActionComment().str()));
            h = rot_crc(h, s->isActive());
            h = rot_crc(h, s->isOneShot());
            h = rot_crc(h, s->isEasy());
            h = rot_crc(h, s->isNormal());
            h = rot_crc(h, s->isHard());
            h = rot_crc(h, s->isSubroutine());
            h = rot_crc(h, static_cast<uint32_t>(s->getDelayEvalSeconds()));
            h = rot_crc(h, crc_str(s->getUiText().str()));
            for (OrCondition* orc = s->getOrCondition(); orc; orc = orc->getNextOrCondition()) {
                for (Condition* c = orc->getFirstAndCondition(); c; c = c->getNext()) {
                    h = rot_crc(h, static_cast<uint32_t>(c->getConditionType()));
                    h = rot_crc(h, static_cast<uint32_t>(c->getNumParameters()));
                    for (Int pi = 0; pi < c->getNumParameters(); ++pi) {
                        Parameter* pm = c->getParameter(pi);
                        h = rot_crc(h, static_cast<uint32_t>(pm->getParameterType()));
                        h = rot_crc(h, static_cast<uint32_t>(pm->getInt()));
                        h = rot_crc(h, f32_bits(pm->getReal()));
                        h = rot_crc(h, crc_str(pm->getString().str()));
                        if (pm->getParameterType() == Parameter::COORD3D) {
                            Coord3D pos;
                            pm->getCoord3D(&pos);
                            h = rot_crc(h, f32_bits(pos.x));
                            h = rot_crc(h, f32_bits(pos.y));
                            h = rot_crc(h, f32_bits(pos.z));
                        }
                        if (pm->getParameterType() == Parameter::OBJECT_STATUS) {
                            for (Int bi = 0; bi < OBJECT_STATUS_COUNT; ++bi) {
                                if (pm->getStatus().test(bi)) {
                                    h = rot_crc(h, static_cast<uint32_t>(bi + 100));
                                }
                            }
                        }
                    }
                }
            }
            for (int branch = 0; branch < 2; ++branch) {
                ScriptAction* head = (branch == 0) ? s->getAction() : s->getFalseAction();
                for (ScriptAction* a = head; a; a = a->getNext()) {
                    h = rot_crc(h, static_cast<uint32_t>(a->getActionType()));
                    h = rot_crc(h, static_cast<uint32_t>(a->getNumParameters()));
                    for (Int pi = 0; pi < a->getNumParameters(); ++pi) {
                        Parameter* pm = a->getParameter(pi);
                        h = rot_crc(h, static_cast<uint32_t>(pm->getParameterType()));
                        h = rot_crc(h, static_cast<uint32_t>(pm->getInt()));
                        h = rot_crc(h, f32_bits(pm->getReal()));
                        h = rot_crc(h, crc_str(pm->getString().str()));
                    }
                }
            }
        }
    };
    hash_script(list->getScript());
    for (ScriptGroup* g = list->getScriptGroup(); g; g = g->getNext()) {
        h = rot_crc(h, crc_str(g->getName().str()));
        h = rot_crc(h, g->isActive());
        h = rot_crc(h, g->isSubroutine());
        hash_script(g->getScript());
    }
    return h;
}

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "Usage: %s <output.json>\n", argv[0]);
        return 1;
    }

    const char* source_paths[4] = {
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Scripts.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/ScriptEngine/Scripts.cpp",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BorderColors.h",
        "vendor/GeneralsX/Core/GameEngine/Include/GameClient/ShellHooks.h",
    };

    const char* tmp_save = "/tmp/pkt434_scripts_save.bin";
    FILE* out = std::fopen(argv[1], "wb");
    if (!out) { std::perror(argv[1]); return 1; }

    std::fprintf(out, "{\n");
    std::fprintf(out, "  \"packet_id\": \"PKT-434\",\n");
    std::fprintf(out, "  \"work_package\": \"WP-6.4\",\n");
    std::fprintf(out, "  \"reference_commit\": \"84a7a20769aa5749f388b1f49bd542c79b3c33e8\",\n");
    std::fprintf(out, "  \"vector_count\": 1000,\n");
    std::fprintf(out, "  \"constants\": {\n");
    std::fprintf(out, "    \"MAX_PARMS\": %d,\n", MAX_PARMS);
    std::fprintf(out, "    \"PARAMETER_NUM_ITEMS\": %d,\n", Parameter::NUM_ITEMS);
    std::fprintf(out, "    \"CONDITION_NUM_ITEMS\": %d,\n", Condition::NUM_ITEMS);
    std::fprintf(out, "    \"ACTION_NUM_ITEMS\": %d,\n", ScriptAction::NUM_ITEMS);
    std::fprintf(out, "    \"SHELL_SCRIPT_HOOK_TOTAL\": %d,\n", SHELL_SCRIPT_HOOK_TOTAL);
    std::fprintf(out, "    \"BORDER_COLORS_SIZE\": %ld\n", BORDER_COLORS_SIZE);
    std::fprintf(out, "  },\n");
    std::fprintf(out, "  \"source_sha256\": {\n");
    for (size_t i = 0; i < 4; ++i) {
        std::string sha = sha256_file(source_paths[i]);
        std::fprintf(out, "    \"%s\": \"%s\"%s\n", source_paths[i], sha.c_str(), i + 1 < 4 ? "," : "");
    }
    std::fprintf(out, "  },\n");
    std::fprintf(out, "  \"vectors\": [\n");

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x74779640u + 0x9e3779b9u;

        delete TheNameKeyGenerator;
        TheNameKeyGenerator = new NameKeyGenerator();
        TheNameKeyGenerator->init();
        init_deterministic_templates();

        // 1. Test Parameter::getUiText across every ParameterType (all 52 variants covered across seeds)
        Parameter::ParameterType sweep_pt = static_cast<Parameter::ParameterType>((seed + (splitmix32(rng) % 7u)) % Parameter::NUM_ITEMS);
        Parameter sweep_param(sweep_pt);
        populate_parameter(&sweep_param, static_cast<Int>(seed), rng);
        AsciiString sweep_ui = sweep_param.getUiText();
        uint32_t param_ui_crc = crc_str(sweep_ui.str());

        // 2. Build 1 or 2 ScriptLists with top-level Scripts and ScriptGroups
        Int num_lists = 1 + static_cast<Int>(seed % 2);
        ScriptList* lists[2] = { nullptr, nullptr };
        for (Int li = 0; li < num_lists; ++li) {
            ScriptList* sl = newInstance(ScriptList);
            lists[li] = sl;

            Int num_top_scripts = 1 + static_cast<Int>(splitmix32(rng) % 2u);
            for (Int si = 0; si < num_top_scripts; ++si) {
                Script* scr = newInstance(Script);
                char sname[32], scomm[32];
                std::snprintf(sname, sizeof(sname), "Scr_%u_%d_%d", seed, li, si);
                std::snprintf(scomm, sizeof(scomm), "Cmt_%u_%d", seed, si);
                scr->setName(sname);
                scr->setComment(scomm);
                scr->setConditionComment("CondCmt");
                scr->setActionComment("ActCmt");
                scr->setActive((splitmix32(rng) & 1u) ? TRUE : FALSE);
                scr->setOneShot((splitmix32(rng) & 1u) ? TRUE : FALSE);
                scr->setEasy((splitmix32(rng) & 1u) ? TRUE : FALSE);
                scr->setNormal((splitmix32(rng) & 1u) ? TRUE : FALSE);
                scr->setHard((splitmix32(rng) & 1u) ? TRUE : FALSE);
                scr->setSubroutine((splitmix32(rng) & 1u) ? TRUE : FALSE);
                scr->setDelayEvalSeconds(static_cast<Int>(splitmix32(rng) % 30u));
                scr->setFrameToEvaluate(splitmix32(rng) % 1000u);

                // Add 1 or 2 OrConditions, each with 1 or 2 Conditions
                Int num_ors = 1 + static_cast<Int>(splitmix32(rng) % 2u);
                OrCondition* prev_or = nullptr;
                for (Int oi = 0; oi < num_ors; ++oi) {
                    OrCondition* orc = newInstance(OrCondition);
                    if (prev_or) prev_or->setNextOrCondition(orc);
                    else scr->setOrCondition(orc);
                    prev_or = orc;

                    Int num_ands = 1 + static_cast<Int>(splitmix32(rng) % 2u);
                    Condition* prev_and = nullptr;
                    for (Int ai = 0; ai < num_ands; ++ai) {
                        Condition::ConditionType ct = static_cast<Condition::ConditionType>(
                            (seed * 11u + static_cast<uint32_t>(li * 7 + si * 5 + oi * 3 + ai)) % Condition::NUM_ITEMS
                        );
                        Condition* cond = newInstance(Condition)(ct);
                        for (Int pi = 0; pi < cond->getNumParameters(); ++pi) {
                            populate_parameter(cond->getParameter(pi), pi, rng);
                        }
                        if (prev_and) prev_and->setNextCondition(cond);
                        else orc->setFirstAndCondition(cond);
                        prev_and = cond;
                    }
                }

                // Add 1 or 2 True Actions
                Int num_acts = 1 + static_cast<Int>(splitmix32(rng) % 2u);
                ScriptAction* prev_act = nullptr;
                for (Int ai = 0; ai < num_acts; ++ai) {
                    ScriptAction::ScriptActionType at = static_cast<ScriptAction::ScriptActionType>(
                        (seed * 17u + static_cast<uint32_t>(li * 13 + si * 7 + ai * 3)) % ScriptAction::NUM_ITEMS
                    );
                    ScriptAction* act = newInstance(ScriptAction)(at);
                    for (Int pi = 0; pi < act->getNumParameters(); ++pi) {
                        populate_parameter(act->getParameter(pi), pi, rng);
                    }
                    if (prev_act) prev_act->setNextAction(act);
                    else scr->setAction(act);
                    prev_act = act;
                }

                // Add 0 or 1 False Action
                if (splitmix32(rng) & 1u) {
                    ScriptAction::ScriptActionType fat = static_cast<ScriptAction::ScriptActionType>(
                        (seed * 23u + static_cast<uint32_t>(si + 1)) % ScriptAction::NUM_ITEMS
                    );
                    ScriptAction* fact = newInstance(ScriptAction)(fat);
                    for (Int pi = 0; pi < fact->getNumParameters(); ++pi) {
                        populate_parameter(fact->getParameter(pi), pi, rng);
                    }
                    scr->setFalseAction(fact);
                }

                Int insert_ndx = static_cast<Int>(splitmix32(rng) % 3u);
                sl->addScript(scr, insert_ndx);
            }

            // Add 1 ScriptGroup with 1 Script
            ScriptGroup* grp = newInstance(ScriptGroup);
            char gname[32];
            std::snprintf(gname, sizeof(gname), "Grp_%u_%d", seed, li);
            grp->setName(gname);
            grp->setActive((splitmix32(rng) & 1u) ? TRUE : FALSE);
            grp->setSubroutine((splitmix32(rng) & 1u) ? TRUE : FALSE);

            Script* gscr = newInstance(Script);
            char gsname[32];
            std::snprintf(gsname, sizeof(gsname), "GScr_%u_%d", seed, li);
            gscr->setName(gsname);
            gscr->setActive((splitmix32(rng) & 1u) ? TRUE : FALSE);
            gscr->setOneShot((splitmix32(rng) & 1u) ? TRUE : FALSE);

            OrCondition* gor = newInstance(OrCondition);
            Condition::ConditionType gct = static_cast<Condition::ConditionType>((seed * 31u + static_cast<uint32_t>(li)) % Condition::NUM_ITEMS);
            Condition* gcond = newInstance(Condition)(gct);
            for (Int pi = 0; pi < gcond->getNumParameters(); ++pi) {
                populate_parameter(gcond->getParameter(pi), pi, rng);
            }
            gor->setFirstAndCondition(gcond);
            gscr->setOrCondition(gor);

            ScriptAction::ScriptActionType gat = static_cast<ScriptAction::ScriptActionType>((seed * 37u + static_cast<uint32_t>(li)) % ScriptAction::NUM_ITEMS);
            ScriptAction* gact = newInstance(ScriptAction)(gat);
            for (Int pi = 0; pi < gact->getNumParameters(); ++pi) {
                populate_parameter(gact->getParameter(pi), pi, rng);
            }
            gscr->setAction(gact);

            grp->addScript(gscr, 0);
            sl->addGroup(grp, 0);
        }

        // 3. Serialize via WriteScriptsDataChunk
        MemOutputStream chunk_mem;
        {
            DataChunkOutput chunk_out(&chunk_mem);
            ScriptList::WriteScriptsDataChunk(chunk_out, lists, num_lists);
        }

        // 4. Parse back via DataChunkInput + ParseScriptsDataChunk + getReadScripts
        SliceChunkInputStream chunk_in_stream(chunk_mem.bytes);
        DataChunkInput chunk_in(&chunk_in_stream);
        chunk_in.registerParser("PlayerScriptsList", AsciiString::TheEmptyString, ScriptList::ParseScriptsDataChunk);
        Bool parsed_ok = chunk_in.parse(nullptr);
        if (!parsed_ok) {
            std::fprintf(stderr, "ParseScriptsDataChunk failed on seed %u\n", seed);
            return 1;
        }
        ScriptList* read_lists[MAX_PLAYER_COUNT]{};
        Int read_count = ScriptList::getReadScripts(read_lists);
        if (read_count != num_lists) {
            std::fprintf(stderr, "getReadScripts count mismatch on seed %u\n", seed);
            return 1;
        }

        uint32_t tree_hash_orig = hash_script_list_tree(lists[0]);
        uint32_t tree_hash_parsed = hash_script_list_tree(read_lists[0]);

        // Re-serialize parsed lists to verify 100% idempotent chunk output after healing/canonicalization
        MemOutputStream chunk_reparsed;
        {
            DataChunkOutput chunk_out2(&chunk_reparsed);
            ScriptList::WriteScriptsDataChunk(chunk_out2, read_lists, read_count);
        }

        // 5. duplicate() and duplicateAndQualify("_Q", "PlyrAmerica_Q", "Player2") on read_lists[0]
        ScriptList* dup_list = read_lists[0]->duplicate();
        uint32_t tree_hash_dup = hash_script_list_tree(dup_list);
        if (tree_hash_dup != tree_hash_parsed) {
            std::fprintf(stderr, "duplicate() tree hash mismatch on seed %u\n", seed);
            return 1;
        }

        ScriptList* qual_list = read_lists[0]->duplicateAndQualify("_Q", "PlyrAmerica_Q", "Player2");
        uint32_t tree_hash_qual = hash_script_list_tree(qual_list);

        // 6. Snapshot XferSave / XferCRC / XferLoad on read_lists[0] after mutating active states
        for (Script* s = read_lists[0]->getScript(); s; s = s->getNext()) {
            s->setActive(s->isActive() ? FALSE : TRUE);
        }
        for (ScriptGroup* g = read_lists[0]->getScriptGroup(); g; g = g->getNext()) {
            g->setActive(g->isActive() ? FALSE : TRUE);
            for (Script* s = g->getScript(); s; s = s->getNext()) {
                s->setActive(s->isActive() ? FALSE : TRUE);
            }
        }

        {
            XferSave xsave;
            xsave.open(tmp_save);
            xsave.xferSnapshot(read_lists[0]);
            xsave.close();
        }
        std::vector<uint8_t> xfer_bytes = read_file_bytes(tmp_save);

        XferCRC xcrc;
        xcrc.open("crc");
        xcrc.xferSnapshot(read_lists[0]);
        xcrc.close();
        UnsignedInt xfer_crc = xcrc.getCRC();

        // XferLoad into dup_list (which had pre-mutation active flags) and verify XferCRC matches
        {
            XferLoad xload;
            xload.open(tmp_save);
            xload.xferSnapshot(dup_list);
            xload.close();
        }
        if (hash_script_list_tree(dup_list) != hash_script_list_tree(read_lists[0])) {
            std::fprintf(stderr, "XferLoad active state mismatch on seed %u\n", seed);
            return 1;
        }

        std::fprintf(out,
            "    {\"seed\":%u,\"num_lists\":%d,\"sweep_pt\":%d,\"param_ui_crc\":%u,"
            "\"tree_hash_orig\":%u,\"tree_hash_parsed\":%u,\"tree_hash_qual\":%u,"
            "\"xfer_crc\":%u,\"xfer_hex\":\"%s\",\"chunk_size\":%zu,\"reparsed_chunk_crc\":%u}%s\n",
            seed, num_lists, static_cast<int>(sweep_pt), param_ui_crc,
            tree_hash_orig, tree_hash_parsed, tree_hash_qual,
            xfer_crc, hex_encode(xfer_bytes).c_str(),
            chunk_reparsed.bytes.size(),
            [&]() {
                CRC c;
                c.computeCRC(chunk_reparsed.bytes.data(), static_cast<Int>(chunk_reparsed.bytes.size()));
                return c.get();
            }(),
            seed + 1 < 1000 ? "," : "");

        deleteInstance(qual_list);
        deleteInstance(dup_list);
        for (Int li = 0; li < num_lists; ++li) {
            deleteInstance(lists[li]);
            deleteInstance(read_lists[li]);
        }
    }

    std::fprintf(out, "  ]\n}\n");
    std::fclose(out);
    std::remove(tmp_save);
    return 0;
}
