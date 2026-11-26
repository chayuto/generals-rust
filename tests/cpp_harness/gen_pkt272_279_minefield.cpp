// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for GenerateMinefieldBehavior (PKT-272) and MinefieldBehavior
// (PKT-279), run on the unmodified reference modules in a small real world, with
// the GeometryInfo footprint operations, PartitionManager::iteratePotentialCollisions
// and the cliff flags of WorldHeightMap.
//
// The cliff query needs the device headers:
//   CPATH=vendor/GeneralsX/GeneralsMD/Code/GameEngineDevice/Include:vendor/GeneralsX/Core/GameEngineDevice/Include \
//     python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt272_279_minefield.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT272_279_Minefield.json
//
// The terrain is the harness's: flat with one step, a water rectangle, a bridge, and
// BaseHeightMapRenderObjClass::isCliffCell over a WorldHeightMap built in memory.
// getHighestLayerForDestination is the reference's own (it is not virtual).
//
// Not covered: effects of the detonation weapon on any object (it is aimed where
// nothing takes damage; only its firing is recorded), what the creation list makes,
// wrapper bodies on a mine, a mine clearer whose AI is updating, a tilted object,
// frames near 2^24, NaN inputs, and footprints with no room between mines (the
// reference never returns).

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/Team.h"
#include "Common/Upgrade.h"
#include "GameClient/FXList.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/AutoHealBehavior.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/GenerateMinefieldBehavior.h"
#include "GameLogic/Module/MinefieldBehavior.h"
#include "GameLogic/Module/UpgradeModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/ObjectIter.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Weapon.h"
#include "W3DDevice/GameClient/BaseHeightMap.h"
#include "W3DDevice/GameClient/WorldHeightMap.h"

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

// ---- private members ---------------------------------------------------------

#define MINES_TAG(Tag, Type, Class, Member)                  \
    struct Tag { using type = Type Class::*; };              \
    template struct MemberAccess<Tag, &Class::Member>;       \
    inline Type Class::* access_member(Tag);

MINES_TAG(HmWidth, Int, WorldHeightMap, m_width)
MINES_TAG(HmHeight, Int, WorldHeightMap, m_height)
MINES_TAG(HmBorder, Int, WorldHeightMap, m_borderSize)
MINES_TAG(HmDataSize, Int, WorldHeightMap, m_dataSize)
MINES_TAG(HmData, UnsignedByte*, WorldHeightMap, m_data)
MINES_TAG(HmStride, Int, WorldHeightMap, m_flipStateWidth)
MINES_TAG(HmCliff, UnsignedByte*, WorldHeightMap, m_cellCliffState)
MINES_TAG(BhmMap, WorldHeightMap*, BaseHeightMapRenderObjClass, m_map)
MINES_TAG(NextIdTag, ObjectID, GameLogic, m_nextObjID)
MINES_TAG(UpgradesDone, UpgradeMaskType, Object, m_objectUpgradesCompleted)
MINES_TAG(HistoricTag, HistoricWeaponDamageList, WeaponTemplate, m_historicDamage)

struct HmInitTag { using type = void (WorldHeightMap::*)(); };
template struct MemberAccess<HmInitTag, &WorldHeightMap::initCliffFlagsFromHeights>;
inline void (WorldHeightMap::* access_member(HmInitTag))();

// ---- random inputs ------------------------------------------------------------

static uint32_t g_state = 0x27200279u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
// A multiple of 1/64 in [lo, hi).
static float pickf(float lo, float hi) {
    const uint32_t steps = static_cast<uint32_t>((hi - lo) * 64.0f);
    return lo + static_cast<float>(pick(steps ? steps : 1)) / 64.0f;
}
static uint32_t bits(float f) {
    uint32_t u;
    std::memcpy(&u, &f, 4);
    return u;
}

// ---- terrain ------------------------------------------------------------------

struct FakeHeightMap {
    WorldHeightMap* map = nullptr;
    std::vector<UnsignedByte> data, cliff;

    void build(Int width, Int height, Int border) {
        std::free(map);
        map = static_cast<WorldHeightMap*>(std::calloc(1, sizeof(WorldHeightMap)));
        const Int stride = (width + 7) / 8;
        data.assign(static_cast<size_t>(width) * height, 0);
        cliff.assign(static_cast<size_t>(stride) * height, 0);
        map->*access_member(HmWidth()) = width;
        map->*access_member(HmHeight()) = height;
        map->*access_member(HmBorder()) = border;
        map->*access_member(HmDataSize()) = width * height;
        map->*access_member(HmData()) = data.data();
        map->*access_member(HmStride()) = stride;
        map->*access_member(HmCliff()) = cliff.data();
    }
    void set_bit(Int x, Int y) {
        cliff[static_cast<size_t>(y) * ((map->*access_member(HmStride()))) + (x >> 3)] |= 1 << (x & 7);
    }
};

static FakeHeightMap g_height_map;
static BaseHeightMapRenderObjClass* g_render = nullptr;

static float g_ground = 0.0f, g_step = 0.0f, g_water_x = 5000.0f, g_water_y = 5000.0f;
static const float kBridgeLo = 703.25f, kBridgeHi = 797.75f, kBridgeZ = 20.0f;

class MineTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    Real getGroundHeight(Real x, Real, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return x > 400.0f ? g_ground + g_step : g_ground;
    }
    Real getLayerHeight(Real x, Real y, PathfindLayerEnum layer, Coord3D* normal = nullptr, Bool = true) const override {
        if (layer != LAYER_GROUND) return kBridgeZ;
        return getGroundHeight(x, y, normal);
    }
    Bool isCliffCell(Real x, Real y) const override { return g_render->isCliffCell(x, y); }
    Bool isUnderwater(Real x, Real y, Real* waterZ = nullptr, Real* terrainZ = nullptr) override {
        if (waterZ) *waterZ = 0.0f;
        if (terrainZ) *terrainZ = getGroundHeight(x, y);
        return x > g_water_x && y > g_water_y;
    }
};

// ---- spies --------------------------------------------------------------------

static std::string g_ocls;
class OclSpy : public ObjectCreationNugget {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(OclSpy, "MinesOclSpy", 4, 4)
public:
    Object* create(const Object* primary, const Coord3D*, const Coord3D*, Real, UnsignedInt = 0) const override { return nullptr; }
    Object* create(const Object* primary, const Object* secondary, UnsignedInt = 0) const override {
        g_ocls += "." + std::to_string(primary->getID()) + "," + std::to_string(secondary->getID());
        return nullptr;
    }
};
EMPTY_DTOR(OclSpy)

static std::string g_fx;
class FxSpy : public FXNugget {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(FxSpy, "MinesFxSpy", 4, 4)
public:
    void doFXPos(const Coord3D*, const Matrix3D* = nullptr, const Real = 0.0f, const Coord3D* = nullptr, const Real = 0.0f) const override {}
    void doFXObj(const Object* primary, const Object* = nullptr) const override {
        const Coord3D* p = primary->getPosition();
        g_fx += "." + std::to_string(bits(p->x)) + "," + std::to_string(bits(p->y)) + "," + std::to_string(bits(p->z));
    }
};
EMPTY_DTOR(FxSpy)

// A fire from an earlier frame is trimmed from the list by the next one, so step
// loops collect after every frame.
static std::string g_fires;
static void collect_fires() {
    const WeaponTemplate* t = TheWeaponStore->findWeaponTemplate("P279_Det");
    HistoricWeaponDamageList& list = const_cast<WeaponTemplate*>(t)->*access_member(HistoricTag());
    for (auto it = list.begin(); it != list.end(); ++it)
        g_fires += "." + std::to_string(bits(it->location.x)) + "," + std::to_string(bits(it->location.y)) + "," + std::to_string(bits(it->location.z));
    list.clear();
}

// ---- fixtures -----------------------------------------------------------------

static std::string geometry(const char* type, float height, float major, float minor) {
    char buf[256];
    std::snprintf(buf, sizeof buf,
                  "  Geometry = %s\n  GeometryHeight = %g\n  GeometryMajorRadius = %g\n  GeometryMinorRadius = %g\n  GeometryIsSmall = No\n",
                  type, height, major, minor);
    return buf;
}

static std::string body(const char* max_health, const char* initial = nullptr) {
    return std::string("  Body = ActiveBody ModuleTag_Body\n    MaxHealth = ") + max_health +
           "\n    InitialHealth = " + (initial ? initial : max_health) + "\n  End\n";
}

static std::string object(const std::string& name, const std::string& kindof, const std::string& rest) {
    return "Object " + name + "\n  KindOf = " + kindof + "\n" + rest + "End\n";
}

static std::string minefield(const std::string& fields) {
    return "  Behavior = MinefieldBehavior ModuleTag_Mine\n" + fields + "  End\n";
}

static std::string auto_heal(int amount, int delay) {
    return "  Behavior = AutoHealBehavior ModuleTag_Heal\n    StartsActive = Yes\n    HealingAmount = " +
           std::to_string(amount) + "\n    HealingDelay = " + std::to_string(delay) + "\n  End\n";
}

static const int kMineVariants = 9;
static const int kUnits = 8;
static const int kBlocks = 3;

static std::string object_ini() {
    std::string s;
    // Mines of the behaviour vectors.
    s += object("P279_M0", "MINE IMMOBILE", geometry("CYLINDER", 3, 6, 6) + body("100") +
                minefield("    DetonationWeapon = P279_Det\n    CreationList = P279_OCL\n"));
    s += object("P279_M1", "MINE IMMOBILE", geometry("CYLINDER", 3, 6, 6) + body("100") +
                minefield("    DetonationWeapon = P279_Det\n    NumVirtualMines = 4\n    Regenerates = Yes\n"
                          "    DegenPercentPerSecondAfterCreatorDies = 10%\n    CreatorDeathCheckRate = 200\n"
                          "    DetonatedBy = ENEMIES NEUTRAL\n") +
                auto_heal(7, 200));
    s += object("P279_M2", "MINE IMMOBILE", geometry("CYLINDER", 3, 5, 5) + body("37.3") +
                minefield("    NumVirtualMines = 8\n    WorkersDetonate = Yes\n    DetonatedBy = ALLIES NEUTRAL ENEMIES\n"
                          "    RepeatDetonateMoveThresh = 5.0\n    Regenerates = Yes\n    StopsRegenAfterCreatorDies = No\n"
                          "    CreationList = P279_OCL\n"));
    s += object("P279_M3", "MINE IMMOBILE", geometry("CYLINDER", 3, 6, 6) + body("75") +
                minefield("    ScootFromStartingPointTime = 1000\n    DetonatedBy = ENEMIES\n    NumVirtualMines = 3\n"
                          "    Regenerates = Yes\n    DegenPercentPerSecondAfterCreatorDies = 50%\n    CreatorDeathCheckRate = 100\n"));
    s += object("P279_M4", "MINE IMMOBILE", geometry("CYLINDER", 3, 6, 6) + body("20") +
                minefield("    NumVirtualMines = 0\n    Regenerates = Yes\n    CreatorDeathCheckRate = 0\n"));
    s += object("P279_M5", "MINE IMMOBILE", geometry("BOX", 3, 8, 4) + body("250") +
                minefield("    NumVirtualMines = 5\n    DetonatedBy = -ENEMIES +ALLIES\n    DetonationWeapon = P279_Det\n"));
    s += object("P279_M6", "MINE IMMOBILE", geometry("SPHERE", 3, 7, 7) + body("10") +
                minefield("    NumVirtualMines = 2\n    Regenerates = Yes\n    DegenPercentPerSecondAfterCreatorDies = 100%\n"
                          "    CreatorDeathCheckRate = 300\n    DetonationWeapon = P279_Det\n") +
                auto_heal(100, 100));
    s += object("P279_M7", "MINE IMMOBILE", geometry("CYLINDER", 3, 6, 6) + body("1000000000.0") +
                minefield("    NumVirtualMines = 3\n    Regenerates = Yes\n    DegenPercentPerSecondAfterCreatorDies = 40%\n"));
    s += object("P279_M8", "MINE IMMOBILE", geometry("CYLINDER", 3, 6, 6) + body("100", "40") +
                minefield("    NumVirtualMines = 10\n    Regenerates = Yes\n    RepeatDetonateMoveThresh = 0.0\n"
                          "    ScootFromStartingPointTime = 33\n") +
                auto_heal(3, 100));
    // Mines of the placement vectors.
    s += object("P279_G0", "MINE IMMOBILE", geometry("CYLINDER", 3, 6, 6) + body("100") + minefield(""));
    s += object("P279_G1", "MINE IMMOBILE", geometry("CYLINDER", 3, 6, 6) + body("100") +
                minefield("    ScootFromStartingPointTime = 500\n"));
    s += object("P279_G2", "MINE IMMOBILE STRUCTURE", geometry("CYLINDER", 3, 7, 7) + body("100") + minefield(""));
    s += object("P279_G3", "MINE IMMOBILE", geometry("BOX", 3, 11, 9) + body("100") + minefield(""));
    s += object("P279_G4", "MINE IMMOBILE", geometry("CYLINDER", 3, 4.5f, 4.5f) + body("100") +
                minefield("    Regenerates = Yes\n    NumVirtualMines = 2\n") + auto_heal(5, 1000));
    // No LandMineInterface, and mines about as narrow as the divisor limit of 1.
    s += object("P279_G5", "MINE IMMOBILE", geometry("CYLINDER", 3, 5, 5) + body("100"));
    s += object("P279_G6", "MINE IMMOBILE", geometry("CYLINDER", 1, 0.53125f, 0.53125f) + body("100") + minefield(""));
    s += object("P279_G7", "MINE IMMOBILE", geometry("CYLINDER", 1, 0.40625f, 0.40625f) + body("100") + minefield(""));
    // Producers.
    auto gen = [](const std::string& fields) {
        return "  Behavior = GenerateMinefieldBehavior ModuleTag_Gen\n" + fields + "  End\n";
    };
    auto producer = [&](int i, const std::string& kindof, const std::string& geom, const std::string& fields) {
        const bool has_trigger = fields.find("TriggeredBy = P272") != std::string::npos;
        s += object("P272_P" + std::to_string(i), kindof,
                    geom + body("500") + gen(fields + (has_trigger ? "" : "    TriggeredBy = P272_Trigger\n")));
    };
    producer(0, "STRUCTURE", geometry("BOX", 10, 30, 20), "    MineName = P279_G0\n    GenerationFX = FX_P272\n    DistanceAroundObject = 12\n");
    producer(1, "STRUCTURE", geometry("CYLINDER", 10, 25, 25), "    MineName = P279_G0\n    RandomJitter = 50%\n    DistanceAroundObject = 15\n    GenerateOnlyOnDeath = Yes\n");
    producer(2, "STRUCTURE", geometry("SPHERE", 10, 20, 20), "    MineName = P279_G1\n    BorderOnly = No\n    MinesPerSquareFoot = 0.004\n    DistanceAroundObject = 20\n");
    producer(3, "STRUCTURE", geometry("BOX", 10, 20, 14), "    MineName = P279_G0\n    BorderOnly = No\n    AlwaysCircular = Yes\n    MinesPerSquareFoot = 0.003\n    DistanceAroundObject = 15\n    GenerateOnlyOnDeath = Yes\n");
    producer(4, "STRUCTURE", geometry("BOX", 10, 22, 12), "    MineName = P279_G0\n    SmartBorder = Yes\n    DistanceAroundObject = 55\n");
    producer(5, "STRUCTURE", geometry("BOX", 10, 22, 12), "    MineName = P279_G1\n    SmartBorder = Yes\n    SmartBorderSkipInterior = No\n    AlwaysCircular = Yes\n    DistanceAroundObject = 30\n    GenerationFX = FX_P272\n");
    producer(6, "STRUCTURE", geometry("CYLINDER", 10, 12, 12), "    MineName = P279_G3\n    BorderOnly = No\n    MinesPerSquareFoot = 0.02\n    DistanceAroundObject = 18\n");
    producer(7, "STRUCTURE", geometry("BOX", 10, 18, 18), "    MineName = P279_G0\n    UpgradedMineName = P279_G4\n    UpgradedTriggeredBy = Upgrade_ChinaEMPMines\n"
             "    GenerateOnlyOnDeath = Yes\n    Upgradable = Yes\n    TriggeredBy = P272_Trigger\n    DistanceAroundObject = 10\n    GenerationFX = FX_P272\n");
    producer(8, "STRUCTURE", geometry("CYLINDER", 10, 20, 20), "    MineName = P279_G4\n    UpgradedMineName = P279_G1\n    Upgradable = Yes\n    TriggeredBy = P272_Trigger P272_Other\n"
             "    ConflictsWith = P272_Conflict\n    RequiresAllTriggers = Yes\n    DistanceAroundObject = 10\n");
    producer(9, "STRUCTURE", geometry("BOX", 10, 18, 18), "    MineName = P279_Missing\n    UpgradedMineName = P279_G0\n    Upgradable = Yes\n    GenerateOnlyOnDeath = Yes\n    TriggeredBy = P272_Trigger\n");
    producer(10, "PRELOAD", geometry("CYLINDER", 5, 3, 3), "    MineName = P279_G1\n    GenerateOnlyOnDeath = Yes\n    BorderOnly = No\n    AlwaysCircular = Yes\n");
    producer(11, "STRUCTURE", geometry("BOX", 10, 20, 16), "    MineName = P279_G2\n    SmartBorder = Yes\n    AlwaysCircular = Yes\n    DistanceAroundObject = 20\n");
    producer(12, "STRUCTURE", geometry("BOX", 10, 24, 16), "    MineName = P279_G3\n    SkipIfThisMuchUnderStructure = 0%\n    RandomJitter = 100%\n    GenerateOnlyOnDeath = No\n    DistanceAroundObject = -4\n");
    producer(13, "STRUCTURE", geometry("CYLINDER", 10, 18, 18), "    MineName = P279_G2\n    SkipIfThisMuchUnderStructure = 100%\n    RandomJitter = 30%\n    DistanceAroundObject = 6\n    TriggeredBy = P272_Trigger\n");
    producer(14, "STRUCTURE", geometry("BOX", 10, 8, 8), "    MineName = P279_G0\n    Upgradable = Yes\n    UpgradedMineName = P279_Missing\n    DistanceAroundObject = 4\n    TriggeredBy = P272_Trigger\n");
    // More mines than the saved byte count holds.
    producer(15, "STRUCTURE", geometry("CYLINDER", 10, 10, 10), "    MineName = P279_G0\n    Upgradable = Yes\n    UpgradedMineName = P279_G0\n    BorderOnly = No\n    MinesPerSquareFoot = 0.04\n    DistanceAroundObject = 40\n    SkipIfThisMuchUnderStructure = 0%\n");

    // A footprint that rounds to one mine.
    producer(16, "STRUCTURE", geometry("CYLINDER", 10, 4, 4), "    MineName = P279_G0\n    BorderOnly = No\n    MinesPerSquareFoot = 0.00001\n    DistanceAroundObject = 20\n");
    producer(17, "STRUCTURE", geometry("BOX", 10, 16, 12), "    MineName = P279_G5\n    DistanceAroundObject = 10\n    RandomJitter = 40%\n    Upgradable = Yes\n    UpgradedMineName = P279_G5\n");
    producer(18, "STRUCTURE", geometry("CYLINDER", 4, 3, 3), "    MineName = P279_G6\n    DistanceAroundObject = 2\n");
    producer(19, "STRUCTURE", geometry("BOX", 4, 3, 2), "    MineName = P279_G7\n    DistanceAroundObject = 1\n");

    // Units that run into mines.
    const std::string small = geometry("CYLINDER", 8, 4, 4) + body("100");
    const std::string ai = "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n";
    auto armed = [](const char* w) {
        return std::string("  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY ") + w + "\n  End\n";
    };
    s += object("P279_U0", "INFANTRY", small);
    s += object("P279_U1", "INFANTRY DOZER", small);
    s += object("P279_U2", "VEHICLE", geometry("BOX", 8, 12, 6) + body("100"));
    s += object("P279_U3", "VEHICLE DOZER", small);
    s += object("P279_U4", "INFANTRY", small + ai + armed("P279_AntiMine"));
    s += object("P279_U5", "INFANTRY", small + ai + armed("P279_Gun"));
    s += object("P279_U6", "INFANTRY", small + ai);
    s += object("P279_U7", "INFANTRY", small + armed("P279_AntiMine"));
    // Structures in the way.
    s += object("P279_S0", "STRUCTURE", geometry("BOX", 12, 14, 8) + body("500"));
    s += object("P279_S1", "STRUCTURE", geometry("CYLINDER", 6, 9, 9) + body("500"));
    s += object("P279_S2", "STRUCTURE", geometry("SPHERE", 6, 7, 7) + body("500"));
    return s;
}

static std::string weapon_ini() {
    auto weapon = [](const char* name, const char* extra) {
        return std::string("Weapon ") + name +
               "\n  PrimaryDamage = 1.0\n  PrimaryDamageRadius = 0.0\n  AttackRange = 100.0\n  DelayBetweenShots = 1000\n  ClipSize = 0\n" +
               extra + "End\n";
    };
    return weapon("P279_Det", "  HistoricBonusCount = 1000000\n  HistoricBonusTime = 33\n  HistoricBonusRadius = 50.0\n") +
           weapon("P279_AntiMine", "  AntiMine = Yes\n") + weapon("P279_Gun", "");
}

static const char* const kUpgrades[4] = {"Upgrade_ChinaEMPMines", "P272_Trigger", "P272_Other", "P272_Conflict"};

// ---- world --------------------------------------------------------------------

static oracle_wp51::World* g_w = nullptr;
static Team* g_team_a2 = nullptr;
static int g_bridge_layer = 0;

// 0 neutral, 1 A, 2 B, 3 a second team of player A.
static Team* team_of(int i) {
    return i == 0 ? g_w->team_neutral : i == 1 ? g_w->team_a : i == 2 ? g_w->team_b : g_team_a2;
}
static int team_index(const Team* t) {
    for (int i = 0; i < 4; ++i)
        if (team_of(i) == t) return i;
    return 9;
}

static Object* spawn(const std::string& tmpl, int team, float x, float y, float z, float angle) {
    const ThingTemplate* tt = TheThingFactory->findTemplate(tmpl.c_str());
    if (!tt) {
        std::fprintf(stderr, "no template %s\n", tmpl.c_str());
        std::exit(1);
    }
    Object* o = TheThingFactory->newObject(tt, team_of(team));
    Coord3D p;
    p.x = x; p.y = y; p.z = z;
    o->setPosition(&p);
    o->setOrientation(angle);
    return o;
}

template <typename T>
static T* find_behavior(Object* obj) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m)
        if (T* typed = dynamic_cast<T*>(*m)) return typed;
    return nullptr;
}

static std::vector<Object*> all_objects() {
    std::vector<Object*> v;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) v.push_back(o);
    std::sort(v.begin(), v.end(), [](Object* a, Object* b) { return a->getID() < b->getID(); });
    return v;
}

static void clear_world() {
    for (Object* o : all_objects()) TheGameLogic->destroyObject(o);
    g_w->flush_destroy_list();
    ThePartitionManager->update();
    TheGameLogic->*access_member(NextIdTag()) = static_cast<ObjectID>(1);
    collect_fires();
    g_fires.clear();
    g_ocls.clear();
    g_fx.clear();
}

static void step_module(UpdateModule* u, UnsignedInt frame) {
    if (u->friend_getNextCallFrame() > frame) return;
    UpdateSleepTime sleep_len = u->update();
    if (sleep_len < 1) sleep_len = UPDATE_SLEEP_NONE;
    UnsignedInt next = frame + static_cast<UnsignedInt>(sleep_len);
    if (next > UPDATE_SLEEP_FOREVER) next = UPDATE_SLEEP_FOREVER;
    u->friend_setNextCallFrame(next);
}

static void save_load(Snapshot* s, UpdateModule* u) {
    const std::string hex = oracle_wp51::World::snapshot_save_hex(s);
    std::vector<uint8_t> bytes;
    for (size_t i = 0; i + 1 < hex.size(); i += 2)
        bytes.push_back(static_cast<uint8_t>(std::strtoul(hex.substr(i, 2).c_str(), nullptr, 16)));
    // A loaded module forgets its heap slot (see the harness notes).
    const Int index = u->friend_getIndexInLogic();
    oracle_wp51::World::snapshot_load_bytes(s, bytes);
    u->friend_setIndexInLogic(index);
}

static std::string vec_bits(const Coord3D& p) {
    return std::to_string(bits(p.x)) + "," + std::to_string(bits(p.y)) + "," + std::to_string(bits(p.z));
}

// Terrain of one module vector, as "ground step waterX waterY cliffLoX cliffLoY cliffHiX cliffHiY".
static std::string random_terrain(float cx, float cy, bool hazards) {
    g_ground = pick(2) ? 0.0f : pickf(-3.0f, 6.0f);
    g_step = pick(3) ? 0.0f : pickf(-4.0f, 4.0f);
    g_water_x = g_water_y = 5000.0f;
    int lo_x = 0, lo_y = 0, hi_x = 0, hi_y = 0;
    if (hazards && pick(3) == 0) {
        g_water_x = cx + pickf(-30.0f, 30.0f);
        g_water_y = cy + pickf(-30.0f, 30.0f);
    }
    if (hazards && pick(3) == 0) {
        lo_x = static_cast<int>(cx / 10.0f) + 2 + static_cast<int>(pick(7)) - 4;
        lo_y = static_cast<int>(cy / 10.0f) + 2 + static_cast<int>(pick(7)) - 4;
        hi_x = lo_x + 1 + static_cast<int>(pick(5));
        hi_y = lo_y + 1 + static_cast<int>(pick(5));
    }
    g_height_map.build(104, 104, 2);
    for (int y = lo_y; y < hi_y; ++y)
        for (int x = lo_x; x < hi_x; ++x)
            if (x >= 0 && y >= 0 && x < 104 && y < 104) g_height_map.set_bit(x, y);
    g_render->*access_member(BhmMap()) = g_height_map.map;
    std::ostringstream out;
    out << bits(g_ground) << ' ' << bits(g_step) << ' ' << bits(g_water_x) << ' ' << bits(g_water_y) << ' '
        << lo_x << ' ' << lo_y << ' ' << hi_x << ' ' << hi_y;
    return out.str();
}

// ---- placement vectors ----------------------------------------------------------

static bool is_mine(const Object* o) {
    const char* n = o->getTemplate()->getName().str();
    return std::strncmp(n, "P279_G", 6) == 0 || std::strncmp(n, "P279_M", 6) == 0;
}

// "id,template,team,producer,x,y,z,angle,layer,savecrc,effdead,next" of every mine.
static std::string mine_list() {
    std::string out;
    for (Object* o : all_objects()) {
        if (!is_mine(o)) continue;
        MinefieldBehavior* m = find_behavior<MinefieldBehavior>(o);
        uint32_t crc = 0;
        if (m) oracle_wp51::World::snapshot_save_hex(static_cast<UpdateModule*>(m), &crc);
        std::ostringstream t;
        t << o->getID() << ',' << o->getTemplate()->getName().str() << ',' << team_index(o->getTeam()) << ','
          << o->getProducerID() << ',' << vec_bits(*o->getPosition()) << ',' << bits(o->getOrientation()) << ','
          << static_cast<int>(o->getLayer()) << ',' << crc << ',' << (o->isEffectivelyDead() ? 1 : 0) << ','
          << (m ? m->friend_getNextCallFrame() : 0) << ';';
        out += t.str();
    }
    return out;
}

static void placement_vector(FILE* out, int index) {
    clear_world();
    // The last four vectors fill past 255 mines.
    const bool crowd = index % 150 == 149;
    int variant = crowd ? 15 : static_cast<int>(pick(19));
    if (!crowd && variant >= 15) ++variant;
    const bool on_bridge = pick(8) == 0;
    const float cx = on_bridge ? pickf(710.0f, 790.0f) : pickf(150.0f, 600.0f);
    const float cy = on_bridge ? pickf(710.0f, 790.0f) : pickf(150.0f, 600.0f);
    const std::string terrain = random_terrain(cx, cy, !crowd);
    const UnsignedInt seed = 1 + pick(1000000);
    UnsignedInt frame = 1 + pick(5000);
    const UnsignedInt frame0 = frame;
    g_w->set_frame(frame);
    InitRandom(seed);

    const float cz = pick(2) ? 0.0f : pickf(0.0f, 40.0f);
    const float angle = pick(3) == 0 ? 0.0f : pickf(-3.14f, 3.14f);
    const int team = 1 + static_cast<int>(pick(3));
    Object* producer = spawn("P272_P" + std::to_string(variant), team, cx, cy, cz, angle);
    GenerateMinefieldBehavior* gen = find_behavior<GenerateMinefieldBehavior>(producer);

    std::ostringstream blocks;
    const int n_blocks = crowd ? 0 : static_cast<int>(pick(4));
    for (int b = 0; b < n_blocks; ++b) {
        const int s = static_cast<int>(pick(kBlocks));
        const float x = cx + pickf(-60.0f, 60.0f), y = cy + pickf(-60.0f, 60.0f);
        const float z = pick(3) ? cz : pickf(-20.0f, 60.0f);
        const float a = pickf(-3.14f, 3.14f);
        spawn("P279_S" + std::to_string(s), 2, x, y, z, a);
        blocks << (b ? ";" : "") << s << ' ' << bits(x) << ' ' << bits(y) << ' ' << bits(z) << ' ' << bits(a);
    }
    ThePartitionManager->update();

    std::ostringstream ops, rows;
    std::string last_list;
    const int n_ops = crowd ? 4 : 3 + static_cast<int>(pick(6));
    for (int k = 0; k < n_ops; ++k) {
        // 0 die, 1 target, 2 attempt upgrade, 3 force refresh, 4 reset upgrade,
        // 5 grant EMP mines, 6 update, 7 step mines, 8 destroy a mine, 9 save and load,
        // 10 would upgrade, 11 partition update
        uint32_t code = pick(12);
        uint32_t a = 0, b = 0, c = 0, d = 0;
        const bool upgradable = variant == 7 || variant == 8 || variant == 9 || variant == 14 || variant == 15 || variant == 17;
        // Most start by placing, some after being given a target.
        const bool on_death = variant == 1 || variant == 3 || variant == 7 || variant == 9 || variant == 10;
        const uint32_t place = on_death ? 0 : 2;
        const bool aim = k == 0 && pick(4) == 0;
        if (aim) code = 1;
        if (!aim && k < 2 && pick(8) != 0) code = place;
        if (upgradable && k == 2 && pick(2)) code = 5;
        if (upgradable && k == 3 && pick(2)) code = 6;
        if (crowd) code = k == 1 ? 5 : k == 2 ? 9 : 6;
        if (crowd && k == 0) code = 2;
        uint32_t ret = 0;
        g_w->set_frame(frame);
        UpgradeModuleInterface* up = gen;
        UpgradeMaskType mask;
        if (code == 2 || code == 4 || code == 10) {
            a = crowd || pick(5) != 0 ? (variant == 8 ? 6 : 2) : pick(16);
            for (int i = 0; i < 4; ++i)
                if (a & (1u << i)) mask.set(TheUpgradeCenter->findUpgrade(kUpgrades[i])->getUpgradeMask());
        }
        if (code == 0) {
            DamageInfo info;
            info.in.m_damageType = DAMAGE_UNRESISTABLE;
            gen->onDie(&info);
        } else if (code == 1) {
            a = pick(4) == 0 ? 0 : 1;
            Coord3D t;
            t.x = cx + pickf(-40.0f, 40.0f);
            t.y = cy + pickf(-40.0f, 40.0f);
            t.z = pickf(-5.0f, 30.0f);
            b = bits(t.x); c = bits(t.y); d = bits(t.z);
            gen->setMinefieldTarget(a ? &t : nullptr);
        } else if (code == 2) {
            ret = up->attemptUpgrade(mask) ? 1 : 0;
        } else if (code == 3) {
            up->forceRefreshUpgrade();
        } else if (code == 4) {
            ret = up->resetUpgrade(mask) ? 1 : 0;
        } else if (code == 5) {
            (producer->*access_member(UpgradesDone())).set(
                TheUpgradeCenter->findUpgrade("Upgrade_ChinaEMPMines")->getUpgradeMask());
        } else if (code == 6) {
            ret = static_cast<uint32_t>(gen->update());
        } else if (code == 7) {
            a = 1 + pick(20);
            for (uint32_t t = 0; t < a; ++t) {
                g_w->set_frame(++frame);
                for (Object* o : all_objects())
                    if (MinefieldBehavior* m = find_behavior<MinefieldBehavior>(o)) step_module(m, frame);
                g_w->flush_destroy_list();
            }
        } else if (code == 8) {
            std::vector<Object*> mines;
            for (Object* o : all_objects())
                if (is_mine(o)) mines.push_back(o);
            a = pick(64);
            if (!mines.empty()) TheGameLogic->destroyObject(mines[a % mines.size()]);
        } else if (code == 9) {
            const std::string hex = oracle_wp51::World::snapshot_save_hex(static_cast<UpdateModule*>(gen));
            // Past 255 mines the count wraps and the save cannot be read back.
            if (hex.size() < 2 * (4 * 255 + 30)) save_load(static_cast<UpdateModule*>(gen), gen);
            else a = 1;
        } else if (code == 10) {
            ret = up->wouldUpgrade(mask) ? 1 : 0;
        } else {
            ThePartitionManager->update();
        }
        g_w->flush_destroy_list();

        uint32_t save_crc_unused = 0;
        const std::string save = oracle_wp51::World::snapshot_save_hex(static_cast<UpdateModule*>(gen), &save_crc_unused);
        const std::string list = mine_list();
        rows << (k ? "|" : "") << ret << ' ' << save << ' '
             << oracle_wp51::World::snapshot_crc(static_cast<UpdateModule*>(gen)) << ' ' << GetGameLogicRandomSeedCRC()
             << " f" << g_fx << ' ' << (list == last_list ? std::string("=") : "m" + list);
        last_list = list;
        g_fx.clear();
        g_ocls.clear();
        collect_fires();
        if (!g_fires.empty()) {
            std::fprintf(stderr, "a placement fired a weapon\n");
            std::exit(1);
        }
        ops << (k ? ";" : "") << code << ' ' << a << ' ' << b << ' ' << c << ' ' << d;
    }

    std::fprintf(out, "%s\n{\"kind\":\"place\",\"variant\":%d,\"team\":%d,\"seed\":%u,\"frame\":%u,"
                 "\"pose\":\"%u %u %u %u\",\"terrain\":\"%s\",\"blocks\":\"%s\",\"ops\":\"%s\",\"rows\":\"%s\"}",
                 index ? "," : "", variant, team, seed, frame0, bits(cx), bits(cy), bits(cz), bits(angle),
                 terrain.c_str(), blocks.str().c_str(), ops.str().c_str(), rows.str().c_str());
}

// ---- behaviour vectors ----------------------------------------------------------

static void behaviour_vector(FILE* out, int index) {
    clear_world();
    const int variant = static_cast<int>(pick(kMineVariants));
    const bool on_bridge = pick(8) == 0;
    const float cx = on_bridge ? pickf(710.0f, 790.0f) : pickf(150.0f, 600.0f);
    const float cy = on_bridge ? pickf(710.0f, 790.0f) : pickf(150.0f, 600.0f);
    const std::string terrain = random_terrain(cx, cy, false);
    const UnsignedInt seed = 1 + pick(1000000);
    UnsignedInt frame = 1 + pick(5000);
    const UnsignedInt frame0 = frame;
    g_w->set_frame(frame);
    InitRandom(seed);

    const int team = 1 + static_cast<int>(pick(2));
    const int producer_team = pick(4) == 0 ? 3 - team : team;
    const bool has_producer = pick(4) != 0;
    Object* producer = has_producer ? spawn("P279_S1", producer_team, cx + 200.0f, cy, 0.0f, 0.0f) : nullptr;
    const float cz = pick(2) ? 0.0f : pickf(-2.0f, 10.0f);
    const float angle = pickf(-3.14f, 3.14f);
    Object* mine = spawn("P279_M" + std::to_string(variant), team, cx, cy, cz, angle);
    if (producer) mine->setProducer(producer);
    MinefieldBehavior* mb = find_behavior<MinefieldBehavior>(mine);
    AutoHealBehavior* heal = find_behavior<AutoHealBehavior>(mine);

    std::ostringstream units_in;
    std::vector<Object*> units;
    const int n_units = 3 + static_cast<int>(pick(6));
    for (int u = 0; u < n_units; ++u) {
        // Clearers often, so that more than three can be immune at once.
        const int t = pick(3) == 0 ? 4 : static_cast<int>(pick(kUnits));
        const int ut = static_cast<int>(pick(3));
        const float x = cx + pickf(-12.0f, 12.0f), y = cy + pickf(-12.0f, 12.0f), z = cz + pickf(-3.0f, 6.0f);
        Object* o = spawn("P279_U" + std::to_string(t), ut, x, y, z, pickf(-3.14f, 3.14f));
        units.push_back(o);
        units_in << (u ? ";" : "") << t << ' ' << ut << ' ' << bits(x) << ' ' << bits(y) << ' ' << bits(z) << ' '
                 << bits(o->getOrientation());
    }
    // Many armed units are already at work: "attacking goal" for each.
    std::ostringstream busy;
    for (int u = 0; u < n_units; ++u) {
        const uint32_t attacking = pick(3) != 0, goal = pick(3) == 0 ? n_units : pick(n_units);
        if (attacking) units[u]->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_IS_ATTACKING));
        if (units[u]->getAI())
            units[u]->getAI()->friend_setGoalObject(goal < static_cast<uint32_t>(n_units) ? units[goal] : nullptr);
        busy << (u ? " " : "") << attacking << ' ' << goal;
    }
    std::vector<ObjectID> ids;
    for (Object* o : units) ids.push_back(o->getID());
    ThePartitionManager->update();

    const ObjectID mine_id = mine->getID();
    const ObjectID producer_id = producer ? producer->getID() : INVALID_ID;
    auto unit = [&](uint32_t i) { return TheGameLogic->findObjectByID(ids[i]); };

    auto row = [&]() {
        BodyModuleInterface* b = mine->getBodyModule();
        std::ostringstream t;
        t << oracle_wp51::World::snapshot_save_hex(static_cast<UpdateModule*>(mb)) << ' '
          << oracle_wp51::World::snapshot_crc(static_cast<UpdateModule*>(mb)) << ' ' << bits(b->getHealth()) << ' '
          << (mine->testStatus(OBJECT_STATUS_MASKED) ? 1 : 0) << (mine->testStatus(OBJECT_STATUS_NO_ATTACK_FROM_AI) ? 1 : 0)
          << (mine->getDrawable() && mine->getDrawable()->getModelConditionFlags().test(MODELCONDITION_RUBBLE) ? 1 : 0)
          << (mine->isDestroyed() ? 1 : 0) << (mine->isEffectivelyDead() ? 1 : 0) << ' ' << vec_bits(*mine->getPosition())
          << ',' << static_cast<int>(mine->getLayer()) << ' ' << mb->friend_getNextCallFrame() << ' ';
        if (heal) {
            uint32_t crc = 0;
            oracle_wp51::World::snapshot_save_hex(static_cast<UpdateModule*>(heal), &crc);
            t << crc << ',' << heal->friend_getNextCallFrame();
        } else {
            t << '-';
        }
        collect_fires();
        t << " f" << g_fires << " o" << g_ocls << ' ' << GetGameLogicRandomSeedCRC();
        g_fires.clear();
        g_ocls.clear();
        return t.str();
    };

    // The AI modules of the units drew while being built.
    InitRandom(seed + 1);
    const std::string spawn_row = row();
    std::ostringstream ops, rows;
    const int n_ops = 4 + static_cast<int>(pick(22));
    // A unit that took a short step usually runs into the mine again.
    int again = -1;
    for (int k = 0; k < n_ops; ++k) {
        const uint32_t r = pick(100);
        // 0 collide, 1 move a unit, 2 damage, 3 heal, 4 disarm, 5 step, 6 attacking,
        // 7 goal, 8 producer, 9 scoot, 10 destroy a unit, 11 kill flag, 12 save and load,
        // 13 time passes, 14 update
        uint32_t code = r < 30 ? 0 : r < 36 ? 1 : r < 52 ? 2 : r < 57 ? 3 : r < 59 ? 4 : r < 73 ? 5 : r < 78 ? 6
                        : r < 83 ? 7 : r < 86 ? 8 : r < 90 ? 9 : r < 92 ? 10 : r < 94 ? 11 : r < 96 ? 12 : r < 98 ? 13 : 14;
        std::vector<uint32_t> p;
        g_w->set_frame(frame);
        const int forced = again;
        again = -1;
        if (forced >= 0 && pick(4) != 0) code = 0;
        if (code == 0) {
            uint32_t i = pick(40) == 0 ? n_units : pick(n_units);
            if (forced >= 0) i = forced;
            // Come back to a clearer more often.
            else if (i < static_cast<uint32_t>(n_units) && pick(2))
                for (int u = 0; u < n_units; ++u) {
                    Object* o = unit((i + u) % n_units);
                    if (o && o->getAI() && o->getAI()->isClearingMines()) {
                        i = (i + u) % n_units;
                        break;
                    }
                }
            p = {i};
            Object* other = i < static_cast<uint32_t>(n_units) ? unit(i) : nullptr;
            Coord3D loc = *mine->getPosition();
            Coord3D normal;
            normal.x = 0; normal.y = 0; normal.z = 1;
            if (i < static_cast<uint32_t>(n_units) && !other) p = {static_cast<uint32_t>(n_units)};
            mb->onCollide(other, &loc, &normal);
        } else if (code == 1) {
            const uint32_t i = pick(n_units);
            const float far = pick(3) ? 3.0f : 20.0f;
            const Coord3D* at = mine->getPosition();
            Object* o = unit(i);
            Coord3D to;
            to.x = (pick(4) == 0 && o ? o->getPosition()->x : at->x) + pickf(-far, far);
            to.y = at->y + pickf(-far, far);
            to.z = at->z + pickf(-3.0f, 3.0f);
            // A short step from where it stands, about the repeat threshold.
            if (o && pick(3) == 0) {
                static const float kShort[6] = {0.5f, 1.0f, 1.0625f, 1.125f, 1.5f, 2.0f};
                to = *o->getPosition();
                to.x += kShort[pick(6)];
                again = static_cast<int>(i);
            }
            p = {i, bits(to.x), bits(to.y), bits(to.z)};
            if (o) o->setPosition(&to);
        } else if (code == 2) {
            static const DamageType kTypes[4] = {DAMAGE_EXPLOSION, DAMAGE_SMALL_ARMS, DAMAGE_UNRESISTABLE, DAMAGE_HEALING};
            const uint32_t type = pick(16) == 0 ? 3 : pick(3);
            const float max_health = mine->getBodyModule()->getMaxHealth();
            const uint32_t how = pick(8);
            const float amount = how == 0 ? max_health * 2.0f : how == 1 ? max_health / 4.0f
                                 : how == 2 ? mine->getBodyModule()->getHealth() : max_health * pickf(0.0f, 0.6f);
            // 0 nobody, 1 the mine, 2.. a unit
            const uint32_t src = pick(2) ? 2 + pick(n_units) : pick(2);
            p = {static_cast<uint32_t>(kTypes[type]), bits(amount), src};
            DamageInfo info;
            info.in.m_damageType = kTypes[type];
            info.in.m_deathType = DEATH_NORMAL;
            info.in.m_amount = amount;
            info.in.m_sourceID = src == 0 ? INVALID_ID : src == 1 ? mine_id : ids[src - 2];
            mine->attemptDamage(&info);
        } else if (code == 3) {
            const float amount = mine->getBodyModule()->getMaxHealth() * pickf(0.0f, 1.2f);
            p = {bits(amount)};
            mine->attemptHealing(amount, mine);
        } else if (code == 4) {
            mb->disarm();
        } else if (code == 5) {
            const uint32_t frames = 1 + pick(pick(3) ? 6 : 60);
            p = {frames};
            for (uint32_t t = 0; t < frames && !mine->isDestroyed(); ++t) {
                g_w->set_frame(++frame);
                step_module(mb, frame);
                if (heal && !mine->isDestroyed()) step_module(heal, frame);
                collect_fires();
            }
        } else if (code == 6) {
            const uint32_t i = pick(n_units), on = pick(5) != 0;
            p = {i, on};
            if (Object* o = unit(i)) {
                if (on) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_IS_ATTACKING));
                else o->clearStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_IS_ATTACKING));
            }
        } else if (code == 7) {
            const uint32_t i = pick(n_units), j = pick(n_units + 1);
            p = {i, j};
            Object* o = unit(i);
            if (o && o->getAI()) o->getAI()->friend_setGoalObject(j < static_cast<uint32_t>(n_units) ? unit(j) : nullptr);
        } else if (code == 8) {
            const uint32_t how = pick(2);
            p = {how};
            if (Object* o = TheGameLogic->findObjectByID(producer_id)) {
                if (how == 0) TheGameLogic->destroyObject(o);
                else o->setEffectivelyDead(TRUE);
            }
        } else if (code == 9) {
            const Coord3D* at = mine->getPosition();
            Coord3D s, e;
            const bool near = pick(6) == 0;
            s.x = at->x + pickf(-30.0f, 30.0f);
            s.y = at->y + pickf(-30.0f, 30.0f);
            s.z = pick(3) == 0 ? pickf(-10.0f, 0.0f) : pickf(0.0f, 60.0f);
            e.x = near ? s.x + (pick(2) ? 0.0f : 0.0625f) : at->x + pickf(-30.0f, 30.0f);
            e.y = near ? s.y : at->y + pickf(-30.0f, 30.0f);
            e.z = pickf(-5.0f, 20.0f);
            if (near && pick(2)) s.z = TheTerrainLogic->getGroundHeight(e.x, e.y) + (pick(2) ? 0.0f : 0.0625f);
            // Short hops, either side of the 0.1 limits.
            if (pick(4) == 0) {
                static const float kHop[6] = {0.0f, 0.0625f, 0.125f, 0.1875f, 0.5f, 1.0f};
                e.x = s.x + kHop[pick(6)];
                e.y = s.y;
                s.z = TheTerrainLogic->getGroundHeight(e.x, e.y) + (pick(2) ? -kHop[pick(6)] : kHop[pick(6)]);
            }
            p = {bits(s.x), bits(s.y), bits(s.z), bits(e.x), bits(e.y), bits(e.z)};
            mb->setScootParms(s, e);
        } else if (code == 10) {
            const uint32_t i = pick(n_units);
            p = {i};
            if (Object* o = unit(i)) {
                // The goal of a state machine is looked up by ID, so it may go.
                TheGameLogic->destroyObject(o);
            }
        } else if (code == 11) {
            const uint32_t i = pick(n_units), on = pick(2);
            p = {i, on};
            if (Object* o = unit(i)) o->setEffectivelyDead(on ? TRUE : FALSE);
        } else if (code == 12) {
            save_load(static_cast<UpdateModule*>(mb), mb);
        } else if (code == 13) {
            const uint32_t frames = 1 + pick(5);
            p = {frames};
            frame += frames;
        } else {
            mb->update();
        }
        const std::string state = row();
        ops << (k ? ";" : "") << code;
        for (uint32_t v : p) ops << ' ' << v;
        rows << (k ? "|" : "") << state;
        const bool gone = mine->isDestroyed();
        g_w->flush_destroy_list();
        if (gone) break;
        // Contacts are processed here, so mines are set off here too.
        ThePartitionManager->update();
        collect_fires();
    }

    std::fprintf(out, ",\n{\"kind\":\"mine\",\"variant\":%d,\"team\":%d,\"producer\":%d,\"seed\":%u,\"frame\":%u,"
                 "\"pose\":\"%u %u %u %u\",\"terrain\":\"%s\",\"units\":\"%s\",\"busy\":\"%s\",\"spawn\":\"%s\",\"ops\":\"%s\",\"rows\":\"%s\"}",
                 variant, team, has_producer ? producer_team : -1, seed, frame0, bits(cx), bits(cy), bits(cz),
                 bits(angle), terrain.c_str(), units_in.str().c_str(), busy.str().c_str(), spawn_row.c_str(), ops.str().c_str(),
                 rows.str().c_str());
}

// ---- leaf vectors -----------------------------------------------------------------

static GeometryInfo random_geometry(std::ostringstream& in) {
    const int type = static_cast<int>(pick(3));
    const float h = pickf(0.5f, 20.0f), major = pickf(0.5f, 25.0f), minor = pickf(0.5f, 25.0f);
    GeometryInfo g(static_cast<GeometryType>(type), pick(2) != 0, h, major, minor);
    in << type << ' ' << bits(h) << ' ' << bits(major) << ' ' << bits(minor);
    return g;
}

static std::string geometry_state(const GeometryInfo& g) {
    std::ostringstream t;
    t << static_cast<int>(g.getGeomType()) << ',' << bits(g.getMajorRadius()) << ',' << bits(g.getMinorRadius()) << ','
      << bits(g.getBoundingCircleRadius()) << ',' << bits(g.getBoundingSphereRadius()) << ',' << bits(g.getFootprintArea());
    return t.str();
}

static void geometry_vector(FILE* out) {
    std::ostringstream in, ops, rows;
    GeometryInfo g = random_geometry(in);
    const UnsignedInt seed = 1 + pick(1000000);
    InitRandom(seed);
    const int n = 4 + static_cast<int>(pick(8));
    for (int k = 0; k < n; ++k) {
        // 0 expand, 1 make a cylinder, 2 point in footprint, 3 clip, 4 random offset
        const uint32_t code = pick(5);
        std::vector<uint32_t> p;
        std::ostringstream t;
        if (code == 0) {
            // Keep a radius, or the random offset of a round footprint never returns.
            float r = pickf(-6.0f, 8.0f);
            if (g.getMajorRadius() + r <= 0.25f || g.getMinorRadius() + r <= 0.25f) r = 2.0f;
            p = {bits(r)};
            g.expandFootprint(r);
        } else if (code == 1) {
            const Real r = g.getBoundingCircleRadius();
            g.set(GEOMETRY_CYLINDER, false, 1, r, r);
        } else if (code == 2 || code == 3) {
            Coord3D c, q;
            c.x = pickf(-50.0f, 50.0f); c.y = pickf(-50.0f, 50.0f); c.z = pickf(-5.0f, 5.0f);
            const float reach = pick(2) ? 30.0f : 5.0f;
            q.x = c.x + pickf(-reach, reach); q.y = c.y + pickf(-reach, reach); q.z = pickf(-5.0f, 5.0f);
            if (pick(6) == 0) q.x = c.x + (pick(2) ? g.getMajorRadius() : -g.getMajorRadius()), q.y = c.y;
            p = {bits(c.x), bits(c.y), bits(c.z), bits(q.x), bits(q.y), bits(q.z)};
            if (code == 2) {
                t << (g.isPointInFootprint(c, q) ? 1 : 0) << ' ';
            } else {
                g.clipPointToFootprint(c, q);
                t << vec_bits(q) << ' ';
            }
        } else {
            Coord3D q;
            q.x = 1; q.y = 2; q.z = 3;
            g.makeRandomOffsetWithinFootprint(q);
            t << vec_bits(q) << ' ';
        }
        t << geometry_state(g) << ' ' << GetGameLogicRandomSeedCRC();
        ops << (k ? ";" : "") << code;
        for (uint32_t v : p) ops << ' ' << v;
        rows << (k ? "|" : "") << t.str();
    }
    std::fprintf(out, ",\n{\"kind\":\"geom\",\"geom\":\"%s\",\"seed\":%u,\"ops\":\"%s\",\"rows\":\"%s\"}",
                 in.str().c_str(), seed, ops.str().c_str(), rows.str().c_str());
}

static void cliff_vector(FILE* out) {
    const Int w = 3 + static_cast<Int>(pick(18)), h = 3 + static_cast<Int>(pick(18));
    const Int border = static_cast<Int>(pick(4));
    const bool from_heights = pick(2) != 0;
    g_height_map.build(w, h, border);
    std::string data;
    static const char* hex = "0123456789abcdef";
    if (from_heights) {
        const int base = static_cast<int>(pick(100));
        // Differences around the cliff limit of 9.8 / 0.625 = 15.68 levels.
        const int spread = 12 + static_cast<int>(pick(10));
        for (UnsignedByte& v : g_height_map.data) {
            v = static_cast<UnsignedByte>(base + static_cast<int>(pick(spread)));
            data += hex[v >> 4];
            data += hex[v & 15];
        }
        (g_height_map.map->*access_member(HmInitTag()))();
    } else {
        for (Int y = 0; y < h; ++y)
            for (Int x = 0; x < w; ++x) {
                const bool set = pick(3) == 0;
                if (set) g_height_map.set_bit(x, y);
                data += set ? '1' : '0';
            }
    }
    g_render->*access_member(BhmMap()) = g_height_map.map;
    std::string flags;
    for (Int y = -1; y <= h; ++y)
        for (Int x = -1; x <= w; ++x) flags += g_height_map.map->getCliffState(x, y) ? '1' : '0';
    std::ostringstream points;
    std::string answers;
    for (int k = 0; k < 40; ++k) {
        float x = pickf(-60.0f, w * 10.0f + 40.0f), y = pickf(-60.0f, h * 10.0f + 40.0f);
        if (k == 0) x = -1.0e12f, y = -1.0e12f;
        if (k == 1) x = 1.0e8f, y = 1.0e8f;
        if (k == 2) x = -9.5f, y = -0.5f;
        points << (k ? " " : "") << bits(x) << ' ' << bits(y);
        answers += g_render->isCliffCell(x, y) ? '1' : '0';
    }
    std::fprintf(out, ",\n{\"kind\":\"cliff\",\"size\":\"%d %d %d %d\",\"data\":\"%s\",\"flags\":\"%s\",\"points\":\"%s\",\"answers\":\"%s\"}",
                 w, h, border, from_heights ? 1 : 0, data.c_str(), flags.c_str(), points.str().c_str(), answers.c_str());
}

static void overlap_vector(FILE* out) {
    clear_world();
    const float cx = pickf(150.0f, 600.0f), cy = pickf(150.0f, 600.0f);
    std::ostringstream in, queries;
    std::string answers;
    const int n = 2 + static_cast<int>(pick(10));
    static const char* const kNames[6] = {"P279_S0", "P279_S1", "P279_S2", "P279_U2", "P279_U0", "P279_G3"};
    for (int i = 0; i < n; ++i) {
        const int t = static_cast<int>(pick(6));
        const float x = cx + pickf(-30.0f, 30.0f), y = cy + pickf(-30.0f, 30.0f), z = pickf(-8.0f, 14.0f);
        const float a = pickf(-3.14f, 3.14f);
        spawn(kNames[t], 1, x, y, z, a);
        in << (i ? ";" : "") << t << ' ' << bits(x) << ' ' << bits(y) << ' ' << bits(z) << ' ' << bits(a);
    }
    ThePartitionManager->update();
    for (int k = 0; k < 8; ++k) {
        std::ostringstream g_in;
        const GeometryInfo g = random_geometry(g_in);
        Coord3D at;
        at.x = cx + pickf(-30.0f, 30.0f); at.y = cy + pickf(-30.0f, 30.0f); at.z = pickf(-20.0f, 25.0f);
        const float angle = pickf(-3.14f, 3.14f);
        const int use2d = pick(3) == 0;
        queries << (k ? ";" : "") << g_in.str() << ' ' << bits(at.x) << ' ' << bits(at.y) << ' ' << bits(at.z) << ' '
                << bits(angle) << ' ' << use2d;
        ObjectIterator* iter = ThePartitionManager->iteratePotentialCollisions(&at, g, angle, use2d != 0);
        MemoryPoolObjectHolder hold(iter);
        if (k) answers += ';';
        for (Object* o = iter->first(); o; o = iter->next()) answers += std::to_string(o->getID()) + ".";
    }
    std::fprintf(out, ",\n{\"kind\":\"overlap\",\"objects\":\"%s\",\"queries\":\"%s\",\"answers\":\"%s\"}",
                 in.str().c_str(), queries.str().c_str(), answers.c_str());
}

// ---- main -----------------------------------------------------------------------

static std::string hex_of(const std::string& s) {
    static const char* hex = "0123456789abcdef";
    std::string out;
    for (unsigned char c : s) {
        out += hex[c >> 4];
        out += hex[c & 15];
    }
    return out;
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_pkt272_279_minefield <out.json>\n");
        return 2;
    }
    static oracle_wp51::World w;
    g_w = &w;
    g_render = static_cast<BaseHeightMapRenderObjClass*>(std::calloc(1, sizeof(BaseHeightMapRenderObjClass)));
    TheTerrainLogic = NEW MineTerrain;
    TheTerrainLogic->init();
    TheWritableGlobalData->m_gravity = -1.0f;
    // Not the defaults, so the module data must have read them.
    TheWritableGlobalData->m_standardMinefieldDistance = 33.0f;
    TheWritableGlobalData->m_standardMinefieldDensity = 0.006f;

    for (const char* name : kUpgrades) w.load_ini(std::string("Upgrade ") + name + "\n  Type = OBJECT\nEnd\n");
    w.load_ini(weapon_ini());
    w.load_ini("FXList FX_P272\nEnd\nObjectCreationList P279_OCL\nEnd\n");
    const_cast<FXList*>(TheFXListStore->findFXList("FX_P272"))->addFXNugget(newInstance(FxSpy));
    const_cast<ObjectCreationList*>(TheObjectCreationListStore->findObjectCreationList("P279_OCL"))
        ->addObjectCreationNugget(newInstance(OclSpy));
    const std::string objects = object_ini();
    w.load_ini(objects);
    w.init_with_neutral_player();
    Dict dict;
    TheTeamFactory->initTeam("TeamA2", AsciiString::TheEmptyString, TRUE, &dict);
    g_team_a2 = TheTeamFactory->findTeam("TeamA2");
    g_team_a2->setControllingPlayer(w.team_a->getControllingPlayer());
    // The FX list is skipped for a shrouded object. A shroud edge for the local player
    // reaches the display, so each player is revealed while another one is local.
    Player* const was_local = ThePlayerList->getLocalPlayer();
    for (int i = 0; i < 3; ++i) {
        ThePlayerList->setLocalPlayer(ThePlayerList->getNthPlayer((i + 1) % 3));
        ThePartitionManager->revealMapForPlayerPermanently(i);
    }
    ThePlayerList->setLocalPlayer(was_local);

    BridgeInfo bridge;
    bridge.fromLeft.x = kBridgeLo; bridge.fromLeft.y = kBridgeLo; bridge.fromLeft.z = kBridgeZ;
    bridge.fromRight.x = kBridgeLo; bridge.fromRight.y = kBridgeHi; bridge.fromRight.z = kBridgeZ;
    bridge.toLeft.x = kBridgeHi; bridge.toLeft.y = kBridgeLo; bridge.toLeft.z = kBridgeZ;
    bridge.toRight.x = kBridgeHi; bridge.toRight.y = kBridgeHi; bridge.toRight.z = kBridgeZ;
    bridge.from.x = kBridgeLo; bridge.from.y = 750.0f; bridge.from.z = kBridgeZ;
    bridge.to.x = kBridgeHi; bridge.to.y = 750.0f; bridge.to.z = kBridgeZ;
    bridge.bridgeWidth = kBridgeHi - kBridgeLo;
    Dict bridge_dict;
    TheTerrainLogic->addBridgeToLogic(&bridge, &bridge_dict, "P272_NoSuchBridge");
    {
        Coord3D on;
        on.x = 750.0f; on.y = 750.0f; on.z = 99999.0f;
        g_bridge_layer = static_cast<int>(TheTerrainLogic->getHighestLayerForDestination(&on));
        if (g_bridge_layer == LAYER_GROUND) {
            std::fprintf(stderr, "the bridge has no layer\n");
            return 1;
        }
    }

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* out = std::fopen(tmp.c_str(), "w");
    if (!out) return 1;
    const int kPlace = 600, kMine = 900, kGeom = 250, kCliff = 120, kOverlap = 200;
    std::fprintf(out, "{\"vector_count\":%d,\"bridge_layer\":%d,\"ini\":\"%s\",\"weapons\":\"%s\",\"vectors\":[",
                 kPlace + kMine + kGeom + kCliff + kOverlap, g_bridge_layer, hex_of(objects).c_str(),
                 hex_of(weapon_ini()).c_str());
    for (int i = 0; i < kPlace; ++i) placement_vector(out, i);
    for (int i = 0; i < kMine; ++i) behaviour_vector(out, i);
    for (int i = 0; i < kGeom; ++i) geometry_vector(out);
    for (int i = 0; i < kCliff; ++i) cliff_vector(out);
    for (int i = 0; i < kOverlap; ++i) overlap_vector(out);
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    if (std::rename(tmp.c_str(), argv[1]) != 0) return 1;
    return 0;
}
