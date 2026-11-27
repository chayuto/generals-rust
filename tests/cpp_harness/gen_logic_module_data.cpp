// Local-only oracle for the `ModuleData` of the logic modules with no ported behaviour.
//   argv[1]: output JSON. ZH_DIR: game directory (default $HOME/Games/generalszh).
//   ORACLE_DUMP: optional file for the retail "module|template/tag|field=value" lines
//   (retail data: do not commit).
// Rows:
//   "syn"    a made-up module block built from the reference field table, and every member
//            after the reference parsed it;
//   "unknown" a block with a key the table lacks;
//   "retail" instance count and digest per (module, field) over the installed game's data.
// The field table of each class comes from its own `buildFieldParse`; each member is read
// at the table's offset according to the parse function, so a function this file does not
// know stops the run.
// Build: python3 scripts/build_oracle.py tests/cpp_harness/gen_logic_module_data.cpp -o <bin>
#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <list>
#include <map>
#include <set>
#include <string>
#include <vector>

#define private public
#define protected public
#include "oracle_wp51_world.h"
#include "Common/BIGFileSystem.h"
#include "Common/FileSystem.h"
#include "Common/SpecialPower.h"
#include "GameClient/FXList.h"
#include "GameClient/ParticleSys.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/AnimationSteeringUpdate.h"
#include "GameLogic/Module/AssaultTransportAIUpdate.h"
#include "GameLogic/Module/BoneFXUpdate.h"
#include "GameLogic/Module/BridgeBehavior.h"
#include "GameLogic/Module/CheckpointUpdate.h"
#include "GameLogic/Module/ChinookAIUpdate.h"
#include "GameLogic/Module/CleanupHazardUpdate.h"
#include "GameLogic/Module/CountermeasuresBehavior.h"
#include "GameLogic/Module/DeliverPayloadAIUpdate.h"
#include "GameLogic/Module/DemoTrapUpdate.h"
#include "GameLogic/Module/DeployStyleAIUpdate.h"
#include "GameLogic/Module/DozerAIUpdate.h"
#include "GameLogic/Module/EMPUpdate.h"
#include "GameLogic/Module/EnemyNearUpdate.h"
#include "GameLogic/Module/FireOCLAfterWeaponCooldownUpdate.h"
#include "GameLogic/Module/FirestormDynamicGeometryInfoUpdate.h"
#include "GameLogic/Module/FlightDeckBehavior.h"
#include "GameLogic/Module/HackInternetAIUpdate.h"
#include "GameLogic/Module/HijackerUpdate.h"
#include "GameLogic/Module/JetAIUpdate.h"
#include "GameLogic/Module/MissileAIUpdate.h"
#include "GameLogic/Module/MobMemberSlavedUpdate.h"
#include "GameLogic/Module/NeutronMissileSlowDeathUpdate.h"
#include "GameLogic/Module/NeutronMissileUpdate.h"
#include "GameLogic/Module/OverlordContain.h"
#include "GameLogic/Module/ParticleUplinkCannonUpdate.h"
#include "GameLogic/Module/PilotFindVehicleUpdate.h"
#include "GameLogic/Module/RailedTransportAIUpdate.h"
#include "GameLogic/Module/RailedTransportDockUpdate.h"
#include "GameLogic/Module/RailroadGuideAIUpdate.h"
#include "GameLogic/Module/RebuildHoleBehavior.h"
#include "GameLogic/Module/RiderChangeContain.h"
#include "GameLogic/Module/SpectreGunshipUpdate.h"
#include "GameLogic/Module/StickyBombUpdate.h"
#include "GameLogic/Module/StructureCollapseUpdate.h"
#include "GameLogic/Module/SupplyTruckAIUpdate.h"
#include "GameLogic/Module/WaveGuideUpdate.h"
#include "GameLogic/Module/WorkerAIUpdate.h"
// The device include directory is not on the engine library's include path.
#include "../../vendor/GeneralsX/Core/GameEngineDevice/Include/StdDevice/Common/StdLocalFileSystem.h"
#include "../../vendor/GeneralsX/Core/GameEngineDevice/Include/W3DDevice/Common/W3DModuleFactory.h"

class LabelText : public oracle_wp51::NullGameText {
public:
    UnicodeString fetch(const Char* label, Bool* exists = nullptr) override {
        if (exists) *exists = TRUE;
        UnicodeString u;
        u.translate(label);
        return u;
    }
    UnicodeString fetch(AsciiString label, Bool* exists = nullptr) override {
        return fetch(label.str(), exists);
    }
};

struct Spec {
    const char* module;
    void (*build)(MultiIniFieldParse&);
    const char* (*base)(const ModuleData*);
};
template <class T> static Spec spec(const char* module) {
    return {module, [](MultiIniFieldParse& p) { T::buildFieldParse(p); },
        [](const ModuleData* d) { return reinterpret_cast<const char*>(static_cast<const T*>(d)); }};
}
#define S_(name) spec<name##ModuleData>(#name)

static std::vector<Spec> specs() {
    return {S_(AnimationSteeringUpdate), S_(AssaultTransportAIUpdate), S_(BoneFXUpdate),
        S_(BridgeBehavior), S_(CheckpointUpdate), S_(ChinookAIUpdate), S_(CleanupHazardUpdate),
        S_(CountermeasuresBehavior), S_(DeliverPayloadAIUpdate), S_(DemoTrapUpdate),
        S_(DeployStyleAIUpdate), S_(DozerAIUpdate), S_(EMPUpdate), S_(EnemyNearUpdate),
        S_(FireOCLAfterWeaponCooldownUpdate), S_(FirestormDynamicGeometryInfoUpdate),
        S_(FlightDeckBehavior), S_(HackInternetAIUpdate), S_(HijackerUpdate), S_(JetAIUpdate),
        S_(LeafletDropBehavior), S_(MissileAIUpdate), S_(MobMemberSlavedUpdate),
        S_(NeutronMissileSlowDeathBehavior), S_(NeutronMissileUpdate), S_(OverlordContain),
        S_(ParticleUplinkCannonUpdate), S_(PilotFindVehicleUpdate), S_(RailedTransportAIUpdate),
        S_(RailedTransportDockUpdate), S_(RailroadBehavior), S_(RebuildHoleBehavior),
        S_(RiderChangeContain), S_(SpectreGunshipUpdate), S_(StickyBombUpdate),
        S_(StructureCollapseUpdate), S_(SupplyTruckAIUpdate),
        spec<AIUpdateModuleData>("TransportAIUpdate"), spec<AIUpdateModuleData>("WanderAIUpdate"),
        S_(WaveGuideUpdate), S_(WorkerAIUpdate)};
}

struct Field {
    std::string name;
    INIFieldParseProc parse;
    const void* user;
    int offset;
};

static std::vector<Field> fieldsOf(const Spec& s) {
    MultiIniFieldParse p;
    s.build(p);
    std::vector<Field> out;
    std::set<std::string> seen;
    for (int i = 0; i < p.getCount(); ++i) {
        const int extra = p.getNthExtraOffset(i);
        for (const FieldParse* f = p.getNthFieldParse(i); f->token; ++f)
            if (seen.insert(f->token).second)
                out.push_back({f->token, f->parse, f->userData, f->offset + extra});
    }
    return out;
}

static std::string R(Real v) {
    UnsignedInt u;
    memcpy(&u, &v, 4);
    char b[16];
    snprintf(b, sizeof(b), "%08x", u);
    return b;
}
static std::string I(long long v) { return std::to_string(v); }
static std::string S(const AsciiString& s) { return s.str(); }
template <class T> static const T& at(const char* p) { return *reinterpret_cast<const T*>(p); }
template <class V> static std::string join(const V& v, std::string (*f)(const typename V::value_type&), const char* sep = ",") {
    std::string out;
    bool first = true;
    for (const auto& x : v) {
        if (!first) out += sep;
        first = false;
        out += f(x);
    }
    return out;
}
static std::string str(const AsciiString& s) { return s.str(); }
template <class F> static std::string bits(const F& flags, int n) {
    std::string out;
    for (int i = 0; i < n; ++i) out += flags.test(i) ? '1' : '0';
    return out;
}

static std::string fxName(const FXList* p) {
    if (!p) return "";
    for (auto& kv : TheFXListStore->m_fxmap)
        if (&kv.second == p) return S(TheNameKeyGenerator->keyToName(kv.first));
    return "?";
}
static std::string fxItem(const FXList* const& p) { return fxName(p); }
static std::string oclName(const ObjectCreationList* p) {
    if (!p) return "";
    for (auto& kv : TheObjectCreationListStore->m_ocls)
        if (&kv.second == p) return S(TheNameKeyGenerator->keyToName(kv.first));
    return "?";
}
static std::string oclItem(const ObjectCreationList* const& p) { return oclName(p); }
static std::string psName(const ParticleSystemTemplate* p) { return p ? S(p->getName()) : ""; }
static std::string weaponName(const WeaponTemplate* p) { return p ? S(p->getName()) : ""; }
static std::string weaponItem(const WeaponTemplate* const& p) { return weaponName(p); }
static std::string powerName(const SpecialPowerTemplate* p) { return p ? S(p->getName()) : ""; }

static std::string decal(const RadiusDecalTemplate& d) {
    return S(d.m_name) + "|" + I((UnsignedInt)d.m_shadowType) + "|" + R(d.m_minOpacity) + "|" +
        R(d.m_maxOpacity) + "|" + I(d.m_opacityThrobTime) + "|" + I((UnsignedInt)d.m_color) + "|" +
        I(d.m_onlyVisibleToOwningPlayer ? 1 : 0);
}

template <class V> static std::string boneHead(const V& info, const GameClientRandomVariable* c,
    const GameLogicRandomVariable* l) {
    const Real low = c ? c->m_low : l->m_low;
    const Real high = c ? c->m_high : l->m_high;
    return S(info.locInfo.boneName) + "|" + I(info.onlyOnce ? 1 : 0) + "|" + R(low) + "|" + R(high) + "|";
}

static std::string bridgeFx(const BridgeFXInfo& i) {
    return fxName(i.fx) + "|" + I(i.timeAndLocationInfo.delay) + "|" + S(i.timeAndLocationInfo.boneName);
}
static std::string bridgeOcl(const BridgeOCLInfo& i) {
    return oclName(i.ocl) + "|" + I(i.timeAndLocationInfo.delay) + "|" + S(i.timeAndLocationInfo.boneName);
}

enum Kind {
    K_REAL, K_INT, K_UINT, K_BOOL, K_ASCII, K_STRINGS, K_APPEND, K_AUDIO, K_COORD, K_RGB, K_FX,
    K_OCL, K_PSYS, K_WEAPON, K_POWER, K_KINDOF, K_STATUS, K_DAMAGE_FLAGS, K_DEATH_FLAGS,
    K_VET_FLAGS, K_DAMAGE_BIT, K_BITS32, K_INDEX, K_LOOKUP, K_TURRET, K_DECAL, K_PAYLOAD,
    K_SD_FX, K_SD_OCL, K_SD_WEAPON, K_SC_OCL, K_SC_FX, K_BR_FX, K_BR_OCL, K_BONE_FX,
    K_BONE_OCL, K_BONE_PSYS, K_STRIP, K_RIDER,
    // Reals with their own parsers.
    K_PERCENT, K_ANGLE, K_ANGULAR, K_VELOCITY, K_DURATION_REAL, K_FRICTION, K_HEIGHT,
    K_POSITIVE, K_DURATION,
};

static bool starts(const std::string& s, const char* p) { return s.rfind(p, 0) == 0; }

static Kind kindOf(const Spec& s, const Field& f) {
    const std::string m = s.module, n = f.name;
    const INIFieldParseProc p = f.parse;
    // File-static parse functions, by module and field.
    if (m == "StructureCollapseUpdate" && n == "OCL") return K_SC_OCL;
    if (m == "StructureCollapseUpdate" && n == "FXList") return K_SC_FX;
    if (m == "RailroadBehavior" && (n == "ForwardFriction" || n == "LateralFriction" ||
            n == "ZFriction" || n == "AerodynamicFriction"))
        return K_FRICTION;
    if (m == "RailroadBehavior" && n == "MinFallHeightForDamage") return K_HEIGHT;
    if (m == "NeutronMissileSlowDeathBehavior" && n == "FX") return K_SD_FX;
    if (m == "NeutronMissileSlowDeathBehavior" && n == "OCL") return K_SD_OCL;
    if (m == "NeutronMissileSlowDeathBehavior" && n == "Weapon") return K_SD_WEAPON;
    if (p == BridgeBehaviorModuleData::parseFX) return K_BR_FX;
    if (p == BridgeBehaviorModuleData::parseOCL) return K_BR_OCL;
    if (p == BoneFXUpdateModuleData::parseFXList) return K_BONE_FX;
    if (p == BoneFXUpdateModuleData::parseObjectCreationList) return K_BONE_OCL;
    if (p == BoneFXUpdateModuleData::parseParticleSystem) return K_BONE_PSYS;
    if (p == FlightDeckBehaviorModuleData::parseRunwayStrip) return K_STRIP;
    if (p == RiderChangeContainModuleData::parseRiderInfo) return K_RIDER;
    if (p == AIUpdateModuleData::parseTurret) return K_TURRET;
    if (p == TransportContainModuleData::parseInitialPayload) return K_PAYLOAD;
    if (p == RadiusDecalTemplate::parseRadiusDecalTemplate) return K_DECAL;
    if (p == INI::parseReal) return K_REAL;
    if (p == INI::parsePositiveNonZeroReal) return K_POSITIVE;
    if (p == INI::parsePercentToReal) return K_PERCENT;
    if (p == INI::parseAngleReal) return K_ANGLE;
    if (p == INI::parseAngularVelocityReal) return K_ANGULAR;
    if (p == INI::parseVelocityReal) return K_VELOCITY;
    if (p == INI::parseDurationReal) return K_DURATION_REAL;
    if (p == INI::parseInt) return K_INT;
    if (p == INI::parseUnsignedInt) return K_UINT;
    if (p == INI::parseDurationUnsignedInt) return K_DURATION;
    if (p == INI::parseBool) return K_BOOL;
    if (p == INI::parseAsciiString) return K_ASCII;
    if (p == INI::parseAsciiStringVector) return K_STRINGS;
    if (p == INI::parseAsciiStringVectorAppend) return K_APPEND;
    if (p == INI::parseAudioEventRTS) return K_AUDIO;
    if (p == INI::parseCoord3D) return K_COORD;
    if (p == INI::parseRGBColor) return K_RGB;
    if (p == INI::parseFXList) return K_FX;
    if (p == INI::parseObjectCreationList) return K_OCL;
    if (p == INI::parseParticleSystemTemplate) return K_PSYS;
    if (p == INI::parseWeaponTemplate) return K_WEAPON;
    if (p == INI::parseSpecialPowerTemplate) return K_POWER;
    if (p == KindOfMaskType::parseFromINI) return K_KINDOF;
    if (p == ObjectStatusMaskType::parseFromINI) return K_STATUS;
    if (p == INI::parseDamageTypeFlags) return K_DAMAGE_FLAGS;
    if (p == INI::parseDeathTypeFlags) return K_DEATH_FLAGS;
    if (p == INI::parseVeterancyLevelFlags) return K_VET_FLAGS;
    if (p == DamageTypeFlags::parseSingleBitFromINI) return K_DAMAGE_BIT;
    if (p == INI::parseBitString32) return K_BITS32;
    if (p == INI::parseIndexList) return K_INDEX;
    if (p == INI::parseLookupList) return K_LOOKUP;
    fprintf(stderr, "unknown parser: %s.%s\n", s.module, f.name.c_str());
    exit(1);
}

static std::string value(const Spec& s, const Field& f, const ModuleData* data) {
    const char* base = s.base(data);
    const char* p = base + f.offset;
    const Kind k = kindOf(s, f);
    switch (k) {
    case K_REAL: case K_PERCENT: case K_ANGLE: case K_ANGULAR: case K_VELOCITY:
    case K_DURATION_REAL: case K_FRICTION: case K_HEIGHT: case K_POSITIVE:
        return R(at<Real>(p));
    case K_INT: case K_INDEX: case K_LOOKUP: case K_DAMAGE_BIT: return I(at<Int>(p));
    case K_UINT: case K_DURATION:
        // The members declared `Int` hold small positive values here.
        return I(at<UnsignedInt>(p));
    case K_DEATH_FLAGS: case K_BITS32: return I(at<UnsignedInt>(p));
    case K_VET_FLAGS: {
        // As the levels held: the member keeps level n in bit n - 1. For REGULAR the
        // reference shifts by -1, which this CPU takes modulo 32; spelled out here
        // because `getVeterancyLevelFlag` is undefined behaviour for it.
        std::string out;
        for (int l = LEVEL_FIRST; l <= LEVEL_LAST; ++l)
            out += (at<VeterancyLevelFlags>(p) >> ((unsigned)(l - 1) & 31u)) & 1u ? '1' : '0';
        return out;
    }
    case K_BOOL: return I(at<Bool>(p) ? 1 : 0);
    case K_ASCII: return S(at<AsciiString>(p));
    case K_STRINGS: case K_APPEND: return join(at<std::vector<AsciiString>>(p), str);
    case K_AUDIO: return S(at<AudioEventRTS>(p).getEventName());
    case K_COORD: { const Coord3D& c = at<Coord3D>(p); return R(c.x) + "," + R(c.y) + "," + R(c.z); }
    case K_RGB: { const RGBColor& c = at<RGBColor>(p); return R(c.red) + "," + R(c.green) + "," + R(c.blue); }
    case K_FX: return fxName(at<const FXList*>(p));
    case K_OCL: return oclName(at<const ObjectCreationList*>(p));
    case K_PSYS: return psName(at<const ParticleSystemTemplate*>(p));
    case K_WEAPON: return weaponName(at<const WeaponTemplate*>(p));
    case K_POWER: return powerName(at<const SpecialPowerTemplate*>(p));
    case K_KINDOF: return bits(at<KindOfMaskType>(p), KINDOF_COUNT);
    case K_STATUS: return bits(at<ObjectStatusMaskType>(p), OBJECT_STATUS_COUNT);
    case K_DAMAGE_FLAGS: return bits(at<DamageTypeFlags>(p), DAMAGE_NUM_TYPES);
    case K_TURRET: return I(at<const void*>(p) ? 1 : 0);
    case K_DECAL: return decal(at<RadiusDecalTemplate>(p));
    case K_PAYLOAD: {
        const auto& payload = reinterpret_cast<const TransportContainModuleData*>(base)->m_initialPayload;
        return S(payload.name) + "|" + I(payload.count);
    }
    case K_SD_FX: case K_SD_OCL: case K_SD_WEAPON: {
        const auto* d = reinterpret_cast<const SlowDeathBehaviorModuleData*>(base);
        std::string out;
        for (int i = 0; i < SD_PHASE_COUNT; ++i) {
            if (i) out += ";";
            out += k == K_SD_FX ? join(d->m_fx[i], fxItem)
                : k == K_SD_OCL ? join(d->m_ocls[i], oclItem) : join(d->m_weapons[i], weaponItem);
        }
        return out;
    }
    case K_SC_OCL: case K_SC_FX: {
        const auto* d = reinterpret_cast<const StructureCollapseUpdateModuleData*>(base);
        std::string out;
        for (int i = 0; i < SC_PHASE_COUNT; ++i) {
            if (i) out += ";";
            out += k == K_SC_OCL ? join(d->m_ocls[i], oclItem) : join(d->m_fxs[i], fxItem);
        }
        return out;
    }
    case K_BR_FX: return join(at<BridgeFXList>(p), bridgeFx);
    case K_BR_OCL: return join(at<BridgeOCLList>(p), bridgeOcl);
    case K_BONE_FX: { const auto& i = at<BoneFXListInfo>(p); return boneHead(i, nullptr, &i.gameLogicDelay) + fxName(i.fx); }
    case K_BONE_OCL: { const auto& i = at<BoneOCLInfo>(p); return boneHead(i, nullptr, &i.gameLogicDelay) + oclName(i.ocl); }
    case K_BONE_PSYS: {
        const auto& i = at<BoneParticleSystemInfo>(p);
        return boneHead(i, &i.gameClientDelay, nullptr) + psName(i.particleSysTemplate);
    }
    case K_STRIP: { const AsciiString* n = &at<AsciiString>(p); return S(n[0]) + "," + S(n[1]); }
    case K_RIDER: {
        const RiderInfo& r = at<RiderInfo>(p);
        return S(r.m_templateName) + "|" + I((int)r.m_modelConditionFlagType) + "|" +
            I((int)r.m_weaponSetFlag) + "|" + I((int)r.m_objectStatusType) + "|" +
            S(r.m_commandSet) + "|" + I((int)r.m_locomotorSetType);
    }
    }
    exit(1);
}

static std::map<std::string, std::string> members(const Spec& s, const ModuleData* data) {
    std::map<std::string, std::string> out;
    for (const Field& f : fieldsOf(s)) out[f.name] = value(s, f, data);
    return out;
}

// One made-up line (or block) for a field; `v` picks the second set of values.
static std::string sample(const Spec& s, const Field& f, int v) {
    const std::string n = f.name;
    auto line = [&](const std::string& text) { return "    " + n + " = " + text + "\n"; };
    auto pick = [&](const char* a, const char* b) { return std::string(v ? b : a); };
    const char* const* names = static_cast<const char* const*>(f.user);
    switch (kindOf(s, f)) {
    case K_REAL: return line(pick("3.25", "-7.5"));
    case K_POSITIVE: return line(pick("3.25", "0.125"));
    case K_PERCENT: return line(pick("37%", "250%"));
    case K_ANGLE: return line(pick("90", "-33.3"));
    case K_ANGULAR: return line(pick("45", "123.4"));
    case K_VELOCITY: return line(pick("60", "17.3"));
    case K_DURATION_REAL: return line(pick("1500", "10"));
    case K_FRICTION: return line(pick("6", "0.7"));
    case K_HEIGHT: return line(pick("55", "7.5"));
    case K_INT: return line(pick("7", "-12"));
    case K_UINT: return line(pick("9", "4000"));
    case K_DURATION: return line(pick("1500", "10"));
    case K_BOOL: return line(pick("Yes", "No"));
    case K_ASCII: return line(pick("OracleName", "None"));
    case K_STRINGS: return v ? line("A B") + line("C") : line("A B C");
    case K_APPEND: return v ? line("A B") + line("C") : line("A");
    case K_AUDIO: return v ? line("OracleSound") + line("NoSound") : line("None");
    case K_COORD: return line(pick("X:1.5 Y:-2 Z:3", "X:0 Y:100.25 Z:-0.5"));
    case K_RGB: return line(pick("R:10 G:20 B:30", "R:255 G:0 B:128"));
    case K_FX: return line(pick("OracleFX", "None"));
    case K_OCL: return line(pick("OracleOCL", "None"));
    case K_PSYS: return line(pick("OraclePSys", "None"));
    case K_WEAPON: return line(pick("OracleWeapon", "None"));
    case K_POWER: return line(pick("OraclePower", "None"));
    case K_KINDOF: return line(pick("INFANTRY VEHICLE", "+STRUCTURE -INFANTRY"));
    case K_STATUS: return line(pick("DESTROYED CAN_ATTACK", "+SOLD"));
    case K_DAMAGE_FLAGS: return line(pick("NONE +LASER +FLAME", "ALL -LASER"));
    case K_DEATH_FLAGS: return line(pick("NONE +LASERED", "ALL -LASERED -CRUSHED"));
    case K_VET_FLAGS: return line(pick("NONE +ELITE", "ALL -REGULAR"));
    case K_DAMAGE_BIT: return line(pick("FLAME", "CRUSH"));
    case K_BITS32: return line(std::string(names[1]) + (v ? "" : std::string(" ") + names[2]));
    case K_INDEX: return line(names[v ? 1 : 2]);
    case K_LOOKUP: return line(static_cast<const LookupListRec*>(f.user)[v ? 1 : 2].name);
    case K_TURRET: return "    " + n + "\n      TurretTurnRate = 60\n    End\n";
    case K_DECAL:
        return v ? "    " + n + "\n      Style = SHADOW_ADDITIVE_DECAL\n      Color = R:1 G:2 B:3\n    End\n"
                 : "    " + n + "\n      Texture = OracleDecal\n      Style = SHADOW_ALPHA_DECAL\n"
                   "      OpacityMin = 25%\n      OpacityMax = 50%\n      OpacityThrobTime = 500\n"
                   "      Color = R:10 G:20 B:30 A:40\n      OnlyVisibleToOwningPlayer = No\n    End\n";
    case K_PAYLOAD: return line(pick("OracleUnit 3", "OracleUnit"));
    case K_SD_FX:
        return line("INITIAL OracleFX None OracleFX") + (v ? line("FINAL None") + line("INITIAL OracleFX") : "");
    case K_SD_OCL: return line("MIDPOINT OracleOCL") + (v ? line("MIDPOINT None") : "");
    case K_SD_WEAPON: return line("FINAL OracleWeapon") + (v ? line("INITIAL None OracleWeapon") : "");
    case K_SC_OCL: return line("BURST OracleOCL None") + (v ? line("INITIAL OracleOCL") + line("BURST None") : "");
    case K_SC_FX: return line("DELAY OracleFX") + (v ? line("FINAL None OracleFX") : "");
    case K_BR_FX:
        return line("FX:OracleFX Delay:500 Bone:Bone01") + (v ? line("FX:None Delay:10") : "");
    case K_BR_OCL:
        return line("OCL:OracleOCL Delay:2000") + (v ? line("OCL:OracleOCL Delay:100 Bone:\"Two Words\"") : "");
    case K_BONE_FX: return line(pick("bone:Bone01 onlyonce:No 100 200 fxlist:OracleFX", "bone:B OnlyOnce:Yes 50 50 FXList:None"));
    case K_BONE_OCL: return line(pick("bone:Bone02 onlyonce:Yes 0 0 ocl:OracleOCL", "bone:B onlyonce:No 700 900 OCL:None"));
    case K_BONE_PSYS: return line(pick("bone:Bone03 onlyonce:No 33 66 psys:OraclePSys", "bone:B onlyonce:Yes 1 2 PSys:None"));
    case K_STRIP: return v ? line("Start End") + line("Again") : line("Start End");
    case K_RIDER:
        return line(pick("OracleUnit MOVING WEAPON_RIDER1 DESTROYED OracleSet SET_NORMAL_UPGRADED",
            "Other TOPPLED VETERAN NONE Set2 SET_SLUGGISH"));
    }
    exit(1);
}

static std::string esc(const std::string& s) {
    std::string out;
    for (char c : s) {
        if (c == '\n') out += "\\n";
        else if (c == '"' || c == '\\') { out += '\\'; out += c; }
        else out += c;
    }
    return out;
}

// Loads `text` through the reference loader; false when it throws.
static bool load(const std::string& path, const std::string& text) {
    FILE* f = fopen(path.c_str(), "wb");
    fwrite(text.data(), 1, text.size(), f);
    fclose(f);
    try {
        INI ini;
        ini.load(path.c_str(), INI_LOAD_OVERWRITE, nullptr);
    } catch (...) {
        return false;
    }
    return true;
}

static std::string objectText(const std::string& object, const char* module, const std::string& body) {
    return "Object " + object + "\n  Behavior = " + module + " ModuleTag_01\n" + body + "  End\nEnd\n";
}

static const ModuleData* firstData(const std::string& object, const char* module) {
    const ThingTemplate* t = TheThingFactory->findTemplate(object.c_str());
    if (!t) return nullptr;
    for (const auto& nugget : t->getBehaviorModuleInfo().m_info)
        if (nugget.first == module) return nugget.second;
    return nullptr;
}

static std::vector<std::string> g_rows;
static const char* PREAMBLE =
    "Weapon OracleWeapon\nEnd\n"
    // Two members are derived from gravity while parsing.
    "GameData\n  Gravity = -64.0\nEnd\n"
    "FXList OracleFX\nEnd\n"
    "ObjectCreationList OracleOCL\nEnd\n"
    "ParticleSystem OraclePSys\nEnd\n"
    "SpecialPower OraclePower\nEnd\n";

static void synthetic(const std::string& path) {
    if (!load(path, PREAMBLE)) {
        fprintf(stderr, "preamble failed\n");
        exit(1);
    }
    g_rows.push_back("{\"row\":\"preamble\",\"text\":\"" + esc(PREAMBLE) + "\"}");
    for (const Spec& s : specs()) {
        const std::vector<Field> fields = fieldsOf(s);
        // 0: an empty block (the defaults); 1 and 2: every field of the table.
        for (int v = 0; v < 3; ++v) {
            std::string body;
            if (v)
                for (const Field& f : fields) body += sample(s, f, v - 1);
            const std::string object = std::string("Oracle_") + s.module + "_" + I(v);
            const std::string text = objectText(object, s.module, body);
            if (!load(path, text)) {
                fprintf(stderr, "rejected %s %d:\n%s", s.module, v, body.c_str());
                exit(1);
            }
            const ModuleData* data = firstData(object, s.module);
            if (!data) {
                fprintf(stderr, "no data %s\n", s.module);
                exit(1);
            }
            std::string values;
            for (const auto& kv : members(s, data)) values += kv.first + "=" + kv.second + "\n";
            g_rows.push_back(std::string("{\"row\":\"syn\",\"module\":\"") + s.module +
                "\",\"text\":\"" + esc(text) + "\",\"values\":\"" + esc(values) + "\"}");
        }
        // An unknown key, and a known key in the wrong case (or, for a class without
        // fields, another module's). This build only asserts on an unknown field
        // (`DEBUG_CRASH`) and goes on; the row records that and that no member moved.
        std::string wrong = fields.empty() ? "ScanDelayTime" : fields[0].name;
        for (char& c : wrong) c = (char)tolower((unsigned char)c);
        const std::string bodies[] = {"    NotAField = 1\n", "    " + wrong + " = 1\n"};
        for (int i = 0; i < 2; ++i) {
            const std::string object = std::string("OracleBad_") + s.module + "_" + I(i);
            const std::string text = objectText(object, s.module, bodies[i]);
            const bool accepted = load(path, text);
            const ModuleData* data = accepted ? firstData(object, s.module) : nullptr;
            const bool untouched = data &&
                members(s, data) == members(s, firstData(std::string("Oracle_") + s.module + "_0", s.module));
            g_rows.push_back(std::string("{\"row\":\"unknown\",\"module\":\"") + s.module +
                "\",\"accepted\":" + I(accepted) + ",\"untouched\":" + I(untouched) +
                ",\"text\":\"" + esc(text) + "\"}");
        }
    }
}

static void retail() {
    std::map<std::string, Spec> byName;
    for (const Spec& s : specs()) byName.emplace(s.module, s);
    std::vector<std::string> lines;
    std::map<std::string, unsigned> instances;
    for (const ThingTemplate* t = TheThingFactory->firstTemplate(); t; t = t->friend_getNextTemplate()) {
        const std::string name = t->getName().str();
        for (const auto& nugget : t->getBehaviorModuleInfo().m_info) {
            auto it = byName.find(nugget.first.str());
            if (it == byName.end()) continue;
            ++instances[it->first];
            const std::string entry = it->first + "|" + name + "/" + S(nugget.m_moduleTag) + "|";
            for (const auto& kv : members(it->second, nugget.second))
                lines.push_back(entry + kv.first + "=" + kv.second);
        }
    }
    std::sort(lines.begin(), lines.end());
    if (const char* dump = getenv("ORACLE_DUMP")) {
        FILE* f = fopen(dump, "wb");
        for (const std::string& l : lines) fprintf(f, "%s\n", l.c_str());
        fclose(f);
    }
    // FNV-1a 64 over the sorted lines of each (module, field).
    std::map<std::pair<std::string, std::string>, std::pair<unsigned, unsigned long long>> sums;
    for (const std::string& l : lines) {
        const size_t a = l.find('|');
        const size_t b = l.find('|', a + 1);
        const size_t c = l.find('=', b + 1);
        auto& s = sums[{l.substr(0, a), l.substr(b + 1, c - b - 1)}];
        if (s.first == 0) s.second = 0xcbf29ce484222325ULL;
        ++s.first;
        for (char ch : l + "\n") s.second = (s.second ^ (unsigned char)ch) * 0x100000001b3ULL;
    }
    for (auto& kv : instances)
        g_rows.push_back("{\"row\":\"instances\",\"module\":\"" + kv.first + "\",\"count\":" + I(kv.second) + "}");
    for (auto& kv : sums) {
        char fnv[20];
        snprintf(fnv, sizeof(fnv), "%016llx", kv.second.second);
        g_rows.push_back("{\"row\":\"retail\",\"module\":\"" + kv.first.first + "\",\"field\":\"" +
            kv.first.second + "\",\"count\":" + I(kv.second.first) + ",\"fnv\":\"" + fnv + "\"}");
    }
}

int main(int argc, char** argv) {
    if (argc < 2) return 2;
    oracle_wp51::World world;
    TheGameText = new LabelText;

    std::string dir = getenv("ZH_DIR") ? getenv("ZH_DIR") : std::string(getenv("HOME")) + "/Games/generalszh";
    TheLocalFileSystem = new StdLocalFileSystem;
    BIGFileSystem* big = new BIGFileSystem;
    TheArchiveFileSystem = big;
    TheFileSystem = new FileSystem;
    big->loadArchiveFilesFromDirectory(dir.c_str(), "*.big", FALSE, TRUE);
    big->loadArchiveFilesFromDirectory((dir + "/ZH_Generals").c_str(), "*.big", FALSE, FALSE);

    // The fixture's module factory has no draw modules.
    TheModuleFactory = new W3DModuleFactory;
    TheModuleFactory->init();

    static const char* PATHS[] = {
        "Data\\INI\\Default\\GameData", "Data\\INI\\GameData",
        "Data\\INI\\Default\\Science", "Data\\INI\\Science",
        "Data\\INI\\ParticleSystem",
        "Data\\INI\\Default\\FXList", "Data\\INI\\FXList",
        "Data\\INI\\Weapon",
        "Data\\INI\\Default\\ObjectCreationList", "Data\\INI\\ObjectCreationList",
        "Data\\INI\\Locomotor",
        "Data\\INI\\Default\\SpecialPower", "Data\\INI\\SpecialPower",
        "Data\\INI\\DamageFX", "Data\\INI\\Armor",
        "Data\\INI\\Default\\Object", "Data\\INI\\Object",
        "Data\\INI\\Default\\Upgrade", "Data\\INI\\Upgrade",
        "Data\\INI\\Default\\CommandButton", "Data\\INI\\CommandButton", "Data\\INI\\CommandSet",
        // Holds `Object` blocks as well as `CrateData`.
        "Data\\INI\\Default\\Crate", "Data\\INI\\Crate",
    };
    for (const char* p : PATHS) {
        fprintf(stdout, "load %s\n", p);
        fflush(stdout);
        try {
            INI ini;
            ini.loadFileDirectory(p, INI_LOAD_OVERWRITE, nullptr, INI::LoadFlags_SearchSubDirs);
        } catch (...) {
            fprintf(stdout, "FAILED %s\n", p);
            return 1;
        }
    }
    retail();
    // After the game's data (the reference refuses to define a weapon twice).
    const std::string scratch = std::string(argv[1]) + ".tmp.ini";
    synthetic(scratch);
    remove(scratch.c_str());

    FILE* out = fopen(argv[1], "wb");
    fprintf(out, "[\n");
    for (size_t i = 0; i < g_rows.size(); ++i)
        fprintf(out, "%s%s\n", g_rows[i].c_str(), i + 1 == g_rows.size() ? "" : ",");
    fprintf(out, "]\n");
    fclose(out);
    _Exit(0);
}
