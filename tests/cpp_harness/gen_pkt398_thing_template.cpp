// PKT-398 (ThingTemplate) + PKT-397 (ThingFactory) + PKT-396 (Thing) +
// PKT-394 (Overridable) + PKT-395 (Override) + PKT-393 (ObjectTypes) +
// PKT-392 (ObjectScriptStatusBits) + PKT-078 (ThingSort) + PKT-139 (ArmorSet) +
// PKT-408 (SparseMatchFinder) C++ Golden Vector Generator (Zero-Stub Build).
// Compiles directly against vendor/GeneralsX/build/macos-vulkan via scripts/build_oracle.py.
//
// Canonical build & verify commands:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt398_thing_template.cpp -o /tmp/gen_pkt398_o0 --opt O0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt398_thing_template.cpp -o /tmp/gen_pkt398_o2 --opt O2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt398_thing_template.cpp -o /tmp/gen_pkt398_ubsan --opt O1 --sanitize
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_pkt398_o0 docs/golden_vectors/PKT-398.json

#include "oracle_common.h"

#include <sstream>
#include <vector>
#include <unistd.h>

#include "Common/DamageFX.h"
#include "Common/Energy.h"
#include "Common/GameAudio.h"
#include "Common/GlobalData.h"
#include "Common/Handicap.h"
#include "Common/Module.h"
#include "Common/ModuleFactory.h"
#include "Common/NameKeyGenerator.h"
#include "Common/Player.h"
#include "Common/ProductionPrerequisite.h"
#include "Common/Radar.h"
#include "Common/Science.h"
#include "Common/Thing.h"
#include "Common/ThingFactory.h"
#include "Common/ThingSort.h"
#include "Common/ThingTemplate.h"
#include "GameClient/FXList.h"
#include "GameClient/GameText.h"
#include "GameLogic/Armor.h"
#include "GameLogic/ArmorSet.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/ObjectScriptStatusBits.h"
#include "GameLogic/ObjectTypes.h"
#include "GameLogic/RankInfo.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/WeaponSet.h"

ORACLE_DEFINE_LINK_GLOBALS("gen_pkt398_thing_template")

// MemberAccess tags for private internals needed only to drive/inspect oracle vectors cleanly.
struct GLSuperweaponTag { using type = UnsignedShort GameLogic::*; };
template struct MemberAccess<GLSuperweaponTag, &GameLogic::m_superweaponRestriction>;
UnsignedShort GameLogic::* access_member(GLSuperweaponTag);

using GLBuildableMap = std::hash_map<AsciiString, BuildableStatus, rts::hash<AsciiString>, rts::equal_to<AsciiString>>;
struct GLBuildableMapTag { using type = GLBuildableMap GameLogic::*; };
template struct MemberAccess<GLBuildableMapTag, &GameLogic::m_thingTemplateBuildableOverrides>;
GLBuildableMap GameLogic::* access_member(GLBuildableMapTag);

struct TTBuildCostTag { using type = UnsignedShort ThingTemplate::*; };
template struct MemberAccess<TTBuildCostTag, &ThingTemplate::m_buildCost>;
UnsignedShort ThingTemplate::* access_member(TTBuildCostTag);

struct TTBuildTimeTag { using type = Real ThingTemplate::*; };
template struct MemberAccess<TTBuildTimeTag, &ThingTemplate::m_buildTime>;
Real ThingTemplate::* access_member(TTBuildTimeTag);

struct TFNextIdTag { using type = UnsignedShort ThingFactory::*; };
template struct MemberAccess<TFNextIdTag, &ThingFactory::m_nextTemplateID>;
UnsignedShort ThingFactory::* access_member(TFNextIdTag);

struct TFFirstTmplTag { using type = ThingTemplate* ThingFactory::*; };
template struct MemberAccess<TFFirstTmplTag, &ThingFactory::m_firstTemplate>;
ThingTemplate* ThingFactory::* access_member(TFFirstTmplTag);

struct PlayerCostMapTag { using type = ProductionChangeMap Player::*; };
template struct MemberAccess<PlayerCostMapTag, &Player::m_productionCostChanges>;
ProductionChangeMap Player::* access_member(PlayerCostMapTag);

struct PlayerTimeMapTag { using type = ProductionChangeMap Player::*; };
template struct MemberAccess<PlayerTimeMapTag, &Player::m_productionTimeChanges>;
ProductionChangeMap Player::* access_member(PlayerTimeMapTag);

struct PlayerKindOfProdTag { using type = std::list<KindOfPercentProductionChange*> Player::*; };
template struct MemberAccess<PlayerKindOfProdTag, &Player::m_kindOfPercentProductionChangeList>;
std::list<KindOfPercentProductionChange*> Player::* access_member(PlayerKindOfProdTag);

struct PlayerHandicapTag { using type = Handicap Player::*; };
template struct MemberAccess<PlayerHandicapTag, &Player::m_handicap>;
Handicap Player::* access_member(PlayerHandicapTag);

struct PlayerEnergyTag { using type = Energy Player::*; };
template struct MemberAccess<PlayerEnergyTag, &Player::m_energy>;
Energy Player::* access_member(PlayerEnergyTag);

struct HandicapArrayTag { using type = Real (Handicap::*)[Handicap::HANDICAP_TYPE_COUNT][2]; };
template struct MemberAccess<HandicapArrayTag, &Handicap::m_handicaps>;
Real (Handicap::*access_member(HandicapArrayTag))[Handicap::HANDICAP_TYPE_COUNT][2];

struct EnergyProdTag { using type = Int Energy::*; };
template struct MemberAccess<EnergyProdTag, &Energy::m_energyProduction>;
Int Energy::* access_member(EnergyProdTag);

struct EnergyConsTag { using type = Int Energy::*; };
template struct MemberAccess<EnergyConsTag, &Energy::m_energyConsumption>;
Int Energy::* access_member(EnergyConsTag);

struct EnergyOwnerTag { using type = Player* Energy::*; };
template struct MemberAccess<EnergyOwnerTag, &Energy::m_owner>;
Player* Energy::* access_member(EnergyOwnerTag);

namespace {

class OracleGameText final : public GameTextInterface {
public:
    void init() override {}
    void reset() override {}
    void update() override {}
    UnicodeString fetch(const Char* label, Bool* exists = nullptr) override {
        if (exists) *exists = (label != nullptr) ? TRUE : FALSE;
        if (!label) return UnicodeString::TheEmptyString;
        std::string s = std::string("TXT:") + label;
        std::vector<WideChar> w;
        w.reserve(s.size() + 1);
        for (unsigned char ch : s) {
            w.push_back(static_cast<WideChar>(ch));
        }
        w.push_back(0);
        UnicodeString out;
        out.set(w.data());
        return out;
    }
    UnicodeString fetch(AsciiString label, Bool* exists = nullptr) override {
        return fetch(label.str(), exists);
    }
    UnicodeString fetchFormat(const Char* label, ...) override { return fetch(label); }
    UnicodeString fetchOrSubstitute(const Char* label, const WideChar*) override { return fetch(label); }
    UnicodeString fetchOrSubstituteFormat(const Char* label, const WideChar*, ...) override { return fetch(label); }
    UnicodeString fetchOrSubstituteFormatVA(const Char* label, const WideChar*, va_list) override { return fetch(label); }
    AsciiStringVec& getStringsWithLabelPrefix(AsciiString) override {
        static AsciiStringVec empty;
        return empty;
    }
    void initMapStringFile(const AsciiString&) override {}
};

class OracleTerrainLogic final : public TerrainLogic {
public:
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        Real hz = 10.0f + 0.05f * x - 0.03f * y;
        if (normal) {
            normal->x = -0.05f;
            normal->y = 0.03f;
            normal->z = 1.0f;
            normal->normalize();
        }
        return hz;
    }
    Real getLayerHeight(Real x, Real y, PathfindLayerEnum layer, Coord3D* normal = nullptr, Bool clip = true) const override {
        (void)clip;
        if (layer == LAYER_GROUND) {
            return getGroundHeight(x, y, normal);
        }
        if (normal) {
            normal->x = 0.0f;
            normal->y = 0.0f;
            normal->z = 1.0f;
        }
        return getGroundHeight(x, y, nullptr) + 5.0f;
    }
    PathfindLayerEnum alignOnTerrain(Real angle, const Coord3D& pos, Bool stickToGround, Matrix3D& mtx) override {
        Coord3D terrainNormal;
        Real terrainAtPos = getGroundHeight(pos.x, pos.y, &terrainNormal);
        makeAlignToNormalMatrix(angle, pos, terrainNormal, mtx);
        if (stickToGround) {
            mtx.Set_Z_Translation(terrainAtPos);
        }
        return LAYER_GROUND;
    }
    Bool isUnderwater(Real x, Real y, Real* waterZ = nullptr, Real* terrainZ = nullptr) override {
        Real hz = getGroundHeight(x, y, nullptr);
        Real wz = (x < 0.0f) ? 12.0f : -5.0f;
        if (waterZ) *waterZ = wz;
        if (terrainZ) *terrainZ = hz;
        return (wz > hz) ? TRUE : FALSE;
    }
};

class OracleThing final : public Thing {
public:
    int react_count = 0;
    Coord3D last_old_pos{};
    Real last_old_angle = 0.0f;
    explicit OracleThing(const ThingTemplate* tt) : Thing(tt) {}
    ~OracleThing() override = default;

    void applyTransform(const Matrix3D& mx) { setTransformMatrix(&mx); }
    Real calcRawHeightAboveTerrain() const { return calculateHeightAboveTerrain(); }

protected:
    void reactToTransformChange(const Matrix3D* /*oldMtx*/, const Coord3D* oldPos, Real oldAngle) override {
        ++react_count;
        if (oldPos) last_old_pos = *oldPos;
        last_old_angle = oldAngle;
    }
};

static const char* kSharedSetupINI =
R"(Science SCIENCE_ALPHA
  PrerequisiteSciences = None
  SciencePurchasePointCost = 1
  IsGrantable = Yes
  DisplayName = SCI:Alpha
  Description = DESC:Alpha
End

Science SCIENCE_BETA
  PrerequisiteSciences = SCIENCE_ALPHA
  SciencePurchasePointCost = 2
  IsGrantable = Yes
  DisplayName = SCI:Beta
  Description = DESC:Beta
End

Science SCIENCE_GAMMA
  PrerequisiteSciences = None
  SciencePurchasePointCost = 3
  IsGrantable = No
  DisplayName = SCI:Gamma
  Description = DESC:Gamma
End

FXList FX_One
End

FXList FX_Two
End

DamageFX DmgFXA
  AmountForMajorFX = DEFAULT 50
  MajorFX = DEFAULT FX_One
  MinorFX = DEFAULT FX_Two
  ThrottleTime = DEFAULT 100
End

DamageFX DmgFXB
  AmountForMajorFX = DEFAULT 120
  MajorFX = DEFAULT FX_Two
  ThrottleTime = DEFAULT 200
End

Armor ArmorA
  Armor = Default 100%
  Armor = EXPLOSION 50%
  Armor = SMALL_ARMS 25%
End

Armor ArmorB
  Armor = Default 80%
  Armor = EXPLOSION 120%
  Armor = ARMOR_PIERCING 60%
End

Armor ArmorC
  Armor = Default 40%
  Armor = SNIPER 150%
End

Weapon WpnA
  PrimaryDamage = 35
  AttackRange = 150
  DamageType = ARMOR_PIERCING
  DeathType = EXPLODED
  WeaponSpeed = 200
End

Weapon WpnB
  PrimaryDamage = 12
  AttackRange = 90
  DamageType = SMALL_ARMS
  DeathType = NORMAL
  WeaponSpeed = 300
End

Weapon WpnC
  PrimaryDamage = 80
  AttackRange = 220
  DamageType = EXPLOSION
  DeathType = EXPLODED
  WeaponSpeed = 120
End
)";

static void parse_shared_setup() {
    oracle::MemINI ini;
    ini.parse_blocks(kSharedSetupINI, INI_LOAD_OVERWRITE, [](INI* i, const char* tok) {
        if (std::strcmp(tok, "Science") == 0) {
            INI::parseScienceDefinition(i);
        } else if (std::strcmp(tok, "FXList") == 0) {
            FXListStore::parseFXListDefinition(i);
        } else if (std::strcmp(tok, "DamageFX") == 0) {
            DamageFXStore::parseDamageFXDefinition(i);
        } else if (std::strcmp(tok, "Armor") == 0) {
            ArmorStore::parseArmorDefinition(i);
        } else if (std::strcmp(tok, "Weapon") == 0) {
            WeaponStore::parseWeaponTemplateDefinition(i);
        }
    });
}

static uint32_t fnv1a_32(const void* data, size_t len, uint32_t seed = 2166136261u) {
    const uint8_t* p = static_cast<const uint8_t*>(data);
    uint32_t h = seed;
    for (size_t i = 0; i < len; ++i) {
        h ^= p[i];
        h *= 16777619u;
    }
    return h;
}

static uint32_t mix_u32(uint32_t h, uint32_t v) {
    return fnv1a_32(&v, sizeof(v), h);
}

static uint32_t mix_i32(uint32_t h, int32_t v) {
    return fnv1a_32(&v, sizeof(v), h);
}

static uint32_t mix_f32(uint32_t h, Real v) {
    return mix_u32(h, oracle::fbits(v));
}

static uint32_t mix_str(uint32_t h, const AsciiString& s) {
    uint32_t len = static_cast<uint32_t>(s.getLength());
    h = mix_u32(h, len);
    if (len > 0) {
        h = fnv1a_32(s.str(), len, h);
    }
    return h;
}

static uint32_t mix_wstr(uint32_t h, const UnicodeString& s) {
    uint32_t len = static_cast<uint32_t>(s.getLength());
    h = mix_u32(h, len);
    for (uint32_t i = 0; i < len; ++i) {
        uint16_t ch = static_cast<uint16_t>(s.getCharAt(static_cast<Int>(i)));
        h = mix_u32(h, ch);
    }
    return h;
}

static std::string armor_flags_to_ini(uint32_t mask) {
    static const char* kArmorFlags[] = {
        "VETERAN", "ELITE", "HERO", "PLAYER_UPGRADE",
        "WEAK_VERSUS_BASEDEFENSES", "SECOND_LIFE", "CRATE_UPGRADE_ONE", "CRATE_UPGRADE_TWO"
    };
    std::string s;
    for (int i = 0; i < 8; ++i) {
        if ((mask >> i) & 1u) {
            if (!s.empty()) s += " ";
            s += kArmorFlags[i];
        }
    }
    return s;
}

static std::string weapon_flags_to_ini(uint32_t mask) {
    static const char* kWeaponFlags[] = {
        "VETERAN", "ELITE", "HERO", "PLAYER_UPGRADE",
        "CRATE_UPGRADE_ONE", "CRATE_UPGRADE_TWO", "GARRISONED", "CAPTURING"
    };
    std::string s;
    for (int i = 0; i < 8; ++i) {
        if ((mask >> i) & 1u) {
            if (!s.empty()) s += " ";
            s += kWeaponFlags[i];
        }
    }
    return s;
}

static ArmorSetFlags make_armor_flags(uint32_t mask) {
    ArmorSetFlags f;
    for (int i = 0; i < ARMORSET_COUNT && i < 8; ++i) {
        if ((mask >> i) & 1u) {
            f.set(i, 1);
        }
    }
    return f;
}

static WeaponSetFlags make_weapon_flags(uint32_t mask) {
    WeaponSetFlags f;
    for (int i = 0; i < 8; ++i) {
        if ((mask >> i) & 1u) {
            f.set(i, 1);
        }
    }
    return f;
}

static uint32_t digest_thing_template(const ThingTemplate* base_tt) {
    const ThingTemplate* tt = static_cast<const ThingTemplate*>(base_tt->getFinalOverride());
    uint32_t h = 2166136261u;
    h = mix_u32(h, tt->getTemplateID());
    h = mix_str(h, tt->getName());
    h = mix_wstr(h, tt->getDisplayName());
    h = mix_i32(h, static_cast<Int>(tt->getDefaultRadarPriority()));
    h = mix_i32(h, tt->getRawTransportSlotCount());
    h = mix_f32(h, tt->getFenceWidth());
    h = mix_f32(h, tt->getFenceXOffset());
    h = mix_i32(h, tt->isBridge() ? 1 : 0);
    h = mix_i32(h, tt->isBridgeLike() ? 1 : 0);
    h = mix_f32(h, tt->friend_calcVisionRange());
    h = mix_f32(h, tt->friend_calcShroudClearingRange());
    h = mix_f32(h, tt->getShroudRevealToAllRange());
    h = mix_f32(h, tt->getPlacementViewAngle());
    h = mix_f32(h, tt->getFactoryExitWidth());
    h = mix_f32(h, tt->getFactoryExtraBibWidth());
    for (Int lvl = 0; lvl < LEVEL_COUNT; ++lvl) {
        h = mix_i32(h, tt->getSkillPointValue(lvl));
        h = mix_i32(h, tt->getExperienceValue(lvl));
        h = mix_i32(h, tt->getExperienceRequired(lvl));
    }
    h = mix_i32(h, tt->isTrainable() ? 1 : 0);
    h = mix_i32(h, tt->isEnterGuard() ? 1 : 0);
    h = mix_i32(h, tt->isHijackGuard() ? 1 : 0);
    h = mix_str(h, tt->getDefaultOwningSide());
    h = mix_i32(h, tt->getPrereqCount());
    h = mix_i32(h, static_cast<Int>(tt->getBuildable()));
    h = mix_u32(h, tt->*access_member(TTBuildCostTag()));
    h = mix_f32(h, tt->*access_member(TTBuildTimeTag()));
    h = mix_i32(h, tt->getRefundValue());
    h = mix_i32(h, static_cast<Int>(tt->getBuildCompletion()));
    h = mix_i32(h, tt->getEnergyProduction());
    h = mix_i32(h, tt->getEnergyBonus());
    h = mix_i32(h, tt->isPrerequisite() ? 1 : 0);
    h = mix_i32(h, tt->isBuildFacility() ? 1 : 0);
    h = mix_u32(h, static_cast<uint32_t>(tt->getDisplayColor()));
    h = mix_i32(h, static_cast<Int>(tt->getEditorSorting()));
    h = mix_i32(h, tt->isKindOf(KINDOF_STRUCTURE) ? 1 : 0);
    h = mix_i32(h, tt->isKindOf(KINDOF_VEHICLE) ? 1 : 0);
    h = mix_i32(h, tt->isKindOf(KINDOF_INFANTRY) ? 1 : 0);
    h = mix_i32(h, tt->isKindOf(KINDOF_STICK_TO_TERRAIN_SLOPE) ? 1 : 0);
    h = mix_str(h, tt->friend_getCommandSetString());
    h = mix_u32(h, static_cast<uint32_t>(tt->getBuildVariations().size()));
    for (const auto& bv : tt->getBuildVariations()) {
        h = mix_str(h, bv);
    }
    h = mix_i32(h, static_cast<Int>(tt->getBehaviorModuleInfo().getCount()));
    for (Int i = 0; i < tt->getBehaviorModuleInfo().getCount(); ++i) {
        h = mix_str(h, tt->getBehaviorModuleInfo().getNthName(static_cast<size_t>(i)));
        h = mix_str(h, tt->getBehaviorModuleInfo().getNthTag(static_cast<size_t>(i)));
    }
    for (Int i = 0; i < 5; ++i) {
        h = mix_str(h, tt->getUpgradeCameoName(i));
    }
    h = mix_f32(h, tt->getAssetScale());
    h = mix_f32(h, tt->getInstanceScaleFuzziness());
    h = mix_f32(h, tt->getStructureRubbleHeight());
    const GeometryInfo& gi = tt->getTemplateGeometryInfo();
    h = mix_i32(h, static_cast<Int>(gi.getGeomType()));
    h = mix_i32(h, gi.getIsSmall() ? 1 : 0);
    h = mix_f32(h, gi.getMajorRadius());
    h = mix_f32(h, gi.getMinorRadius());
    h = mix_f32(h, gi.getBoundingCircleRadius());
    h = mix_i32(h, static_cast<Int>(tt->getShadowType()));
    h = mix_f32(h, tt->getShadowSizeX());
    h = mix_f32(h, tt->getShadowSizeY());
    h = mix_f32(h, tt->getShadowOffsetX());
    h = mix_f32(h, tt->getShadowOffsetY());
    h = mix_str(h, tt->getShadowTextureName());
    h = mix_u32(h, tt->getOcclusionDelay());
    h = mix_u32(h, tt->getThreatValue());
    h = mix_u32(h, tt->getMaxSimultaneousOfType());
    h = mix_str(h, tt->getMaxSimultaneousLinkKey() == NAMEKEY_INVALID
        ? AsciiString::TheEmptyString
        : TheNameKeyGenerator->keyToName(tt->getMaxSimultaneousLinkKey()));
    h = mix_i32(h, static_cast<Int>(tt->getCrushableLevel()));
    h = mix_i32(h, static_cast<Int>(tt->getCrusherLevel()));
    h = mix_i32(h, tt->canPossiblyHaveAnyWeapon() ? 1 : 0);
    h = mix_i32(h, tt->isBuildableItem() ? 1 : 0);
    const FXList* fx = tt->getPerUnitFX(AsciiString("FXTest"));
    h = mix_i32(h, fx != nullptr ? 1 : 0);
    h = mix_str(h, tt->getPerUnitSound(AsciiString("MoveTest"))->getEventName());
    return h;
}

static void append_u32_array(std::ostringstream& ss, const uint32_t* arr, size_t n) {
    ss << "[";
    for (size_t i = 0; i < n; ++i) {
        if (i > 0) ss << ",";
        ss << arr[i];
    }
    ss << "]";
}

static void append_i32_array(std::ostringstream& ss, const int32_t* arr, size_t n) {
    ss << "[";
    for (size_t i = 0; i < n; ++i) {
        if (i > 0) ss << ",";
        ss << arr[i];
    }
    ss << "]";
}

static void append_str_array(std::ostringstream& ss, const std::vector<std::string>& arr) {
    ss << "[";
    for (size_t i = 0; i < arr.size(); ++i) {
        if (i > 0) ss << ",";
        ss << "\"" << oracle::json_escape(arr[i]) << "\"";
    }
    ss << "]";
}

static std::string armor_coef_str(const ArmorTemplate* at) {
    if (!at) return "null";
    uint32_t h = mix_f32(2166136261u, at->adjustDamage(DAMAGE_EXPLOSION, 100.0f));
    h = mix_f32(h, at->adjustDamage(DAMAGE_SMALL_ARMS, 100.0f));
    h = mix_f32(h, at->adjustDamage(DAMAGE_ARMOR_PIERCING, 100.0f));
    return std::to_string(h);
}

} // namespace

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/PKT-398.json";

    oracle::init_memory();
    TheWritableGlobalData = NEW GlobalData;
    TheNameKeyGenerator = NEW NameKeyGenerator;
    TheNameKeyGenerator->init();
    TheGameLogic = NEW GameLogic;

    static OracleGameText g_game_text;
    TheGameText = &g_game_text;

    static OracleTerrainLogic g_terrain_logic;
    TheTerrainLogic = &g_terrain_logic;

    TheScienceStore = NEW ScienceStore;
    TheFXListStore = NEW FXListStore;
    TheDamageFXStore = NEW DamageFXStore;
    TheArmorStore = NEW ArmorStore;
    TheWeaponStore = NEW WeaponStore;
    TheRankInfoStore = NEW RankInfoStore;
    TheRankInfoStore->init();
    parse_shared_setup();

    const char* files_sha[] = {
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingSort.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ArmorSet.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/SparseMatchFinder.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectScriptStatusBits.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectTypes.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/ObjectTypes.cpp",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Overridable.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Override.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Thing.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/Thing/Thing.cpp",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/Thing/ThingFactory.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/Thing/ThingTemplate.cpp"
    };

    std::ostringstream out;
    out << "{\n"
        << "  \"packet_id\": \"PKT-398\",\n"
        << "  \"packets\": [\"PKT-078\", \"PKT-139\", \"PKT-392\", \"PKT-393\", \"PKT-394\", \"PKT-395\", \"PKT-396\", \"PKT-397\", \"PKT-398\", \"PKT-408\"],\n"
        << "  \"modules\": [\"ThingSort\", \"ArmorSet\", \"ObjectScriptStatusBits\", \"ObjectTypes\", \"Overridable\", \"Override\", \"Thing\", \"ThingFactory\", \"ThingTemplate\", \"SparseMatchFinder\"],\n"
        << "  \"vector_count\": 1000,\n"
        << "  \"source_sha256\": {\n";
    for (size_t i = 0; i < sizeof(files_sha) / sizeof(files_sha[0]); ++i) {
        out << "    \"" << files_sha[i] << "\": \"" << oracle::sha256_file(files_sha[i]) << "\""
            << (i + 1 < sizeof(files_sha) / sizeof(files_sha[0]) ? "," : "") << "\n";
    }
    out << "  },\n";
    out << "  \"setup\": {\n"
        << "    \"shared_ini\": \"" << oracle::json_escape(kSharedSetupINI) << "\",\n"
        << "    \"status_bits\": ["
        << OBJECT_STATUS_SCRIPT_DISABLED << ","
        << OBJECT_STATUS_SCRIPT_UNPOWERED << ","
        << OBJECT_STATUS_SCRIPT_UNSELLABLE << ","
        << OBJECT_STATUS_SCRIPT_UNSTEALTHED << ","
        << OBJECT_STATUS_SCRIPT_TARGETABLE << "],\n"
        << "    \"editor_sorting_count\": " << ES_NUM_SORTING_TYPES << "\n"
        << "  },\n"
        << "  \"vectors\": [\n";

    static const char* kRadarPrios[] = { "NOT_ON_RADAR", "STRUCTURE", "UNIT", "LOCAL_UNIT_ONLY" };
    static const char* kSortings[] = { "STRUCTURE", "INFANTRY", "VEHICLE", "MISC_MAN_MADE", "DEBRIS" };
    static const char* kGeoms[] = { "SPHERE", "CYLINDER", "BOX" };
    static const char* kShadows[] = { "SHADOW_DECAL", "SHADOW_VOLUME", "SHADOW_PROJECTION" };
    static const char* kArmorNames[] = { "ArmorA", "ArmorB", "ArmorC" };
    static const char* kWpnNames[] = { "WpnA", "WpnB", "WpnC" };

    oracle::MemINI mem_ini;
    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed ^ 0x517cc1b7u;
        if (rng == 0) rng = 0x9e3779b9u;

        // Per-seed GlobalData / GameLogic tunables
        UnsignedInt occ_delay = 30u * (1u + (oracle::splitmix32(rng) % 5u));
        Real gravity = -0.75f - 0.25f * static_cast<Real>(oracle::splitmix32(rng) % 5u);
        Real low_e_pen = 0.25f * static_cast<Real>(1u + (oracle::splitmix32(rng) % 4u));
        Real min_e_spd = 0.1f * static_cast<Real>(1u + (oracle::splitmix32(rng) % 5u));
        Real max_e_spd = 0.6f + 0.1f * static_cast<Real>(oracle::splitmix32(rng) % 4u);
        Real mult_fac = 0.5f + 0.125f * static_cast<Real>(oracle::splitmix32(rng) % 5u);
        UnsignedShort sw_restr = static_cast<UnsignedShort>(1u + (oracle::splitmix32(rng) % 4u));

        TheWritableGlobalData->m_defaultOcclusionDelay = occ_delay;
        TheWritableGlobalData->m_gravity = gravity;
        TheWritableGlobalData->m_LowEnergyPenaltyModifier = low_e_pen;
        TheWritableGlobalData->m_MinLowEnergyProductionSpeed = min_e_spd;
        TheWritableGlobalData->m_MaxLowEnergyProductionSpeed = max_e_spd;
        TheWritableGlobalData->m_MultipleFactory = mult_fac;
        TheGameLogic->*access_member(GLSuperweaponTag()) = sw_restr;

        TheModuleFactory = NEW ModuleFactory;
        TheModuleFactory->init();
        TheThingFactory = NEW ThingFactory;

        bool is_struct = (seed % 3 == 0);
        bool is_trainable = (seed % 2 == 0);
        bool is_cc = (seed % 4 == 1);
        bool sw_mode = (seed % 5 == 0);
        bool explicit_skill = (seed % 2 == 1);
        bool explicit_shadow_tex = (seed % 3 != 0);
        bool unit_has_weapons = (seed % 7 != 0);
        bool has_slope_kindof = (seed % 2 == 0);

        Int build_cost = static_cast<Int>(300 + (oracle::splitmix32(rng) % 25) * 50);
        Real build_time = static_cast<Real>(2 + (oracle::splitmix32(rng) % 15));
        Int refund_val = static_cast<Int>((oracle::splitmix32(rng) % 6) * 50);
        Int e_prod = static_cast<Int>(oracle::splitmix32(rng) % 11) - 5;
        Int e_bonus = static_cast<Int>(oracle::splitmix32(rng) % 5);
        Real vision = static_cast<Real>(80 + (oracle::splitmix32(rng) % 20) * 10);
        Real shroud = static_cast<Real>(100 + (oracle::splitmix32(rng) % 20) * 10);
        Real reveal_all = static_cast<Real>(oracle::splitmix32(rng) % 3) * 25.0f;
        Real view_deg = static_cast<Real>(oracle::splitmix32(rng) % 180) - 90.0f;
        Real geom_r = static_cast<Real>(8 + (oracle::splitmix32(rng) % 20));
        Real geom_min = static_cast<Real>(4 + (oracle::splitmix32(rng) % 12));
        Real geom_h = static_cast<Real>(6 + (oracle::splitmix32(rng) % 18));
        int geom_idx = static_cast<int>(oracle::splitmix32(rng) % 3);
        int sort_idx = static_cast<int>(oracle::splitmix32(rng) % 5);
        int prio_idx = static_cast<int>(oracle::splitmix32(rng) % 4);
        int shadow_idx = static_cast<int>(oracle::splitmix32(rng) % 3);
        uint32_t crusher = oracle::splitmix32(rng) % 4;
        uint32_t crushable = 1 + (oracle::splitmix32(rng) % 4);
        uint32_t max_sim = 1 + (oracle::splitmix32(rng) % 6);
        int a1 = static_cast<int>(oracle::splitmix32(rng) % 3);
        int a2 = (a1 + 1) % 3;
        int w1 = static_cast<int>(oracle::splitmix32(rng) % 3);
        int w2 = (w1 + 1) % 3;

        std::ostringstream ini_ss;
        ini_ss << "Object DefaultThingTemplate\n"
               << "  DisplayName = OBJECT:Default\n"
               << "  RadarPriority = STRUCTURE\n"
               << "  VisionRange = 55\n"
               << "  BuildCost = 100\n"
               << "  BuildTime = 5\n"
               << "  ArmorSet\n"
               << "    Armor = ArmorA\n"
               << "    DamageFX = DmgFXA\n"
               << "  End\n"
               << "  WeaponSet\n"
               << "    Weapon = PRIMARY WpnA\n"
               << "  End\n"
               << "  Body = InactiveBody Tag_DefBody\n"
               << "  End\n"
               << "  InheritableModule\n"
               << "    Behavior = AutoHealBehavior ModuleTag_DefaultAutoHealBehavior\n"
               << "      StartsActive = Yes\n"
               << "      HealingAmount = 4\n"
               << "      HealingDelay = 200\n"
               << "    End\n"
               << "  End\n"
               << "  OverrideableByLikeKind\n"
               << "    Behavior = LifetimeUpdate Tag_DefLike\n"
               << "      MinLifetime = 1000\n"
               << "      MaxLifetime = 2000\n"
               << "    End\n"
               << "  End\n"
               << "End\n"
               << "Object FacilityA\n"
               << "  DisplayName = OBJECT:FacA\n"
               << "  KindOf = STRUCTURE" << (is_cc ? " COMMANDCENTER" : "") << "\n"
               << "  Buildable = Yes\n"
               << "  BuildCost = 2000\n"
               << "  BuildTime = 10\n"
               << "End\n"
               << "Object FacilityB\n"
               << "  DisplayName = OBJECT:FacB\n"
               << "  KindOf = STRUCTURE\n"
               << "  Buildable = Yes\n"
               << "  BuildCost = 1500\n"
               << "  BuildTime = 8\n"
               << "End\n"
               << "Object AltVarUnit\n"
               << "  DisplayName = OBJECT:AltVar\n"
               << "  KindOf = VEHICLE\n"
               << "  Buildable = Yes\n"
               << "  BuildCost = 450\n"
               << "  BuildTime = 4\n"
               << "End\n"
               << "Object UnitMain\n"
               << "  DisplayName = OBJECT:UnitMain\n"
               << "  RadarPriority = " << kRadarPrios[prio_idx] << "\n"
               << "  TransportSlotCount = " << (1 + (seed % 5)) << "\n"
               << "  FenceWidth = " << (12.5f + static_cast<Real>(seed % 4)) << "\n"
               << "  FenceXOffset = " << (2.0f + static_cast<Real>(seed % 3)) << "\n"
               << "  IsBridge = " << ((seed % 6 == 0) ? "Yes" : "No") << "\n"
               << "  ArmorSet\n"
               << "    Armor = " << kArmorNames[a1] << "\n"
               << "    DamageFX = DmgFXA\n"
               << "  End\n"
               << "  ArmorSet\n"
               << "    Conditions = VETERAN PLAYER_UPGRADE\n"
               << "    Armor = " << kArmorNames[a2] << "\n"
               << "    DamageFX = DmgFXB\n"
               << "  End\n";
        if (unit_has_weapons) {
            ini_ss << "  WeaponSet\n"
                   << "    Weapon = PRIMARY " << kWpnNames[w1] << "\n"
                   << "    Weapon = SECONDARY " << kWpnNames[w2] << "\n"
                   << "    AutoChooseSources = PRIMARY FROM_PLAYER FROM_AI\n"
                   << "    PreferredAgainst = SECONDARY VEHICLE STRUCTURE\n"
                   << "    ShareWeaponReloadTime = Yes\n"
                   << "  End\n"
                   << "  WeaponSet\n"
                   << "    Conditions = HERO\n"
                   << "    Weapon = PRIMARY " << kWpnNames[w2] << "\n"
                   << "    WeaponLockSharedAcrossSets = Yes\n"
                   << "  End\n";
        } else {
            ini_ss << "  WeaponSet\n"
                   << "    Weapon = PRIMARY None\n"
                   << "  End\n";
        }
        ini_ss << "  VisionRange = " << vision << "\n"
               << "  ShroudClearingRange = " << shroud << "\n"
               << "  ShroudRevealToAllRange = " << reveal_all << "\n"
               << "  PlacementViewAngle = " << view_deg << "\n"
               << "  FactoryExitWidth = " << (25.0f + static_cast<Real>(seed % 10)) << "\n"
               << "  FactoryExtraBibWidth = " << (5.0f + static_cast<Real>(seed % 5)) << "\n";
        if (explicit_skill) {
            ini_ss << "  SkillPointValue = 10 20 30 40\n";
        }
        ini_ss << "  ExperienceValue = 50 100 150 250\n"
               << "  ExperienceRequired = 0 100 300 700\n"
               << "  IsTrainable = " << (is_trainable ? "Yes" : "No") << "\n"
               << "  EnterGuard = " << ((seed % 3 == 0) ? "Yes" : "No") << "\n"
               << "  HijackGuard = " << ((seed % 4 == 0) ? "Yes" : "No") << "\n"
               << "  Side = America\n"
               << "  Prerequisites\n"
               << "    Object = FacilityA FacilityB\n"
               << "    Science = SCIENCE_ALPHA\n"
               << "  End\n"
               << "  Buildable = Yes\n"
               << "  BuildCost = " << build_cost << "\n"
               << "  BuildTime = " << build_time << "\n"
               << "  RefundValue = " << refund_val << "\n"
               << "  BuildCompletion = " << ((seed % 2 == 0) ? "APPEARS_AT_RALLY_POINT" : "PLACED_BY_PLAYER") << "\n"
               << "  EnergyProduction = " << e_prod << "\n"
               << "  EnergyBonus = " << e_bonus << "\n"
               << "  IsForbidden = " << ((seed % 9 == 0) ? "Yes" : "No") << "\n"
               << "  IsPrerequisite = No\n"
               << "  DisplayColor = R:10 G:20 B:240\n"
               << "  EditorSorting = " << kSortings[sort_idx] << "\n"
               << "  KindOf = SELECTABLE " << (is_struct ? "STRUCTURE IMMOBILE" : "VEHICLE") << "\n"
               << "  CommandSet = CmdSetUnitMain\n"
               << "  BuildVariations = AltVarUnit\n"
               << "  Body = ActiveBody Tag_MainBody\n"
               << "    MaxHealth = 500\n"
               << "    InitialHealth = 500\n"
               << "  End\n"
               << "  Behavior = DestroyDie Tag_MainDie\n"
               << "  End\n"
               << "  Behavior = LifetimeUpdate Tag_OverrideLike\n"
               << "    MinLifetime = 300\n"
               << "    MaxLifetime = 600\n"
               << "  End\n"
               << "  VoiceSelect = NoSound\n"
               << "  UnitSpecificSounds\n"
               << "    MoveTest = NoSound\n"
               << "  End\n"
               << "  UnitSpecificFX\n"
               << "    FXTest = FX_One\n"
               << "  End\n"
               << "  Scale = " << (1.0f + 0.1f * static_cast<Real>(seed % 5)) << "\n"
               << "  Geometry = " << kGeoms[geom_idx] << "\n"
               << "  GeometryIsSmall = " << ((seed % 4 == 0) ? "Yes" : "No") << "\n"
               << "  GeometryMajorRadius = " << geom_r << "\n"
               << "  GeometryMinorRadius = " << geom_min << "\n"
               << "  GeometryHeight = " << geom_h << "\n"
               << "  Shadow = " << kShadows[shadow_idx] << "\n"
               << "  ShadowSizeX = 12\n"
               << "  ShadowSizeY = 14\n"
               << "  ShadowOffsetX = 1\n"
               << "  ShadowOffsetY = -1\n";
        if (explicit_shadow_tex) {
            ini_ss << "  ShadowTexture = CustomShadow\n";
        }
        ini_ss << "  InstanceScaleFuzziness = 0.05\n"
               << "  StructureRubbleHeight = 2\n"
               << "  ThreatValue = " << (10 + (seed % 50)) << "\n";
        if (sw_mode) {
            ini_ss << "  MaxSimultaneousOfType = DeterminedBySuperweaponRestriction\n";
        } else {
            ini_ss << "  MaxSimultaneousOfType = " << max_sim << "\n";
        }
        ini_ss << "  MaxSimultaneousLinkKey = LinkKeyShared\n"
               << "  CrusherLevel = " << crusher << "\n"
               << "  CrushableLevel = " << crushable << "\n"
               << "End\n"
               << "Object SlopeUnit\n"
               << "  DisplayName = OBJECT:SlopeUnit\n"
               << "  KindOf = INFANTRY" << (has_slope_kindof ? " STICK_TO_TERRAIN_SLOPE" : "") << "\n"
               << "  BuildCost = 200\n"
               << "  BuildTime = 3\n"
               << "End\n"
               << "ObjectReskin UnitReskin UnitMain\n"
               << "  Geometry = BOX\n"
               << "  GeometryMajorRadius = " << (geom_r + 3.0f) << "\n"
               << "  GeometryMinorRadius = " << (geom_min + 2.0f) << "\n"
               << "  GeometryHeight = " << (geom_h + 1.0f) << "\n"
               << "  FenceWidth = 33\n"
               << "  FenceXOffset = 4.5\n"
               << "  MaxSimultaneousOfType = 2\n"
               << "  MaxSimultaneousLinkKey = LinkKeyReskin\n"
               << "End\n"
               << "ObjectReskin UnitReskin2 UnitMain\n"
               << "  Geometry = SPHERE\n"
               << "  GeometryMajorRadius = " << (geom_r + 1.0f) << "\n"
               << "End\n";

        std::string init_ini = ini_ss.str();
        mem_ini.parse_blocks(init_ini, INI_LOAD_OVERWRITE, [](INI* i, const char* tok) {
            if (std::strcmp(tok, "Object") == 0) {
                INI::parseObjectDefinition(i);
            } else if (std::strcmp(tok, "ObjectReskin") == 0) {
                INI::parseObjectReskinDefinition(i);
            }
        });
        TheThingFactory->postProcessLoad();

        const ThingTemplate* def_tt = TheThingFactory->findTemplate("DefaultThingTemplate");
        const ThingTemplate* fac_a = TheThingFactory->findTemplate("FacilityA");
        const ThingTemplate* fac_b = TheThingFactory->findTemplate("FacilityB");
        const ThingTemplate* alt_tt = TheThingFactory->findTemplate("AltVarUnit");
        const ThingTemplate* main_tt = TheThingFactory->findTemplate("UnitMain");
        const ThingTemplate* slope_tt = TheThingFactory->findTemplate("SlopeUnit");
        const ThingTemplate* reskin_tt = TheThingFactory->findTemplate("UnitReskin");
        const ThingTemplate* reskin2_tt = TheThingFactory->findTemplate("UnitReskin2");

        uint32_t pre_ovr_crc = digest_thing_template(main_tt);
        uint32_t reskin_crc = digest_thing_template(reskin_tt);

        // Apply Map.ini override + MapOnlyObj
        Int ovr_cost = build_cost + 125;
        Real ovr_time = build_time + 1.5f;
        Real ovr_vision = vision + 35.0f;
        std::ostringstream ovr_ss;
        ovr_ss << "Object UnitMain\n"
               << "  BuildCost = " << ovr_cost << "\n"
               << "  BuildTime = " << ovr_time << "\n"
               << "  VisionRange = " << ovr_vision << "\n"
               << "  Prerequisites\n"
               << "    Object = FacilityB\n"
               << "    Science = SCIENCE_BETA\n"
               << "  End\n"
               << "  RemoveModule Tag_OverrideLike\n"
               << "  AddModule\n"
               << "    Behavior = KeepObjectDie Tag_OvrDie\n"
               << "    End\n"
               << "  End\n"
               << "End\n"
               << "Object MapOnlyObj\n"
               << "  DisplayName = OBJECT:MapOnly\n"
               << "  KindOf = VEHICLE\n"
               << "  BuildCost = 777\n"
               << "  BuildTime = 6\n"
               << "End\n";
        std::string ovr_ini = ovr_ss.str();
        mem_ini.parse_blocks(ovr_ini, INI_LOAD_CREATE_OVERRIDES, [](INI* i, const char* tok) {
            if (std::strcmp(tok, "Object") == 0) {
                INI::parseObjectDefinition(i);
            } else if (std::strcmp(tok, "ObjectReskin") == 0) {
                INI::parseObjectReskinDefinition(i);
            }
        });

        const ThingTemplate* main_final = static_cast<const ThingTemplate*>(main_tt->getFinalOverride());
        const ThingTemplate* map_only_tt = TheThingFactory->findTemplate("MapOnlyObj");
        uint32_t post_ovr_crc = digest_thing_template(main_tt);

        Int eq_mask = 0;
        if (main_tt->isEquivalentTo(main_final)) eq_mask |= 1;
        if (main_tt->isEquivalentTo(reskin_tt)) eq_mask |= 2;
        if (reskin_tt->isEquivalentTo(reskin2_tt)) eq_mask |= 4;
        if (main_tt->isEquivalentTo(alt_tt)) eq_mask |= 8;
        if (main_tt->isEquivalentTo(fac_a)) eq_mask |= 16;
        if (fac_a->isBuildFacility()) eq_mask |= 32;
        if (fac_b->isBuildFacility()) eq_mask |= 64;
        if (map_only_tt != nullptr) eq_mask |= 128;

        // Buildable override check
        if (seed % 4 == 0) {
            (TheGameLogic->*access_member(GLBuildableMapTag()))[main_tt->getName()] = BSTATUS_ONLY_BY_AI;
        }
        Int ovr_buildable = static_cast<Int>(main_final->getBuildable());
        (TheGameLogic->*access_member(GLBuildableMapTag())).clear();

        // SparseMatchFinder queries (4 ArmorSet queries + 4 WeaponSet queries)
        uint32_t am_q[4];
        uint32_t wm_q[4];
        uint32_t am_hash = 2166136261u;
        uint32_t wm_hash = 2166136261u;
        for (int q = 0; q < 4; ++q) {
            am_q[q] = oracle::splitmix32(rng) & 0xffu;
            wm_q[q] = oracle::splitmix32(rng) & 0xffu;
            const ArmorTemplateSet* ats = main_final->findArmorTemplateSet(make_armor_flags(am_q[q]));
            uint32_t ac = ats && ats->getArmorTemplate()
                ? std::strtoul(armor_coef_str(ats->getArmorTemplate()).c_str(), nullptr, 10)
                : 0u;
            am_hash = mix_u32(am_hash, ac);
            am_hash = mix_i32(am_hash, (ats && ats->getDamageFX()) ? 1 : 0);

            const WeaponTemplateSet* wts = main_final->findWeaponTemplateSet(make_weapon_flags(wm_q[q]));
            wm_hash = mix_i32(wm_hash, wts ? 1 : 0);
            if (wts) {
                wm_hash = mix_i32(wm_hash, wts->isSharedReloadTime() ? 1 : 0);
                wm_hash = mix_i32(wm_hash, wts->isWeaponLockSharedAcrossSets() ? 1 : 0);
                wm_hash = mix_i32(wm_hash, wts->getNth(PRIMARY_WEAPON) ? 1 : 0);
                wm_hash = mix_i32(wm_hash, wts->getNth(SECONDARY_WEAPON) ? 1 : 0);
            }
        }

        // Player::calcCostToBuild & calcTimeToBuild (using final override template so override build values are exercised)
        Real p_cost_pct = -0.2f + 0.1f * static_cast<Real>(oracle::splitmix32(rng) % 5);
        Real p_time_pct = -0.25f + 0.125f * static_cast<Real>(oracle::splitmix32(rng) % 5);
        Real p_ko_pct = -0.15f + 0.1f * static_cast<Real>(oracle::splitmix32(rng) % 4);
        Real p_hc[4] = {
            0.8f + 0.1f * static_cast<Real>(oracle::splitmix32(rng) % 5),
            0.85f + 0.1f * static_cast<Real>(oracle::splitmix32(rng) % 4),
            0.75f + 0.125f * static_cast<Real>(oracle::splitmix32(rng) % 5),
            0.9f + 0.1f * static_cast<Real>(oracle::splitmix32(rng) % 4),
        };
        Int p_e_prod = static_cast<Int>(oracle::splitmix32(rng) % 12);
        Int p_e_cons = static_cast<Int>(oracle::splitmix32(rng) % 12);

        Player player(0);
        NameKeyType unit_key = NAMEKEY(main_final->getName());
        (player.*access_member(PlayerCostMapTag()))[unit_key] = p_cost_pct;
        (player.*access_member(PlayerTimeMapTag()))[unit_key] = p_time_pct;
        KindOfPercentProductionChange* kpc = newInstance(KindOfPercentProductionChange);
        kpc->m_kindOf = KINDOFMASK_NONE;
        kpc->m_kindOf.set(is_struct ? KINDOF_STRUCTURE : KINDOF_VEHICLE);
        kpc->m_percent = p_ko_pct;
        kpc->m_ref = 1;
        (player.*access_member(PlayerKindOfProdTag())).push_back(kpc);

        auto& hc_arr = (player.*access_member(PlayerHandicapTag())).*access_member(HandicapArrayTag());
        hc_arr[Handicap::BUILDCOST][0] = p_hc[0];
        hc_arr[Handicap::BUILDCOST][1] = p_hc[1];
        hc_arr[Handicap::BUILDTIME][0] = p_hc[2];
        hc_arr[Handicap::BUILDTIME][1] = p_hc[3];

        Energy& p_en = player.*access_member(PlayerEnergyTag());
        p_en.*access_member(EnergyOwnerTag()) = nullptr;
        p_en.*access_member(EnergyProdTag()) = p_e_prod;
        p_en.*access_member(EnergyConsTag()) = p_e_cons;

        Int calc_cost = main_final->calcCostToBuild(&player);
        Int calc_time = main_final->calcTimeToBuild(&player);

        // Clean up KindOfPercentProductionChange explicitly so Patient Player dtor won't leak.
        for (auto* ptr : (player.*access_member(PlayerKindOfProdTag()))) {
            deleteInstance(ptr);
        }
        (player.*access_member(PlayerKindOfProdTag())).clear();

        // Thing & Transform tests (normal template + slope template)
        Real th_x = -40.0f + 10.0f * static_cast<Real>(oracle::splitmix32(rng) % 9);
        Real th_y = -30.0f + 10.0f * static_cast<Real>(oracle::splitmix32(rng) % 7);
        Real th_z = 5.0f + 3.0f * static_cast<Real>(oracle::splitmix32(rng) % 6);
        Real th_z1 = th_z + 4.5f;
        Real th_ang = -3.0f + 0.5f * static_cast<Real>(oracle::splitmix32(rng) % 13);
        Real th_pt[3] = {
            1.5f + static_cast<Real>(oracle::splitmix32(rng) % 5),
            -2.0f + static_cast<Real>(oracle::splitmix32(rng) % 5),
            0.5f + static_cast<Real>(oracle::splitmix32(rng) % 4),
        };

        OracleThing th(main_tt);
        Bool pos_before = th.isPositioned();
        Coord3D c0{th_x, th_y, th_z};
        th.setPosition(&c0);
        Bool pos_after = th.isPositioned();
        Real h_terr0 = th.getHeightAboveTerrain();
        Real h_water0 = th.getHeightAboveTerrainOrWater();
        Bool is_above0 = th.isAboveTerrain();
        Bool sig_above0 = th.isSignificantlyAboveTerrain();
        th.setPositionZ(th_z1);
        Real h_terr1 = th.getHeightAboveTerrain();
        Real h_water1 = th.getHeightAboveTerrainOrWater();
        th.setOrientation(th_ang);
        const Coord3D* dir2 = th.getUnitDirectionVector2D();
        Coord3D dir3{};
        th.getUnitDirectionVector3D(dir3);
        Coord3D bone_in{th_pt[0], th_pt[1], th_pt[2]};
        Matrix3D bone_mtx;
        bone_mtx.Make_Identity();
        bone_mtx.Rotate_Z(0.25f);
        bone_mtx.Translate(2.0f, -1.0f, 3.0f);
        Coord3D world_pt{};
        Matrix3D world_mtx;
        th.convertBonePosToWorldPos(&bone_in, &bone_mtx, &world_pt, &world_mtx);
        Coord3D xform_pt{};
        th.transformPoint(&bone_in, &xform_pt);

        uint32_t th_mtx_in[12];
        Matrix3D custom_mx;
        custom_mx.Make_Identity();
        custom_mx.Rotate_Z(th_ang * 0.5f);
        custom_mx.Translate(th_x + 5.0f, th_y - 3.0f, th_z + 2.0f);
        for (int r = 0; r < 3; ++r) {
            for (int c = 0; c < 4; ++c) {
                th_mtx_in[r * 4 + c] = oracle::fbits(custom_mx[r][c]);
            }
        }
        th.applyTransform(custom_mx);
        Real cached_ang_after_mx = th.getOrientation();
        const Coord3D* cached_pos_after_mx = th.getPosition();

        uint32_t th_hash = mix_i32(2166136261u, pos_before ? 1 : 0);
        th_hash = mix_i32(th_hash, pos_after ? 1 : 0);
        th_hash = mix_f32(th_hash, h_terr0);
        th_hash = mix_f32(th_hash, h_water0);
        th_hash = mix_i32(th_hash, is_above0 ? 1 : 0);
        th_hash = mix_i32(th_hash, sig_above0 ? 1 : 0);
        th_hash = mix_f32(th_hash, h_terr1);
        th_hash = mix_f32(th_hash, h_water1);
        th_hash = mix_f32(th_hash, dir2->x);
        th_hash = mix_f32(th_hash, dir2->y);
        th_hash = mix_f32(th_hash, dir3.x);
        th_hash = mix_f32(th_hash, dir3.y);
        th_hash = mix_f32(th_hash, dir3.z);
        th_hash = mix_f32(th_hash, world_pt.x);
        th_hash = mix_f32(th_hash, world_pt.y);
        th_hash = mix_f32(th_hash, world_pt.z);
        th_hash = mix_f32(th_hash, xform_pt.x);
        th_hash = mix_f32(th_hash, xform_pt.y);
        th_hash = mix_f32(th_hash, xform_pt.z);
        th_hash = mix_f32(th_hash, cached_ang_after_mx);
        th_hash = mix_f32(th_hash, cached_pos_after_mx->x);
        th_hash = mix_f32(th_hash, cached_pos_after_mx->y);
        th_hash = mix_f32(th_hash, cached_pos_after_mx->z);
        th_hash = mix_i32(th_hash, th.react_count);

        OracleThing slope_th(slope_tt);
        slope_th.setPosition(&c0);
        slope_th.setPositionZ(th_z1);
        slope_th.setOrientation(th_ang);
        Coord3D slope_dir3{};
        slope_th.getUnitDirectionVector3D(slope_dir3);
        uint32_t slope_hash = mix_f32(2166136261u, slope_th.getPosition()->x);
        slope_hash = mix_f32(slope_hash, slope_th.getPosition()->y);
        slope_hash = mix_f32(slope_hash, slope_th.getPosition()->z);
        slope_hash = mix_f32(slope_hash, slope_th.getOrientation());
        slope_hash = mix_f32(slope_hash, slope_dir3.x);
        slope_hash = mix_f32(slope_hash, slope_dir3.y);
        slope_hash = mix_f32(slope_hash, slope_dir3.z);
        slope_hash = mix_i32(slope_hash, slope_th.react_count);

        // ObjectTypes tests
        static const char* kOtPool[] = { "UnitMain", "FacilityA", "MissingUnit", "UnitReskin", "AltVarUnit" };
        std::vector<std::string> ot_names;
        for (int k = 0; k < 4; ++k) {
            ot_names.push_back(kOtPool[(seed + static_cast<uint32_t>(k)) % 5]);
        }
        std::string ot_rm = ot_names[seed % 4];
        std::string ot_grp = "Group_" + std::to_string(seed);
        ObjectTypes* ot = newInstance(ObjectTypes)(AsciiString(ot_grp.c_str()));
        for (const auto& nm : ot_names) {
            ot->addObjectType(AsciiString(nm.c_str()));
        }
        ot->addObjectType(AsciiString(ot_names[0].c_str())); // intentional dup guard
        ot->removeObjectType(AsciiString(ot_rm.c_str()));

        Int ot_in_mask = 0;
        if (ot->isInSet(main_tt)) ot_in_mask |= 1;
        if (ot->isInSet(fac_a)) ot_in_mask |= 2;
        if (ot->isInSet(reskin_tt)) ot_in_mask |= 4;
        if (ot->isInSet(alt_tt)) ot_in_mask |= 8;
        std::vector<const ThingTemplate*> resolved_tmpls;
        std::vector<Int> counts;
        Int ot_prep_cnt = ot->prepForPlayerCounting(resolved_tmpls, counts);

        oracle::MemoryXfer otsave(XFER_SAVE);
        otsave.open(AsciiString("OT"));
        oracle::call_snapshot_xfer(ot, &otsave);
        otsave.close();

        ObjectTypes* ot_loaded = newInstance(ObjectTypes);
        oracle::MemoryXfer otload(XFER_LOAD);
        otload.bytes = otsave.bytes;
        otload.open(AsciiString("OT"));
        oracle::call_snapshot_xfer(ot_loaded, &otload);
        otload.close();

        XferCRC otxcrc;
        otxcrc.open("OTCRC");
        oracle::call_snapshot_crc(ot_loaded, &otxcrc);
        otxcrc.close();

        deleteInstance(ot_loaded);
        deleteInstance(ot);

        // Capture Seed-0 snapshot values BEFORE TheThingFactory->reset() destroys override templates.
        auto join_mods = [](const ThingTemplate* tt) {
            std::string acc;
            const ModuleInfo& mi = tt->getBehaviorModuleInfo();
            for (Int i = 0; i < mi.getCount(); ++i) {
                if (i > 0) acc += ",";
                acc += mi.getNthName(static_cast<size_t>(i)).str();
                acc += ":";
                acc += mi.getNthTag(static_cast<size_t>(i)).str();
            }
            return acc;
        };
        UnsignedShort s0_def_tid = 0, s0_main_tid = 0, s0_reskin_tid = 0, s0_ovr_cost = 0;
        Int s0_def_mods = 0, s0_main_mods = 0, s0_ovr_mods = 0, s0_ovr_prereqs = 0;
        UnsignedInt s0_def_occ = 0;
        Int s0_main_skill0 = 0, s0_main_any_wpn = 0, s0_reskin_geom = 0;
        size_t s0_main_bvar = 0;
        std::string s0_def_mod_list, s0_main_mod_list, s0_ovr_mod_list, s0_main_shadow_tex;
        Real s0_reskin_fence_w = 0.0f, s0_reskin_rad = 0.0f;
        Real s0_ovr_time = 0.0f, s0_ovr_vision = 0.0f;
        Real s0_hc_cost = 0.0f, s0_hc_time = 0.0f, s0_eratio = 0.0f;
        if (seed == 0) {
            s0_def_tid = def_tt->getTemplateID();
            s0_def_mods = def_tt->getBehaviorModuleInfo().getCount();
            s0_def_occ = def_tt->getOcclusionDelay();
            s0_def_mod_list = join_mods(def_tt);

            s0_main_tid = main_tt->getTemplateID();
            s0_main_mods = main_tt->getBehaviorModuleInfo().getCount();
            s0_main_mod_list = join_mods(main_tt);
            s0_main_shadow_tex = main_tt->getShadowTextureName().str();
            s0_main_skill0 = main_tt->getSkillPointValue(0);
            s0_main_bvar = main_tt->getBuildVariations().size();
            s0_main_any_wpn = main_tt->canPossiblyHaveAnyWeapon() ? 1 : 0;

            s0_reskin_tid = reskin_tt->getTemplateID();
            s0_reskin_fence_w = reskin_tt->getFenceWidth();
            s0_reskin_geom = static_cast<Int>(reskin_tt->getTemplateGeometryInfo().getGeomType());
            s0_reskin_rad = reskin_tt->getTemplateGeometryInfo().getMajorRadius();

            s0_ovr_cost = main_final->*access_member(TTBuildCostTag());
            s0_ovr_time = main_final->*access_member(TTBuildTimeTag());
            s0_ovr_vision = main_final->friend_calcVisionRange();
            s0_ovr_mods = main_final->getBehaviorModuleInfo().getCount();
            s0_ovr_mod_list = join_mods(main_final);
            s0_ovr_prereqs = main_final->getPrereqCount();

            s0_hc_cost = player.getHandicap()->getHandicap(Handicap::BUILDCOST, main_final);
            s0_hc_time = player.getHandicap()->getHandicap(Handicap::BUILDTIME, main_final);
            s0_eratio = player.getEnergy()->getEnergySupplyRatio();
        }

        // Reset factory & check post-reset invariants
        TheThingFactory->reset();
        const ThingTemplate* after_reset_main = TheThingFactory->findTemplate("UnitMain");
        const ThingTemplate* after_reset_map_only = TheThingFactory->findTemplate("MapOnlyObj", FALSE);
        uint32_t post_reset_crc = digest_thing_template(after_reset_main);
        UnsignedShort post_reset_next_id = TheThingFactory->*access_member(TFNextIdTag());

        out << "    {\"seed\":" << seed
            << ",\"occ\":" << occ_delay
            << ",\"grav\":" << oracle::fbits(gravity)
            << ",\"epen\":" << oracle::fbits(low_e_pen)
            << ",\"espd\":[" << oracle::fbits(min_e_spd) << "," << oracle::fbits(max_e_spd) << "," << oracle::fbits(mult_fac) << "]"
            << ",\"swr\":" << sw_restr
            << ",\"ini\":\"" << oracle::json_escape(init_ini) << "\""
            << ",\"ovr_ini\":\"" << oracle::json_escape(ovr_ini) << "\""
            << ",\"p_pcts\":[" << oracle::fbits(p_cost_pct) << "," << oracle::fbits(p_time_pct) << "," << oracle::fbits(p_ko_pct) << "]"
            << ",\"p_hc\":[" << oracle::fbits(p_hc[0]) << "," << oracle::fbits(p_hc[1]) << "," << oracle::fbits(p_hc[2]) << "," << oracle::fbits(p_hc[3]) << "]"
            << ",\"p_en\":[" << p_e_prod << "," << p_e_cons << "]";
        out << ",\"am_q\":";
        append_u32_array(out, am_q, 4);
        out << ",\"wm_q\":";
        append_u32_array(out, wm_q, 4);
        out << ",\"th_in\":[" << oracle::fbits(th_x) << "," << oracle::fbits(th_y) << "," << oracle::fbits(th_z)
            << "," << oracle::fbits(th_z1) << "," << oracle::fbits(th_ang)
            << "," << oracle::fbits(th_pt[0]) << "," << oracle::fbits(th_pt[1]) << "," << oracle::fbits(th_pt[2]) << "]"
            << ",\"th_mtx\":";
        append_u32_array(out, th_mtx_in, 12);
        out << ",\"ot_names\":";
        append_str_array(out, ot_names);
        out << ",\"ot_rm\":\"" << oracle::json_escape(ot_rm) << "\""
            << ",\"tt_crcs\":[" << pre_ovr_crc << "," << reskin_crc << "," << post_ovr_crc << "," << post_reset_crc << "]"
            << ",\"eq_mask\":" << eq_mask
            << ",\"ovr_b\":" << ovr_buildable
            << ",\"map_gone\":" << (after_reset_map_only == nullptr ? 1 : 0)
            << ",\"rst_nid\":" << post_reset_next_id
            << ",\"sm_h\":[" << am_hash << "," << wm_hash << "]"
            << ",\"build\":[" << calc_cost << "," << calc_time << "]"
            << ",\"th_h\":[" << th_hash << "," << slope_hash << "]"
            << ",\"ot_out\":[" << ot_in_mask << "," << ot_prep_cnt << "," << static_cast<int32_t>(otsave.bytes.size()) << "," << otsave.bytes_crc() << "," << otxcrc.getCRC() << "]";

        if (seed == 0) {
            out << ",\"s0_default_tmpl\":{\"tid\":" << s0_def_tid
                << ",\"mods\":" << s0_def_mods
                << ",\"mod_list\":\"" << oracle::json_escape(s0_def_mod_list) << "\""
                << ",\"occ\":" << s0_def_occ << "}"
                << ",\"s0_unit_tmpl\":{\"tid\":" << s0_main_tid
                << ",\"mods\":" << s0_main_mods
                << ",\"mod_list\":\"" << oracle::json_escape(s0_main_mod_list) << "\""
                << ",\"shadow_tex\":\"" << oracle::json_escape(s0_main_shadow_tex) << "\""
                << ",\"skill0\":" << s0_main_skill0
                << ",\"bvar\":" << s0_main_bvar
                << ",\"any_wpn\":" << s0_main_any_wpn << "}"
                << ",\"s0_reskin_tmpl\":{\"tid\":" << s0_reskin_tid
                << ",\"fence_w\":" << oracle::fbits(s0_reskin_fence_w)
                << ",\"geom\":" << s0_reskin_geom
                << ",\"rad\":" << oracle::fbits(s0_reskin_rad) << "}"
                << ",\"s0_override_tmpl\":{\"ovr_cost\":" << s0_ovr_cost
                << ",\"ovr_time\":" << oracle::fbits(s0_ovr_time)
                << ",\"ovr_vision\":" << oracle::fbits(s0_ovr_vision)
                << ",\"ovr_mods\":" << s0_ovr_mods
                << ",\"mod_list\":\"" << oracle::json_escape(s0_ovr_mod_list) << "\""
                << ",\"ovr_prereqs\":" << s0_ovr_prereqs << "}"
                << ",\"s0_post_reset\":{\"restored_cost\":" << after_reset_main->*access_member(TTBuildCostTag())
                << ",\"restored_mods\":" << after_reset_main->getBehaviorModuleInfo().getCount()
                << ",\"mod_list\":\"" << oracle::json_escape(join_mods(after_reset_main)) << "\""
                << ",\"next_id\":" << post_reset_next_id << "}"
                << ",\"s0_sparse\":{\"am_hash\":" << am_hash << ",\"wm_hash\":" << wm_hash << "}"
                << ",\"s0_build_math\":{\"cost\":" << calc_cost << ",\"time\":" << calc_time
                << ",\"hc_cost\":" << oracle::fbits(s0_hc_cost)
                << ",\"hc_time\":" << oracle::fbits(s0_hc_time)
                << ",\"eratio\":" << oracle::fbits(s0_eratio) << "}"
                << ",\"s0_thing\":{\"ht0\":" << oracle::fbits(h_terr0)
                << ",\"hw0\":" << oracle::fbits(h_water0)
                << ",\"ht1\":" << oracle::fbits(h_terr1)
                << ",\"dir2\":[" << oracle::fbits(dir2->x) << "," << oracle::fbits(dir2->y) << "]"
                << ",\"dir3\":[" << oracle::fbits(dir3.x) << "," << oracle::fbits(dir3.y) << "," << oracle::fbits(dir3.z) << "]"
                << ",\"world_pt\":[" << oracle::fbits(world_pt.x) << "," << oracle::fbits(world_pt.y) << "," << oracle::fbits(world_pt.z) << "]"
                << ",\"xform_pt\":[" << oracle::fbits(xform_pt.x) << "," << oracle::fbits(xform_pt.y) << "," << oracle::fbits(xform_pt.z) << "]"
                << ",\"pos_mx\":[" << oracle::fbits(cached_pos_after_mx->x) << "," << oracle::fbits(cached_pos_after_mx->y) << "," << oracle::fbits(cached_pos_after_mx->z) << "]"
                << ",\"ang_mx\":" << oracle::fbits(cached_ang_after_mx)
                << ",\"reacts\":" << th.react_count << "}"
                << ",\"s0_thing_slope\":{\"px\":" << oracle::fbits(slope_th.getPosition()->x)
                << ",\"pz\":" << oracle::fbits(slope_th.getPosition()->z)
                << ",\"d3\":[" << oracle::fbits(slope_dir3.x) << "," << oracle::fbits(slope_dir3.y) << "," << oracle::fbits(slope_dir3.z) << "]"
                << ",\"reacts\":" << slope_th.react_count << "}"
                << ",\"s0_obj_types\":{\"ot_xhex\":\"" << otsave.hex() << "\""
                << ",\"prep_cnt\":" << ot_prep_cnt
                << ",\"in_mask\":" << ot_in_mask << "}";
        }

        out << "}" << (seed + 1 < 1000 ? "," : "") << "\n";

        delete TheThingFactory;
        TheThingFactory = nullptr;
        delete TheModuleFactory;
        TheModuleFactory = nullptr;
    }

    out << "  ]\n}\n";
    oracle::write_file_atomic(out_path, out.str());

    delete TheWeaponStore;
    TheWeaponStore = nullptr;
    delete TheRankInfoStore;
    TheRankInfoStore = nullptr;
    delete TheArmorStore;
    TheArmorStore = nullptr;
    delete TheDamageFXStore;
    TheDamageFXStore = nullptr;
    delete TheFXListStore;
    TheFXListStore = nullptr;
    delete TheScienceStore;
    TheScienceStore = nullptr;
    TheTerrainLogic = nullptr;
    TheGameText = nullptr;
    delete TheGameLogic;
    TheGameLogic = nullptr;
    delete TheNameKeyGenerator;
    TheNameKeyGenerator = nullptr;
    delete TheWritableGlobalData;
    TheWritableGlobalData = nullptr;
    shutdownMemoryManager();
    return 0;
}
