#include "PreRTS.h"
#include "Common/crc.h"
#include "Common/INI.h"
#define DEFINE_DEATH_NAMES
#include "GameLogic/Damage.h"

#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/crc.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/RandomValue.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/System/Damage.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INI.cpp"

const AsciiString AsciiString::TheEmptyString;
const UnicodeString UnicodeString::TheEmptyString;

static FileSystem g_fs;
FileSystem* TheFileSystem = &g_fs;
DamageFXStore* TheDamageFXStore = nullptr;
AudioManager* TheAudio = nullptr;
ScienceStore* TheScienceStore = nullptr;
SpecialPowerStore* TheSpecialPowerStore = nullptr;
ThingFactory* TheThingFactory = nullptr;
UpgradeCenter* TheUpgradeCenter = nullptr;
Anim2DCollection* TheAnim2DCollection = nullptr;
GameTextInterface* TheGameText = nullptr;
ImageCollection* TheMappedImageCollection = nullptr;
ParticleSystemManager* TheParticleSystemManager = nullptr;
ArmorStore* TheArmorStore = nullptr;
ObjectCreationListStore* TheObjectCreationListStore = nullptr;
WeaponStore* TheWeaponStore = nullptr;

const char* const TheVeterancyNames[] = {
    "REGULAR", "VETERAN", "ELITE", "HEROIC", nullptr
};
static_assert(ARRAY_SIZE(TheVeterancyNames) == LEVEL_COUNT + 1, "Incorrect array size");

struct IniBlockSample {
    UnsignedByte u8_val;
    Short i16_val;
    UnsignedShort u16_val;
    Int i32_val;
    UnsignedInt u32_val;
    Real f32_val;
    Real pos_f32;
    Real pct_val;
    Real angle_val;
    Real ang_vel;
    Real vel_val;
    Real accel_val;
    Real dur_real;
    UnsignedInt dur_uint;
    UnsignedShort dur_ushort;
    Bool bool_val;
    UnsignedInt bit32_mask;
    UnsignedByte bit8_mask;
    UnsignedInt bit32_flags;
    Int idx_val;
    UnsignedByte idx_u8;
    Int lookup_val;
    AsciiString ascii_val;
    AsciiString quoted_val;
    std::vector<AsciiString> vec_val;
    std::vector<AsciiString> sounds_val;
    RGBColor rgb;
    RGBAColorInt rgba_int;
    Color color_int;
    Coord2D c2;
    Coord3D c3;
    ICoord2D ic2;
    GameClientRandomVariable rand_var;
    UnsignedInt vet_flags;
    DamageTypeFlags dmg_flags;
    DeathTypeFlags death_flags;
};

static IniBlockSample g_loaded_sample{};

static const LookupListRec s_testLookup[] = {
    { "ALPHA", 10 },
    { "BETA", -5 },
    { "GAMMA", 42 },
    { "DELTA", 1000 },
    { nullptr, 0 }
};

static const char* const s_testFlags[] = {
    "FLAG_A", "FLAG_B", "FLAG_C", "FLAG_D", "FLAG_E", "FLAG_F", "FLAG_G", "FLAG_H",
    "FLAG_I", "FLAG_J", nullptr
};

static const FieldParse s_samplePart1[] = {
    { "U8Val", INI::parseUnsignedByte, nullptr, offsetof(IniBlockSample, u8_val) },
    { "I16Val", INI::parseShort, nullptr, offsetof(IniBlockSample, i16_val) },
    { "U16Val", INI::parseUnsignedShort, nullptr, offsetof(IniBlockSample, u16_val) },
    { "I32Val", INI::parseInt, nullptr, offsetof(IniBlockSample, i32_val) },
    { "U32Val", INI::parseUnsignedInt, nullptr, offsetof(IniBlockSample, u32_val) },
    { "F32Val", INI::parseReal, nullptr, offsetof(IniBlockSample, f32_val) },
    { "PosF32", INI::parsePositiveNonZeroReal, nullptr, offsetof(IniBlockSample, pos_f32) },
    { "PctVal", INI::parsePercentToReal, nullptr, offsetof(IniBlockSample, pct_val) },
    { "AngleVal", INI::parseAngleReal, nullptr, offsetof(IniBlockSample, angle_val) },
    { "AngVel", INI::parseAngularVelocityReal, nullptr, offsetof(IniBlockSample, ang_vel) },
    { "VelVal", INI::parseVelocityReal, nullptr, offsetof(IniBlockSample, vel_val) },
    { "AccelVal", INI::parseAccelerationReal, nullptr, offsetof(IniBlockSample, accel_val) },
    { "DurReal", INI::parseDurationReal, nullptr, offsetof(IniBlockSample, dur_real) },
    { "DurUInt", INI::parseDurationUnsignedInt, nullptr, offsetof(IniBlockSample, dur_uint) },
    { "DurUShort", INI::parseDurationUnsignedShort, nullptr, offsetof(IniBlockSample, dur_ushort) },
    { "BoolVal", INI::parseBool, nullptr, offsetof(IniBlockSample, bool_val) },
    { "BitIn32", INI::parseBitInInt32, reinterpret_cast<const void*>(uintptr_t(1u << 7)), offsetof(IniBlockSample, bit32_mask) },
    { nullptr, nullptr, nullptr, 0 }
};

static const FieldParse s_samplePart2[] = {
    { "BitStr8", INI::parseBitString8, s_testFlags, offsetof(IniBlockSample, bit8_mask) },
    { "BitStr32", INI::parseBitString32, s_testFlags, offsetof(IniBlockSample, bit32_flags) },
    { "IdxList", INI::parseIndexList, s_testFlags, offsetof(IniBlockSample, idx_val) },
    { "IdxU8", INI::parseByteSizedIndexList, s_testFlags, offsetof(IniBlockSample, idx_u8) },
    { "Lookup", INI::parseLookupList, s_testLookup, offsetof(IniBlockSample, lookup_val) },
    { "AsciiVal", INI::parseAsciiString, nullptr, offsetof(IniBlockSample, ascii_val) },
    { "QuotedVal", INI::parseQuotedAsciiString, nullptr, offsetof(IniBlockSample, quoted_val) },
    { "VecVal", INI::parseAsciiStringVector, nullptr, offsetof(IniBlockSample, vec_val) },
    { "VecAppend", INI::parseAsciiStringVectorAppend, nullptr, offsetof(IniBlockSample, vec_val) },
    { "SoundsVal", INI::parseSoundsList, nullptr, offsetof(IniBlockSample, sounds_val) },
    { "Rgb", INI::parseRGBColor, nullptr, offsetof(IniBlockSample, rgb) },
    { "RgbaInt", INI::parseRGBAColorInt, nullptr, offsetof(IniBlockSample, rgba_int) },
    { "ColorInt", INI::parseColorInt, nullptr, offsetof(IniBlockSample, color_int) },
    { "Coord2", INI::parseCoord2D, nullptr, offsetof(IniBlockSample, c2) },
    { "Coord3", INI::parseCoord3D, nullptr, offsetof(IniBlockSample, c3) },
    { "ICoord2", INI::parseICoord2D, nullptr, offsetof(IniBlockSample, ic2) },
    { "RandVar", INI::parseGameClientRandomVariable, nullptr, offsetof(IniBlockSample, rand_var) },
    { "VetFlags", INI::parseVeterancyLevelFlags, nullptr, offsetof(IniBlockSample, vet_flags) },
    { "DmgFlags", INI::parseDamageTypeFlags, nullptr, offsetof(IniBlockSample, dmg_flags) },
    { "DeathFlags", INI::parseDeathTypeFlags, nullptr, offsetof(IniBlockSample, death_flags) },
    { nullptr, nullptr, nullptr, 0 }
};

void INI::parseGameDataDefinition(INI* ini) {
    MultiIniFieldParse p;
    p.add(s_samplePart1);
    p.add(s_samplePart2);
    ini->initFromINIMulti(&g_loaded_sample, p);
}

#define STUB_PARSE(fn) void INI::fn(INI*) {}
STUB_PARSE(parseAIDataDefinition)
STUB_PARSE(parseAnim2DDefinition)
STUB_PARSE(parseArmorDefinition)
STUB_PARSE(parseAudioEventDefinition)
STUB_PARSE(parseAudioSettingsDefinition)
STUB_PARSE(parseBenchProfile)
STUB_PARSE(parseTerrainBridgeDefinition)
STUB_PARSE(parseCampaignDefinition)
STUB_PARSE(parseChallengeModeDefinition)
STUB_PARSE(parseCommandButtonDefinition)
STUB_PARSE(parseMetaMapDefinition)
STUB_PARSE(parseCommandSetDefinition)
STUB_PARSE(parseControlBarResizerDefinition)
STUB_PARSE(parseControlBarSchemeDefinition)
STUB_PARSE(parseCrateTemplateDefinition)
STUB_PARSE(parseCredits)
STUB_PARSE(parseDamageFXDefinition)
STUB_PARSE(parseDialogDefinition)
STUB_PARSE(parseDrawGroupNumberDefinition)
STUB_PARSE(parseDynamicGameLODDefinition)
STUB_PARSE(parseEvaEvent)
STUB_PARSE(parseFXListDefinition)
STUB_PARSE(parseHeaderTemplateDefinition)
STUB_PARSE(parseInGameUIDefinition)
STUB_PARSE(parseLODPreset)
STUB_PARSE(parseLanguageDefinition)
STUB_PARSE(parseLocomotorTemplateDefinition)
STUB_PARSE(parseMapCacheDefinition)
STUB_PARSE(parseMapDataDefinition)
STUB_PARSE(parseMappedImageDefinition)
STUB_PARSE(parseMiscAudio)
STUB_PARSE(parseMouseDefinition)
STUB_PARSE(parseMouseCursorDefinition)
STUB_PARSE(parseMultiplayerColorDefinition)
STUB_PARSE(parseMultiplayerSettingsDefinition)
STUB_PARSE(parseMultiplayerStartingMoneyChoiceDefinition)
STUB_PARSE(parseMusicTrackDefinition)
STUB_PARSE(parseObjectDefinition)
STUB_PARSE(parseObjectCreationListDefinition)
STUB_PARSE(parseObjectReskinDefinition)
STUB_PARSE(parseOnlineChatColorDefinition)
STUB_PARSE(parseParticleSystemDefinition)
STUB_PARSE(parsePlayerTemplateDefinition)
STUB_PARSE(parseRankDefinition)
STUB_PARSE(parseTerrainRoadDefinition)
STUB_PARSE(parseScienceDefinition)
STUB_PARSE(parseShellMenuSchemeDefinition)
STUB_PARSE(parseSpecialPowerDefinition)
STUB_PARSE(parseStaticGameLODDefinition)
STUB_PARSE(parseTerrainDefinition)
STUB_PARSE(parseUpgradeDefinition)
STUB_PARSE(parseVideoDefinition)
STUB_PARSE(parseWaterSettingDefinition)
STUB_PARSE(parseWaterTransparencyDefinition)
STUB_PARSE(parseWeaponTemplateDefinition)
STUB_PARSE(parseWeatherDefinition)
STUB_PARSE(parseWebpageURLDefinition)
STUB_PARSE(parseWindowTransitions)
void parseReallyLowMHz(INI*) {}

static uint32_t splitmix32(uint32_t& s) {
    uint32_t z = (s += 0x9e3779b9u);
    z = (z ^ (z >> 16)) * 0x85ebca6bu;
    z = (z ^ (z >> 13)) * 0xc2b2ae35u;
    return z ^ (z >> 16);
}

static uint32_t fbits(Real v) {
    uint32_t b = 0;
    std::memcpy(&b, &v, 4);
    return b;
}

static uint32_t rot_crc(uint32_t crc, uint32_t w) {
    return ((crc << 1) | (crc >> 31)) + w;
}

static uint32_t str_crc(uint32_t crc, const char* s) {
    UnsignedInt n = s ? static_cast<UnsignedInt>(std::strlen(s)) : 0u;
    crc = rot_crc(crc, n);
    for (UnsignedInt i = 0; i < n; ++i) {
        crc = rot_crc(crc, static_cast<UnsignedByte>(s[i]));
    }
    return crc;
}

static void json_hex(FILE* fp, const char* s) {
    std::fputc('"', fp);
    if (s) {
        for (const unsigned char* p = reinterpret_cast<const unsigned char*>(s); *p; ++p) {
            std::fprintf(fp, "%02x", *p);
        }
    }
    std::fputc('"', fp);
}

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "Usage: gen_wave3_ini <out.json>\n");
        return 1;
    }
    FILE* fp = std::fopen(argv[1], "wb");
    if (!fp) { std::perror(argv[1]); return 1; }

    std::fprintf(fp, "{\n  \"packet_id\": \"PKT-087\",\n  \"module\": \"INI\",\n"
                     "  \"oracle\": \"vendor/GeneralsX unmodified INI.cpp + Damage.cpp + crc.cpp + libgm.a\",\n"
                     "  \"vector_count\": 1000,\n  \"vectors\": [\n");

    const char* int_tokens[] = {
        "0", "42", "-17", "+99", "255", "-32768", "32767", "65535",
        "123abc", "+45px", "99999999999", "-99999999999", "+-5", "abc"
    };
    const char* uint_tokens[] = {
        "0", "123", "+77", "-1", "4294967295", "99999999999", "88xyz", "nope"
    };
    const char* real_tokens[] = {
        "0.0", "1.5", "-2.25", "+3.75", "100.125", "-0.5", "12.5%", "+25px", "NaN", "inf", "bad"
    };
    const char* bool_tokens[] = { "Yes", "yes", "No", "NO", "true", "1" };
    const char* decl_lines[] = {
        "  Object   AmericaTankCrusader   ",
        "OBJECT AmericaTankCrusader",
        "Object AmericaTank",
        "Object AmericaTankCrusaderExtra",
        "  End  ",
        "End",
        "End\r",
        "  END\t"
    };

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x9e3779b1u + 0x243f6a88u;

        // 1. Scan primitives on deterministic tokens picked from rng.
        const char* itok = int_tokens[splitmix32(rng) % ARRAY_SIZE(int_tokens)];
        Bool i_ok = TRUE;
        Int i_val = 0;
        try { i_val = INI::scanInt(itok); } catch (...) { i_ok = FALSE; }

        const char* utok = uint_tokens[splitmix32(rng) % ARRAY_SIZE(uint_tokens)];
        Bool u_ok = TRUE;
        UnsignedInt u_val = 0;
        try { u_val = INI::scanUnsignedInt(utok); } catch (...) { u_ok = FALSE; }

        const char* rtok = real_tokens[splitmix32(rng) % ARRAY_SIZE(real_tokens)];
        Bool r_ok = TRUE;
        Real r_val = 0.0f;
        try { r_val = INI::scanReal(rtok); } catch (...) { r_ok = FALSE; }

        const char* btok = bool_tokens[splitmix32(rng) % ARRAY_SIZE(bool_tokens)];
        Bool b_ok = TRUE;
        Bool b_val = FALSE;
        try { b_val = INI::scanBool(btok); } catch (...) { b_ok = FALSE; }

        const char* dline = decl_lines[splitmix32(rng) % ARRAY_SIZE(decl_lines)];
        char dbuf1[128], dbuf2[128];
        std::snprintf(dbuf1, sizeof(dbuf1), "%s", dline);
        std::snprintf(dbuf2, sizeof(dbuf2), "%s", dline);
        Bool is_decl = INI::isDeclarationOfType(AsciiString("Object"), AsciiString("AmericaTankCrusader"), dbuf1);
        Bool is_end = INI::isEndOfBlock(dbuf2);

        // 2. Construct rich INI file exercising all scanners, comments, quotes, flags, block parse.
        UnsignedInt u8 = splitmix32(rng) % 256u;
        Int i16 = static_cast<Int>(splitmix32(rng) % 65536u) - 32768;
        UnsignedInt u16 = splitmix32(rng) % 65536u;
        Int i32 = static_cast<Int>(splitmix32(rng) % 200000u) - 100000;
        UnsignedInt u32 = splitmix32(rng);
        Real f32_v = static_cast<Real>(static_cast<Int>(splitmix32(rng) % 4000u) - 2000) * 0.25f;
        Real pos_v = static_cast<Real>((splitmix32(rng) % 2000u) + 1u) * 0.5f;
        Real pct_v = static_cast<Real>((splitmix32(rng) % 400u)) * 0.5f;
        Real deg_v = static_cast<Real>(splitmix32(rng) % 3600u) * 0.1f;
        UnsignedInt dur_ms = (splitmix32(rng) % 10000u);
        Bool bflag = (splitmix32(rng) & 1u) ? TRUE : FALSE;
        UnsignedInt cr = splitmix32(rng) % 256u, cg = splitmix32(rng) % 256u, cb = splitmix32(rng) % 256u, ca = splitmix32(rng) % 256u;
        Bool omit_a = (splitmix32(rng) & 1u) ? TRUE : FALSE;
        const char* flag_choice = s_testFlags[splitmix32(rng) % 8];
        const char* lk_choice = s_testLookup[splitmix32(rng) % 4].name;
        const char* dist_name = GameClientRandomVariable::DistributionTypeNames[splitmix32(rng) % 6];
        const char* dmg_op = (seed % 3 == 0) ? "ALL -EXPLOSION -POISON" :
                             (seed % 3 == 1) ? "NONE +SMALL_ARMS +LASER +MICROWAVE" :
                                               "NONE +SUBDUAL_MISSILE +STATUS";
        const char* death_op = (seed & 1) ? "NONE +NORMAL +BURNED +POISONED_GAMMA" : "ALL -SUICIDED";
        const char* vet_op = (seed % 2 == 0) ? "NONE +REGULAR +HEROIC" : "ALL -VETERAN";

        const char* bit32_prefix = (seed & 1) ? "+" : "";
        char bit32_line[64];
        std::snprintf(bit32_line, sizeof(bit32_line), "%s%s %sFLAG_I", bit32_prefix, flag_choice, bit32_prefix);

        char ini_text[4096];
        int len = std::snprintf(
            ini_text, sizeof(ini_text),
            "; Leading comment ignored\r\n"
            "GameData\x0b\n"
            "  U8Val = %u ; trailing comment\n"
            "  I16Val = %d\n"
            "  U16Val = %u\n"
            "  I32Val = %d\n"
            "  U32Val = %u\n"
            "  F32Val = %.2f\n"
            "  PosF32 = %.1f\n"
            "  PctVal = %.1f%%\n"
            "  AngleVal = %.1f\n"
            "  AngVel = %.1f\n"
            "  VelVal = %.2f\n"
            "  AccelVal = %.2f\n"
            "  DurReal = %u\n"
            "  DurUInt = %u\n"
            "  DurUShort = %u\n"
            "  BoolVal = %s\n"
            "  BitIn32 = %s\n"
            "  BitStr8 = +FLAG_A +FLAG_D -FLAG_B\n"
            "  BitStr32 = %s\n"
            "  IdxList = %s\n"
            "  IdxU8 = %s\n"
            "  Lookup = %s\n"
            "  AsciiVal = \"SingleQuoted\"\n"
            "  QuotedVal = \"Two Words\"\n"
            "  VecVal = Alpha Beta Gamma\n"
            "  VecAppend = Delta\n"
            "  SoundsVal = SndA, SndB SndC\n"
            "  Rgb = R:%u G:%u B:%u\n"
            "  RgbaInt = R:%u G:%u B:%u%s\n"
            "  ColorInt = R:%u G:%u B:%u A:%u\n"
            "  Coord2 = X:%.2f Y:%.2f\n"
            "  Coord3 = X:%.2f Y:%.2f Z:%.2f\n"
            "  ICoord2 = X:%d Y:%d\n"
            "  RandVar = %.2f %.2f %s\n"
            "  VetFlags = %s\n"
            "  DmgFlags = %s\n"
            "  DeathFlags = %s\n"
            "End\n",
            u8, i16, u16, i32, u32,
            static_cast<double>(f32_v), static_cast<double>(pos_v), static_cast<double>(pct_v),
            static_cast<double>(deg_v), static_cast<double>(deg_v * 0.5f),
            static_cast<double>(pos_v), static_cast<double>(pos_v),
            dur_ms, dur_ms, dur_ms,
            bflag ? "Yes" : "No",
            bflag ? "Yes" : "No",
            bit32_line, flag_choice, flag_choice, lk_choice,
            cr, cg, cb,
            cr, cg, cb,
            omit_a ? "" : " A:128",
            cr, cg, cb, ca,
            static_cast<double>(f32_v), static_cast<double>(pos_v),
            static_cast<double>(f32_v), static_cast<double>(pos_v), static_cast<double>(pct_v),
            i16, static_cast<Int>(u8),
            static_cast<double>(f32_v), static_cast<double>(f32_v + 10.0f), dist_name,
            vet_op, dmg_op, death_op);
        if (len <= 0 || static_cast<size_t>(len) >= sizeof(ini_text)) return 2;

        g_loaded_sample = IniBlockSample{};
        g_loaded_sample.bit8_mask = static_cast<UnsignedByte>(seed & 0x55u);
        g_loaded_sample.bit32_mask = (seed & 1u) ? 0xffu : 0x00u;
        g_fs.nextFileContent.assign(ini_text, static_cast<size_t>(len));
        g_fs.nextFileExists = true;

        HarnessXferCRC xfer_crc;
        INI ini;
        Bool load_ok = TRUE;
        try {
            ini.load(AsciiString("Data\\INI\\Test.ini"), INI_LOAD_OVERWRITE, &xfer_crc);
        } catch (const INIException& ex) {
            if (seed == 0) std::fprintf(stderr, "SEED 0 INIException: %s\n", ex.mFailureMessage ? ex.mFailureMessage : "(null)");
            load_ok = FALSE;
        } catch (Int code) {
            if (seed == 0) std::fprintf(stderr, "SEED 0 error code: 0x%x\n", code);
            load_ok = FALSE;
        } catch (...) {
            if (seed == 0) std::fprintf(stderr, "SEED 0 unknown exception\n");
            load_ok = FALSE;
        }

        // Compute deterministic state CRC of parsed g_loaded_sample fields.
        uint32_t out_crc = 0;
        out_crc = rot_crc(out_crc, g_loaded_sample.u8_val);
        out_crc = rot_crc(out_crc, static_cast<uint32_t>(g_loaded_sample.i16_val));
        out_crc = rot_crc(out_crc, g_loaded_sample.u16_val);
        out_crc = rot_crc(out_crc, static_cast<uint32_t>(g_loaded_sample.i32_val));
        out_crc = rot_crc(out_crc, g_loaded_sample.u32_val);
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.f32_val));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.pos_f32));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.pct_val));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.angle_val));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.ang_vel));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.vel_val));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.accel_val));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.dur_real));
        out_crc = rot_crc(out_crc, g_loaded_sample.dur_uint);
        out_crc = rot_crc(out_crc, g_loaded_sample.dur_ushort);
        out_crc = rot_crc(out_crc, g_loaded_sample.bool_val);
        out_crc = rot_crc(out_crc, g_loaded_sample.bit32_mask);
        out_crc = rot_crc(out_crc, g_loaded_sample.bit8_mask);
        out_crc = rot_crc(out_crc, g_loaded_sample.bit32_flags);
        out_crc = rot_crc(out_crc, static_cast<uint32_t>(g_loaded_sample.idx_val));
        out_crc = rot_crc(out_crc, g_loaded_sample.idx_u8);
        out_crc = rot_crc(out_crc, static_cast<uint32_t>(g_loaded_sample.lookup_val));
        out_crc = str_crc(out_crc, g_loaded_sample.ascii_val.str());
        out_crc = str_crc(out_crc, g_loaded_sample.quoted_val.str());
        out_crc = rot_crc(out_crc, static_cast<uint32_t>(g_loaded_sample.vec_val.size()));
        for (const auto& s : g_loaded_sample.vec_val) out_crc = str_crc(out_crc, s.str());
        out_crc = rot_crc(out_crc, static_cast<uint32_t>(g_loaded_sample.sounds_val.size()));
        for (const auto& s : g_loaded_sample.sounds_val) out_crc = str_crc(out_crc, s.str());
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.rgb.red));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.rgb.green));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.rgb.blue));
        out_crc = rot_crc(out_crc, g_loaded_sample.rgba_int.red);
        out_crc = rot_crc(out_crc, g_loaded_sample.rgba_int.green);
        out_crc = rot_crc(out_crc, g_loaded_sample.rgba_int.blue);
        out_crc = rot_crc(out_crc, g_loaded_sample.rgba_int.alpha);
        out_crc = rot_crc(out_crc, g_loaded_sample.color_int);
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.c2.x));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.c2.y));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.c3.x));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.c3.y));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.c3.z));
        out_crc = rot_crc(out_crc, static_cast<uint32_t>(g_loaded_sample.ic2.x));
        out_crc = rot_crc(out_crc, static_cast<uint32_t>(g_loaded_sample.ic2.y));
        out_crc = rot_crc(out_crc, static_cast<uint32_t>(g_loaded_sample.rand_var.getDistributionType()));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.rand_var.getMinimumValue()));
        out_crc = rot_crc(out_crc, fbits(g_loaded_sample.rand_var.getMaximumValue()));
        out_crc = rot_crc(out_crc, g_loaded_sample.vet_flags);
        for (Int b = 0; b < DAMAGE_NUM_TYPES; ++b) {
            out_crc = rot_crc(out_crc, g_loaded_sample.dmg_flags.test(b) ? 1u : 0u);
        }
        out_crc = rot_crc(out_crc, g_loaded_sample.death_flags);

        std::fprintf(fp, "    {\"seed\":%u,\"itok\":\"%s\",\"i_ok\":%u,\"i_val\":%d,"
                           "\"utok\":\"%s\",\"u_ok\":%u,\"u_val\":%u,"
                           "\"rtok\":\"%s\",\"r_ok\":%u,\"r_bits\":%u,"
                           "\"btok\":\"%s\",\"b_ok\":%u,\"b_val\":%u,"
                           "\"dline_hex\":",
            seed, itok, i_ok, i_val,
            utok, u_ok, u_val,
            rtok, r_ok, fbits(r_val),
            btok, b_ok, b_val);
        json_hex(fp, dline);
        std::fprintf(fp, ",\"is_decl\":%u,\"is_end\":%u,\"bit8_init\":%u,\"bit32_init\":%u,"
                         "\"load_ok\":%u,\"line_crc\":%u,\"out_crc\":%u,\"ini_hex\":",
            is_decl, is_end, static_cast<UnsignedInt>(seed & 0x55u),
            (seed & 1u) ? 0xffu : 0x00u,
            load_ok, xfer_crc.getCRC(), out_crc);
        json_hex(fp, ini_text);
        std::fprintf(fp, "}%s\n", (seed + 1u < 1000u) ? "," : "");
    }

    std::fprintf(fp, "  ]\n}\n");
    std::fclose(fp);
    std::fprintf(stderr, "Generated 1000 PKT-087 vectors to %s\n", argv[1]);
    return 0;
}
