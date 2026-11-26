// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for HordeUpdate (PKT-336), SpectreGunshipDeploymentUpdate
// (PKT-370), AutoFindHealingUpdate (PKT-306) and AssistedTargetingUpdate
// (PKT-304), run on the unmodified reference modules in a small real world.
// The AI is the real AIUpdateInterface (so evaluateMoraleBonus is the reference
// one) with aiDoCommand recorded and isIdle set by the harness.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt336_370_306_304_leaves.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT336_370_306_304_Leaves.json
//
// Not covered: loading a save; objects without a drawable; a horde update rate
// of zero; a Spectre without its special power module or gunship template, or
// triggered without a target (all null dereferences); a map whose low corner is
// not the origin; isAbleToAttack through spawned or slaved objects; the turret
// test of isAbleToAttack is driven through the real turrets of the AI.

#include "oracle_wp51_world.h"

#include "Common/DrawModule.h"
#include "Common/ModuleFactory.h"
#include "Common/Player.h"
#include "Common/SpecialPower.h"
#include "Common/Team.h"
#include "Common/Upgrade.h"
#include "GameClient/Drawable.h"
#include "GameLogic/Module/LaserUpdate.h"
#include "GameLogic/AI.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/AssistedTargetingUpdate.h"
#include "GameLogic/Module/AutoFindHealingUpdate.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/HordeUpdate.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Module/SpectreGunshipDeploymentUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/Weapon.h"

#include <cstdio>
#include <cstring>
#include <map>
#include <set>
#include <sstream>
#include <string>
#include <vector>

// Private members read or staged by the harness.
struct DrawArrayTag { using type = Module** (Drawable::*)[NUM_DRAWABLE_MODULE_TYPES]; };
template struct MemberAccess<DrawArrayTag, &Drawable::m_modules>;
DrawArrayTag::type access_member(DrawArrayTag);
struct FadeTargetTag { using type = Real Drawable::*; };
template struct MemberAccess<FadeTargetTag, &Drawable::m_decalOpacityFadeTarget>;
FadeTargetTag::type access_member(FadeTargetTag);
struct FadeRateTag { using type = Real Drawable::*; };
template struct MemberAccess<FadeRateTag, &Drawable::m_decalOpacityFadeRate>;
FadeRateTag::type access_member(FadeRateTag);
struct WStatusTag { using type = WeaponStatus Weapon::*; };
template struct MemberAccess<WStatusTag, &Weapon::m_status>;
WStatusTag::type access_member(WStatusTag);
struct WAmmoTag { using type = UnsignedInt Weapon::*; };
template struct MemberAccess<WAmmoTag, &Weapon::m_ammoInClip>;
WAmmoTag::type access_member(WAmmoTag);
struct WAgainTag { using type = UnsignedInt Weapon::*; };
template struct MemberAccess<WAgainTag, &Weapon::m_whenWeCanFireAgain>;
WAgainTag::type access_member(WAgainTag);
struct WPreTag { using type = UnsignedInt Weapon::*; };
template struct MemberAccess<WPreTag, &Weapon::m_whenPreAttackFinished>;
WPreTag::type access_member(WPreTag);

struct LaserParentTag { using type = DrawableID LaserUpdate::*; };
template struct MemberAccess<LaserParentTag, &LaserUpdate::m_parentID>;
LaserParentTag::type access_member(LaserParentTag);

struct ExposedFactory : public ModuleFactory { using ModuleFactory::addModuleInternal; };

static void die(const char* what) {
    std::fprintf(stderr, "oracle: %s\n", what);
    std::exit(1);
}

static uint32_t g_state = 0x33637030u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}

static uint32_t bits(Real r) {
    uint32_t b;
    std::memcpy(&b, &r, 4);
    return b;
}

static void put(std::ostringstream& out, const Coord3D& c) {
    out << ' ' << bits(c.x) << ' ' << bits(c.y) << ' ' << bits(c.z);
}

// --- recording AI -----------------------------------------------------------

static std::set<ObjectID> g_busy;
static std::ostringstream g_orders;

class RecAI : public AIUpdateInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(RecAI, "OracleLeafRecAI", 64, 64)
public:
    RecAI(Thing* thing, const ModuleData* data) : AIUpdateInterface(thing, data) {}
    void aiDoCommand(const AICommandParms* parms) override {
        const int me = static_cast<int>(getObject()->getID());
        if (parms->m_cmd == AICMD_GET_HEALED) {
            g_orders << " H " << me << ' ' << static_cast<int>(parms->m_obj->getID()) << ' '
                     << static_cast<int>(parms->m_cmdSource);
        } else if (parms->m_cmd == AICMD_ATTACK_OBJECT) {
            g_orders << " A " << me << ' ' << static_cast<int>(parms->m_obj->getID()) << ' '
                     << parms->m_intValue << ' ' << static_cast<int>(parms->m_cmdSource);
        }
    }
    Bool isIdle() const override { return g_busy.count(getObject()->getID()) == 0; }
};
RecAI::~RecAI() {}

static Module* make_rec_ai(Thing* t, const ModuleData* d) { return newInstance(RecAI)(t, d); }

// --- terrain: a slope, so edge heights matter --------------------------------

class SlopeTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return x * 0.25f + y * 0.125f;
    }
};

// --- shared helpers ---------------------------------------------------------

template <typename T>
static T* find_behavior(Object* obj) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (T* typed = dynamic_cast<T*>(*m)) return typed;
    }
    return nullptr;
}

static std::vector<SpecialPowerModule*> spms(Object* obj) {
    std::vector<SpecialPowerModule*> out;
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (SpecialPowerModule* typed = dynamic_cast<SpecialPowerModule*>(*m)) out.push_back(typed);
    }
    return out;
}

static std::string module_state(UpdateModule* m) {
    std::ostringstream out;
    out << oracle_wp51::World::snapshot_save_hex(m) << ' ' << oracle_wp51::World::snapshot_crc(m)
        << ' ' << m->friend_getNextCallFrame();
    return out.str();
}

// The harness's single-module scheduler step.
static void step(UpdateModule* u, UnsignedInt frame) {
    if (u->friend_getNextCallFrame() > frame) return;
    UpdateSleepTime sleep_len = u->update();
    if (sleep_len < 1) sleep_len = UPDATE_SLEEP_NONE;
    UnsignedInt next = frame + static_cast<UnsignedInt>(sleep_len);
    if (next > UPDATE_SLEEP_FOREVER) next = UPDATE_SLEEP_FOREVER;
    u->friend_setNextCallFrame(next);
}

static Object* spawn(oracle_wp51::World& w, const std::string& name, Team* team, const Coord3D& pos) {
    const ThingTemplate* tt = TheThingFactory->findTemplate(name.c_str());
    if (!tt) die(("missing template " + name).c_str());
    Object* o = w.spawn_object(name.c_str(), team, pos);
    if (!o) die("spawn failed");
    return o;
}

static std::vector<Object*> live_objects() {
    std::vector<Object*> v;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) v.push_back(o);
    return v;
}

static void clear_world(oracle_wp51::World& w) {
    for (Object* o : live_objects()) TheGameLogic->destroyObject(o);
    w.flush_destroy_list();
    ThePartitionManager->update();
    g_busy.clear();
}

// Object ids are counted up; each vector records the id its first object gets.
static ObjectID g_next_probe_base(oracle_wp51::World& w) {
    Object* probe = spawn(w, "P_Prop", w.team_a, Coord3D{500.0f, 500.0f, 0.0f});
    const ObjectID id = probe->getID();
    TheGameLogic->destroyObject(probe);
    w.flush_destroy_list();
    return static_cast<ObjectID>(static_cast<int>(id) + 1);
}

static std::string ints(const std::vector<long>& v) {
    std::ostringstream out;
    out << '[';
    for (size_t i = 0; i < v.size(); ++i) out << (i ? "," : "") << v[i];
    out << ']';
    return out.str();
}

static std::string table(const std::vector<std::vector<long>>& v) {
    std::ostringstream out;
    for (size_t i = 0; i < v.size(); ++i) {
        out << (i ? ";" : "");
        for (size_t k = 0; k < v[i].size(); ++k) out << (k ? " " : "") << v[i][k];
    }
    return out.str();
}

struct Vector {
    std::vector<std::vector<long>> units;
    std::vector<std::vector<long>> ops;
    std::vector<std::string> rows;
    std::vector<long> setup;
};

// ---------------------------------------------------------------------------
// Fixture
// ---------------------------------------------------------------------------

struct HordeVariant {
    const char* kinds;
    const char* rate;     // milliseconds; nullptr: default
    const char* filter;   // KindOf of the horde; nullptr: empty
    int count;
    int radius;
    int rub;              // negative: default
    int allies_only;      // -1: default
    int exact;
    const char* action;   // nullptr: default
    int nationalism;      // -1: default
    int geometry;
    int flag;
};

static const HordeVariant kHorde[] = {
    {"INFANTRY", "100", "INFANTRY", 3, 40, 20, -1, 0, nullptr, -1, 5, 1},
    {"INFANTRY", nullptr, "INFANTRY", 2, 30, -1, 1, 0, "HORDE", 1, 5, 0},
    {"INFANTRY", "200", "INFANTRY", 5, 60, 40, 0, 0, "HORDE_FIXED", -1, 10, 0},
    {"VEHICLE", "100", "VEHICLE", 3, 40, 30, -1, 0, nullptr, -1, 10, 1},
    {"VEHICLE", "300", "VEHICLE", 2, 25, 5, 0, 1, "HORDE_FIXED", 0, 5, 0},
    {"VEHICLE PORTABLE_STRUCTURE", "100", "VEHICLE", 2, 40, 20, -1, 0, nullptr, -1, 5, 0},
    {"INFANTRY VEHICLE", "100", "INFANTRY VEHICLE", 2, 40, 20, -1, 0, nullptr, -1, 5, 0},
    {"INFANTRY", "33", nullptr, 4, 50, 20, 0, 0, nullptr, -1, 20, 0},
    {"VEHICLE", "100", nullptr, 0, 20, 20, 0, 0, nullptr, -1, 5, 0},
    {"INFANTRY", "100", "INFANTRY", 1, 0, 50, -1, 1, "HORDE_FIXED", 1, 5, 0},
};
static const int kHordeVariants = static_cast<int>(sizeof(kHorde) / sizeof(kHorde[0]));

static const char* const kScanRate[] = {nullptr, "33", "100", "300"};
static const int kScanRange[] = {0, 50, 100, 150};
static const char* const kNeverHeal[] = {nullptr, "0.5", "1.0", "0.25"};
static const int kHealers = 8;

static const int kSpectres = 8;
static const int kGunships = 3;

static const int kAssist = 14;

static std::string body(const char* extra = "") {
    return std::string("  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n"
                       "    InitialHealth = 100\n") + extra + "  End\n";
}

static std::string ai(const std::string& extra = "", const char* loco = "P_Ground") {
    return "  Behavior = OracleLeafAI ModuleTag_AI\n" + extra + "  End\n" +
           "  Locomotor = SET_NORMAL " + loco + "\n";
}

static std::string geometry(int radius) {
    return "  Geometry = CYLINDER\n  GeometryMajorRadius = " + std::to_string(radius) +
           "\n  GeometryHeight = 10\n";
}

static std::string fixture_ini() {
    std::ostringstream s;
    s << "Upgrade Upgrade_Nationalism\n  Type = PLAYER\nEnd\n";
    s << "Upgrade Upgrade_Fanaticism\n  Type = PLAYER\nEnd\n";
    s << "Locomotor P_Ground\n  Surfaces = GROUND\n  Speed = 60\n  TurnRate = 180\n"
         "  Acceleration = 60\n  Braking = 60\nEnd\n";
    const int heights[kGunships] = {70, 0, 125};
    for (int g = 0; g < kGunships; ++g)
        s << "Locomotor P_Gun" << g << "\n  Surfaces = AIR\n  Speed = 60\n  TurnRate = 180\n"
          << "  Acceleration = 60\n  Braking = 60\n  MinSpeed = 0\n  Lift = 100\n"
          << "  PreferredHeight = " << heights[g] << "\nEnd\n";
    for (int w = 0; w < 2; ++w)
        s << "Weapon P_Weapon" << w << "\n  PrimaryDamage = 1\n  AttackRange = 100\n"
          << "  DelayBetweenShots = 500\n  ClipSize = " << (w ? 3 : 0)
          << "\n  ClipReloadTime = 1000\nEnd\n";
    s << "SpecialPower P_SP0\n  Enum = SPECIAL_DAISY_CUTTER\n  ReloadTime = 3000\nEnd\n";
    s << "SpecialPower P_SP1\n  Enum = SPECIAL_PARADROP_AMERICA\n  ReloadTime = 7000\nEnd\n";
    s << "Object P_Prop\n  KindOf = PRELOAD\nEnd\n";

    for (int v = 0; v < kHordeVariants; ++v) {
        const HordeVariant& h = kHorde[v];
        for (int has_ai = 0; has_ai < 2; ++has_ai) {
            s << "Object P336_" << v << '_' << has_ai << "\n  KindOf = PRELOAD " << h.kinds << '\n'
              << body() << geometry(h.geometry);
            if (has_ai) s << ai();
            s << "  Behavior = HordeUpdate ModuleTag_Horde\n";
            if (h.rate) s << "    UpdateRate = " << h.rate << '\n';
            if (h.filter) s << "    KindOf = " << h.filter << '\n';
            s << "    Count = " << h.count << "\n    Radius = " << h.radius << '\n';
            if (h.rub >= 0) s << "    RubOffRadius = " << h.rub << '\n';
            if (h.allies_only >= 0) s << "    AlliesOnly = " << (h.allies_only ? "Yes" : "No") << '\n';
            if (h.exact) s << "    ExactMatch = Yes\n";
            if (h.action) s << "    Action = " << h.action << '\n';
            if (h.nationalism >= 0) s << "    AllowedNationalism = " << (h.nationalism ? "Yes" : "No") << '\n';
            if (h.flag) s << "    FlagSubObjectNames = FLAG01 FLAG02\n";
            s << "  End\nEnd\n";
        }
    }
    s << "Object P336_Outsider\n  KindOf = PRELOAD INFANTRY VEHICLE\n" << body() << geometry(5)
      << "End\n";

    for (int v = 0; v < kHealers; ++v) {
        s << "Object P306_" << v << "\n  KindOf = PRELOAD INFANTRY" << (v == 3 ? " HEAL_PAD" : "")
          << '\n' << (v == 5 ? std::string() : body()) << geometry(5);
        if (v != 6) s << ai();
        s << "  Behavior = AutoFindHealingUpdate ModuleTag_Find\n";
        if (kScanRate[v % 4]) s << "    ScanRate = " << kScanRate[v % 4] << '\n';
        s << "    ScanRange = " << kScanRange[(v + 1) % 4] << '\n';
        if (kNeverHeal[(v / 2) % 4]) s << "    NeverHeal = " << kNeverHeal[(v / 2) % 4] << '\n';
        if (v % 2) s << "    AlwaysHeal = 0.9\n";
        s << "  End\nEnd\n";
    }
    s << "Object P306_Pad\n  KindOf = PRELOAD STRUCTURE HEAL_PAD\n" << body() << geometry(15) << "End\n";
    s << "Object P306_Other\n  KindOf = PRELOAD STRUCTURE\n" << body() << geometry(15) << "End\n";

    for (int g = 0; g < kGunships; ++g) {
        s << "Object P370_Gunship" << g << "\n  KindOf = PRELOAD VEHICLE AIRCRAFT SELECTABLE\n"
          << body() << geometry(10) << ai("", ("P_Gun" + std::to_string(g)).c_str());
        if (g != 1)
            s << "  Behavior = SpecialAbility ModuleTag_SP\n    SpecialPowerTemplate = P_SP0\n"
              << "    UpdateModuleStartsAttack = " << (g == 2 ? "Yes" : "No") << "\n  End\n";
        s << "End\n";
    }
    // power list, starts-attack, the update's power, create location, gunship
    struct Spectre { const char* powers; int starts; int mine; int loc; int gunship; };
    const Spectre spectres[kSpectres] = {
        {"0", 1, 0, -1, 0}, {"0", 0, 0, 0, 1}, {"10", 1, 0, 1, 2},
        {"00", 1, 0, 2, 0}, {"01", 0, 0, 3, 2}, {"10", 0, 1, 0, 0},
        {"01", 0, 1, 2, 0}, {"0", 1, 0, 1, 0},
    };
    // The last two carry a second update, for P_SP0 with another gunship.
    const char* const locs[4] = {"CREATE_AT_EDGE_NEAR_SOURCE", "CREATE_AT_EDGE_FARTHEST_FROM_SOURCE",
                                 "CREATE_AT_EDGE_NEAR_TARGET", "CREATE_AT_EDGE_FARTHEST_FROM_TARGET"};
    for (int v = 0; v < kSpectres; ++v) {
        const Spectre& c = spectres[v];
        s << "Object P370_Source" << v << "\n  KindOf = PRELOAD STRUCTURE\n" << body() << geometry(30);
        for (int k = 0; c.powers[k]; ++k)
            s << "  Behavior = SpecialAbility ModuleTag_SP" << k << "\n    SpecialPowerTemplate = P_SP"
              << c.powers[k] << "\n    UpdateModuleStartsAttack = " << (c.starts ? "Yes" : "No")
              << "\n  End\n";
        s << "  Behavior = SpectreGunshipDeploymentUpdate ModuleTag_Spectre\n"
          << "    SpecialPowerTemplate = P_SP" << c.mine << "\n    GunshipTemplateName = P370_Gunship"
          << c.gunship << "\n    AttackAreaRadius = 150\n";
        if (c.loc >= 0) s << "    CreateLocation = " << locs[c.loc] << '\n';
        s << "  End\n";
        if (v >= 6)
            s << "  Behavior = SpectreGunshipDeploymentUpdate ModuleTag_Second\n"
                 "    SpecialPowerTemplate = P_SP0\n    GunshipTemplateName = P370_Gunship2\n  End\n";
        s << "End\n";
    }

    s << "Object P304_Laser\n  KindOf = PRELOAD INERT\n"
         "  ClientUpdate = LaserUpdate ModuleTag_Laser\n  End\nEnd\n";
    s << "Object P304_Plain\n  KindOf = PRELOAD INERT\nEnd\n";
    s << "Object P304_Victim\n  KindOf = PRELOAD STRUCTURE\n" << body() << geometry(10) << "End\n";
    for (int c = 0; c < 2; ++c)
        s << "Object P304_Bunker" << c << "\n  KindOf = PRELOAD STRUCTURE\n" << body() << geometry(20)
          << "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 4\n"
          << "    PassengersAllowedToFire = " << (c ? "Yes" : "No")
          << "\n    AllowInsideKindOf = INFANTRY\n  End\nEnd\n";

    struct Assist {
        const char* kinds; int weapons; int has_ai; int guard; int clip; const char* slot;
        const char* from; const char* to; int contain; int turret;
    };
    const Assist assists[kAssist] = {
        {"STRUCTURE", 3, 1, 0, 4, "PRIMARY", "P304_Laser", "P304_Laser", 0, 0},
        {"STRUCTURE CAN_ATTACK", 3, 1, 0, -1, "SECONDARY", "P304_Plain", "P304_Laser", 0, 0},
        {"STRUCTURE", 2, 1, 0, 1, "SECONDARY", "Nothing_Here", "", 0, 0},
        {"STRUCTURE CAN_ATTACK", 0, 1, 0, 2, "PRIMARY", "P304_Laser", "P304_Plain", 0, 0},
        {"STRUCTURE CAN_ATTACK", 3, 0, 0, 3, "PRIMARY", "P304_Laser", "P304_Laser", 0, 0},
        {"INFANTRY", 1, 0, 1, 5, "PRIMARY", "", "P304_Laser", 0, 0},
        {"STRUCTURE PORTABLE_STRUCTURE", 3, 1, 0, 2, "TERTIARY", "", "", 0, 0},
        {"STRUCTURE SPAWNS_ARE_THE_WEAPONS", 3, 1, 0, 2, "PRIMARY", "", "", 0, 0},
        {"INFANTRY", 0, 1, 0, 2, "PRIMARY", "", "", 0, 0},
        // Fights only through the passengers in its own hold (a transport lets
        // only infantry fire, and asks about itself).
        {"INFANTRY", 3, 0, 0, 2, "PRIMARY", "", "", 2, 0},
        {"STRUCTURE", 3, 1, 0, 2, "PRIMARY", "", "", 0, 1},
        {"STRUCTURE", 3, 1, 0, 2, "SECONDARY", "P304_Laser", "", 0, 2},
        {"STRUCTURE", 2, 1, 0, 2, "SECONDARY", "", "", 0, 1},
        {"INFANTRY", 3, 1, 0, 7, "SECONDARY", "P304_Laser", "P304_Laser", 0, 0},
    };
    for (int v = 0; v < kAssist; ++v) {
        const Assist& a = assists[v];
        s << "Object P304_" << v << "\n  KindOf = PRELOAD " << a.kinds << '\n' << body() << geometry(8);
        if (a.guard) s << "  EnterGuard = Yes\n";
        if (a.weapons) {
            s << "  WeaponSet\n    Conditions = None\n";
            if (a.weapons & 1) s << "    Weapon = PRIMARY P_Weapon0\n";
            if (a.weapons & 2) s << "    Weapon = SECONDARY P_Weapon1\n";
            s << "  End\n";
        }
        if (a.has_ai) {
            std::string turret;
            // Turret 1 holds the primary weapon; turret 2 holds both.
            if (a.turret)
                turret = std::string("    Turret\n      ControlledWeaponSlots = ") +
                         (a.turret == 1 ? "PRIMARY" : "PRIMARY SECONDARY") + "\n    End\n";
            s << ai(turret);
        }
        if (a.contain == 2)
            s << "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 4\n"
              << "    PassengersAllowedToFire = Yes\n    AllowInsideKindOf = INFANTRY\n  End\n";
        s << "  Behavior = AssistedTargetingUpdate ModuleTag_Assist\n";
        if (a.clip >= 0) s << "    AssistingClipSize = " << a.clip << '\n';
        s << "    AssistingWeaponSlot = " << a.slot << '\n';
        if (*a.from) s << "    LaserFromAssisted = " << a.from << '\n';
        if (*a.to) s << "    LaserToTarget = " << a.to << '\n';
        s << "  End\nEnd\n";
    }
    s << "Object P304_Rider\n  KindOf = PRELOAD INFANTRY\n  TransportSlotCount = 1\n" << body()
      << geometry(2) << "End\n";
    return s.str();
}

// ---------------------------------------------------------------------------
// HordeUpdate
// ---------------------------------------------------------------------------

class SizeDraw : public DrawModule, public ObjectDrawInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(SizeDraw, "OracleLeafDraw", 64, 64)
public:
    SizeDraw(Thing* thing, const ModuleData* data) : DrawModule(thing, data) {}
    Real w = 0.0f, h = 0.0f;
    void setTerrainDecalSize(Real x, Real y) override { w = x; h = y; }

    NameKeyType getModuleNameKey() const override { return NAMEKEY("OracleLeafDraw"); }
    void doDrawModule(const Matrix3D*) override {}
    void setShadowsEnabled(Bool) override {}
    void releaseShadows() override {}
    void allocateShadows() override {}
    void setFullyObscuredByShroud(Bool) override {}
    void reactToTransformChange(const Matrix3D*, const Coord3D*, Real) override {}
    void reactToGeometryChange() override {}
    ObjectDrawInterface* getObjectDrawInterface() override { return this; }
    const ObjectDrawInterface* getObjectDrawInterface() const override { return this; }
    Bool clientOnly_getRenderObjInfo(Coord3D*, Real*, Matrix3D*) const override { return FALSE; }
    Bool clientOnly_getRenderObjBoundBox(OBBoxClass*) const override { return FALSE; }
    Bool clientOnly_getRenderObjBoneTransform(const AsciiString&, Matrix3D*) const override { return FALSE; }
    Int getPristineBonePositionsForConditionState(const ModelConditionFlags&, const char*, Int,
                                                  Coord3D*, Matrix3D*, Int) const override { return 0; }
    Int getCurrentBonePositions(const char*, Int, Coord3D*, Matrix3D*, Int) const override { return 0; }
    Bool getCurrentWorldspaceClientBonePositions(const char*, Matrix3D&) const override { return FALSE; }
    Bool getProjectileLaunchOffset(const ModelConditionFlags&, WeaponSlotType, Int, Matrix3D*,
                                   WhichTurretType, Coord3D*, Coord3D*) const override { return FALSE; }
    void updateProjectileClipStatus(UnsignedInt, UnsignedInt, WeaponSlotType) override {}
    void updateDrawModuleSupplyStatus(Int, Int) override {}
    void notifyDrawModuleDependencyCleared() override {}
    void setHidden(Bool) override {}
    void replaceModelConditionState(const ModelConditionFlags&) override {}
    void replaceIndicatorColor(Color) override {}
    Bool handleWeaponFireFX(WeaponSlotType, Int, const FXList*, Real, const Coord3D*, Real) override { return FALSE; }
    Int getBarrelCount(WeaponSlotType) const override { return 0; }
    void setSelectable(Bool) override {}
    void setAnimationLoopDuration(UnsignedInt) override {}
    void setAnimationCompletionTime(UnsignedInt) override {}
    Bool updateBonesForClientParticleSystems() override { return FALSE; }
    void setAnimationFrame(int) override {}
    void setPauseAnimation(Bool) override {}
    void updateSubObjects() override {}
    void showSubObject(const AsciiString&, Bool) override {}

protected:
    void crc(Xfer*) override {}
    void xfer(Xfer*) override {}
    void loadPostProcess() override {}
};
SizeDraw::~SizeDraw() {}

class SizeDrawData : public ModuleData {};

struct HordeUnit {
    Object* o = nullptr;
    HordeUpdate* horde = nullptr;
    SizeDraw* draw = nullptr;
    Module** saved = nullptr;
    Module* fake[2] = {nullptr, nullptr};
};

static Team* team_of(oracle_wp51::World& w, long t) {
    return t == 0 ? w.team_neutral : (t == 1 ? w.team_a : w.team_b);
}

static Coord3D at(long x, long y, long z = 0) {
    return Coord3D{static_cast<Real>(x), static_cast<Real>(y), static_cast<Real>(z)};
}

static std::string horde_row(const std::vector<HordeUnit>& units) {
    std::ostringstream out;
    out << GetGameLogicRandomSeedCRC();
    for (const HordeUnit& u : units) {
        out << " |";
        if (u.horde) {
            out << ' ' << module_state(u.horde) << ' ' << static_cast<int>(u.horde->isInHorde())
                << static_cast<int>(u.horde->isTrueHordeMember());
        } else {
            out << " -";
        }
        uint32_t bonus = 0;
        for (int b = 0; b < 32; ++b)
            if (u.o->testWeaponBonusCondition(static_cast<WeaponBonusConditionType>(b))) bonus |= 1u << b;
        Drawable* d = u.o->getDrawable();
        out << ' ' << bonus << ' ' << static_cast<int>(d->getTerrainDecalType()) << ' '
            << bits(u.draw->w) << ' ' << bits(u.draw->h) << ' ' << bits(d->*access_member(FadeTargetTag()))
            << ' ' << bits(d->*access_member(FadeRateTag()));
    }
    return out.str();
}

static void horde_vector(oracle_wp51::World& w, Vector& v) {
    static SizeDrawData draw_data;
    const UnsignedInt seed = 1 + pick(100000);
    const UnsignedInt start = pick(5) == 0 ? 0x01000000u + pick(100000) : 1 + pick(200);
    const long relation = pick(3);  // between teams A and B
    const long upgrades = pick(4) == 0 ? 0 : pick(16);  // A nationalism, A fanaticism, B, B
    w.set_frame(start);
    Player* pa = w.team_a->getControllingPlayer();
    Player* pb = w.team_b->getControllingPlayer();
    const Relationship rel = relation == 0 ? ENEMIES : (relation == 1 ? NEUTRAL : ALLIES);
    pa->setPlayerRelationship(pb, rel);
    pb->setPlayerRelationship(pa, rel);
    const UpgradeTemplate* nat = TheUpgradeCenter->findUpgrade("Upgrade_Nationalism");
    const UpgradeTemplate* fan = TheUpgradeCenter->findUpgrade("Upgrade_Fanaticism");
    if (!nat || !fan) die("upgrades");
    auto set_upgrade = [&](long which, bool on) {
        Player* p = which < 2 ? pa : pb;
        const UpgradeTemplate* t = (which & 1) ? fan : nat;
        if (on) p->addUpgrade(t, UPGRADE_STATUS_COMPLETE);
        else p->removeUpgrade(t);
    };
    for (int k = 0; k < 4; ++k) set_upgrade(k, (upgrades >> k) & 1);
    TheGameLogic->setDrawIconUI(TRUE);

    const ObjectID base = g_next_probe_base(w);
    InitRandom(seed);
    // A cluster, sometimes straddling the map edge.
    const long cx = pick(5) == 0 ? 0 : 200;
    const bool line = pick(6) == 0;
    const int line_variant = pick(2) ? 3 : 0;
    const long gap = (line_variant ? 30 : 20) + 5 * (static_cast<long>(pick(3)) - 1);
    // Out of the probe's reach, within the true member's.
    const long far = (line_variant ? 63 : 53);
    const long line_x[4] = {200, 200 + gap, 200 + far, 200 + far + 2};
    const int count = line ? 4 : 2 + static_cast<int>(pick(7));
    const int main_variant = static_cast<int>(pick(kHordeVariants));
    const long main_team = pick(3);
    std::vector<HordeUnit> units(count);
    for (int i = 0; i < count; ++i) {
        long variant = pick(4) == 0 ? pick(kHordeVariants) : main_variant;
        if (pick(10) == 0) variant = -1;
        const long has_ai = variant >= 0 && pick(6) != 0;
        const long team = pick(4) == 0 ? pick(3) : main_team;
        const long x = cx - 30 + 5 * static_cast<long>(pick(13));
        const long y = 200 - 20 + 5 * static_cast<long>(pick(9));
        const long z = pick(6) == 0 ? 10 * static_cast<long>(pick(4)) : 0;
        if (line) {
            variant = line_variant;
            v.units.push_back({variant, 1, main_team, line_x[i], 200, 0});
        } else {
            v.units.push_back({variant, has_ai, team, x, y, z});
        }
        const std::vector<long>& spec = v.units.back();
        const std::string name = variant < 0
                                     ? std::string("P336_Outsider")
                                     : "P336_" + std::to_string(variant) + "_" + std::to_string(spec[1]);
        HordeUnit& u = units[i];
        u.o = spawn(w, name, team_of(w, spec[2]), at(spec[3], spec[4], spec[5]));
        u.horde = find_behavior<HordeUpdate>(u.o);
        if ((variant >= 0) != (u.horde != nullptr)) die("horde module");
        Drawable* d = u.o->getDrawable();
        if (!d) die("no drawable");
        u.draw = newInstance(SizeDraw)(d, &draw_data);
        u.fake[0] = u.draw;
        u.saved = (d->*access_member(DrawArrayTag()))[0];
        (d->*access_member(DrawArrayTag()))[0] = u.fake;
    }
    ThePartitionManager->update();

    v.setup = {static_cast<long>(seed), static_cast<long>(start), relation, upgrades,
               static_cast<long>(base)};
    v.rows.push_back(horde_row(units));
    UnsignedInt frame = start;
    const int ops = 4 + static_cast<int>(pick(10));
    for (int k = 0; k < ops; ++k) {
        long code = pick(10);
        long a = pick(count), b = 0, c = 0;
        if (line && (code == 4 || code == 5)) code = 0;  // the line stays put
        if (code <= 3) {  // frames
            code = 0;
            a = 1 + pick(pick(3) == 0 ? 60 : 12);
        } else if (code == 4 || code == 5) {  // move
            code = 1;
            b = cx - 30 + 5 * static_cast<long>(pick(13));
            c = 200 - 20 + 5 * static_cast<long>(pick(9));
        } else if (code == 6) {  // toggle dead
            code = 2;
        } else if (code == 7) {  // toggle an upgrade
            code = 3;
            a = pick(4);
            b = pick(2);
        } else if (code == 8) {  // icon UI
            code = 4;
            a = pick(2);
        } else {  // direct update
            code = 5;
            if (!units[a].horde) code = 0, a = 1;
        }
        long ret = 0;
        w.set_frame(frame);
        if (code == 0) {
            for (long t = 0; t < a; ++t) {
                w.set_frame(++frame);
                for (HordeUnit& u : units)
                    if (u.horde) step(u.horde, frame);
            }
        } else if (code == 1) {
            const Coord3D p = at(b, c, static_cast<long>(units[a].o->getPosition()->z));
            units[a].o->setPosition(&p);
            ThePartitionManager->update();
        } else if (code == 2) {
            units[a].o->setEffectivelyDead(!units[a].o->isEffectivelyDead());
        } else if (code == 3) {
            set_upgrade(a, b);
        } else if (code == 4) {
            TheGameLogic->setDrawIconUI(a != 0);
        } else {
            ret = static_cast<long>(units[a].horde->update());
        }
        v.ops.push_back({code, a, b, c});
        v.rows.push_back(horde_row(units) + " = " + std::to_string(ret));
    }
    for (HordeUnit& u : units) {
        (u.o->getDrawable()->*access_member(DrawArrayTag()))[0] = u.saved;
        deleteInstance(u.draw);
    }
    for (int k = 0; k < 4; ++k) set_upgrade(k, false);
    pa->setPlayerRelationship(pb, ENEMIES);
    pb->setPlayerRelationship(pa, ENEMIES);
    TheGameLogic->setDrawIconUI(TRUE);
    clear_world(w);
}

// ---------------------------------------------------------------------------
// AutoFindHealingUpdate
// ---------------------------------------------------------------------------

static void healing_vector(oracle_wp51::World& w, Vector& v) {
    const long variant = pick(kHealers);
    const long human = pick(5) == 0;
    const UnsignedInt start = 1 + pick(200);
    w.set_frame(start);
    Player* pa = w.team_a->getControllingPlayer();
    pa->setPlayerType(human ? PLAYER_HUMAN : PLAYER_COMPUTER, FALSE);
    const ObjectID base = g_next_probe_base(w);
    InitRandom(7);
    const long x = 300 + 5 * static_cast<long>(pick(5));
    const long y = 300 + 5 * static_cast<long>(pick(5));
    // units[0] is the seeker; the rest are pads and non-pads around it.
    const int pads = static_cast<int>(pick(6));
    std::vector<Object*> objs;
    std::vector<std::vector<long>> places;
    for (int i = 0; i < pads; ++i) {
        long dx = 25 * (static_cast<long>(pick(9)) - 4);
        long dy = 25 * (static_cast<long>(pick(9)) - 4);
        if (i && pick(3) == 0) {  // a tie with an earlier one, mirrored
            dx = -(places[0][1] - x);
            dy = places[0][2] - y;
        }
        const long is_pad = pick(5) != 0;
        places.push_back({is_pad, x + dx, y + dy, static_cast<long>(pick(3))});
    }
    const long seeker_first = pick(2);
    Object* seeker = nullptr;
    auto make_seeker = [&]() {
        seeker = spawn(w, "P306_" + std::to_string(variant), w.team_a, at(x, y));
    };
    if (seeker_first) make_seeker();
    for (int i = 0; i < pads; ++i) {
        objs.push_back(spawn(w, places[i][0] ? "P306_Pad" : "P306_Other", team_of(w, places[i][3]),
                             at(places[i][1], places[i][2], 5 * i)));
        v.units.push_back(places[i]);
    }
    if (!seeker_first) make_seeker();
    ThePartitionManager->update();
    AutoFindHealingUpdate* m = find_behavior<AutoFindHealingUpdate>(seeker);
    if (!m) die("healing module");

    auto row = [&](long ret) {
        std::ostringstream out;
        out << module_state(m) << ' ' << ret;
        BodyModuleInterface* b = seeker->getBodyModule();
        out << ' ' << (b ? bits(b->getHealth()) : 0u);
        out << g_orders.str();
        g_orders.str("");
        return out.str();
    };
    g_orders.str("");
    v.setup = {variant, human, static_cast<long>(start), static_cast<long>(base), x, y, seeker_first};
    v.rows.push_back(row(0));
    UnsignedInt frame = start;
    long is_human = human;
    const int ops = 5 + static_cast<int>(pick(14));
    for (int k = 0; k < ops; ++k) {
        long code = pick(10);
        long a = 0, b = 0, c = 0;
        if (k == 0 && pick(3) != 0) code = 4;
        if (code <= 3) {  // frames
            code = 0;
            a = 1 + pick(8);
        } else if (code <= 5) {  // change health, in quarter points
            code = 1;
            const long choices[] = {-200, -20, -19, -21, -201, -199, -1, 4, 400, -300, -100, 100};
            a = choices[pick(12)];
        } else if (code == 6) {  // busy or idle
            code = 2;
            a = pick(3) == 0;
        } else if (code == 7) {  // human or computer
            code = 3;
            a = pick(4) == 0;
        } else if (code == 8 && pads) {  // move a pad
            code = 4;
            a = pick(pads);
            b = x + 25 * (static_cast<long>(pick(9)) - 4);
            c = y + 25 * (static_cast<long>(pick(9)) - 4);
        } else {  // direct update
            code = 5;
        }
        long ret = 0;
        w.set_frame(frame);
        if (code == 0) {
            for (long t = 0; t < a; ++t) {
                w.set_frame(++frame);
                step(m, frame);
            }
        } else if (code == 1) {
            if (seeker->getBodyModule())
                seeker->getBodyModule()->internalChangeHealth(static_cast<Real>(a) * 0.25f);
        } else if (code == 2) {
            if (a) g_busy.insert(seeker->getID());
            else g_busy.erase(seeker->getID());
        } else if (code == 3) {
            is_human = a;
            pa->setPlayerType(is_human ? PLAYER_HUMAN : PLAYER_COMPUTER, FALSE);
        } else if (code == 4) {
            const Coord3D p = at(b, c, 5 * a);
            objs[a]->setPosition(&p);
            ThePartitionManager->update();
        } else {
            ret = static_cast<long>(m->update());
        }
        v.ops.push_back({code, a, b, c});
        v.rows.push_back(row(ret));
    }
    pa->setPlayerType(PLAYER_COMPUTER, FALSE);
    clear_world(w);
}

// ---------------------------------------------------------------------------
// SpectreGunshipDeploymentUpdate
// ---------------------------------------------------------------------------

static std::string selection_of(int player) {
    Player* p = ThePlayerList->getNthPlayer(player);
    AIGroupPtr group = TheAI->createGroup();
#if RETAIL_COMPATIBLE_AIGROUP
    p->getCurrentSelectionAsAIGroup(group);
    const VecObjectID ids = group->getAllIDs();
    TheAI->destroyGroup(group);
#else
    p->getCurrentSelectionAsAIGroup(group.Peek());
    const VecObjectID ids = group->getAllIDs();
    group->removeAll();
#endif
    std::ostringstream out;
    for (ObjectID id : ids) out << static_cast<int>(id) << ',';
    return out.str().empty() ? std::string("-") : out.str();
}

static void spectre_vector(oracle_wp51::World& w, Vector& v) {
    const long variant = pick(kSpectres);
    const UnsignedInt start = 1 + pick(5000);
    const long team = pick(4) == 0 ? 2 : 1;
    // On a diagonal or a midline half of the time, where the edges tie.
    auto coordinate = [&]() -> long {
        const long marks[] = {0, 100, 250, 499, 500, 501, 750, 900, 1000};
        return pick(2) ? marks[pick(9)] : static_cast<long>(pick(1100)) - 50;
    };
    long sx = coordinate(), sy = pick(4) == 0 ? sx : coordinate();
    if (sx < 20 || sx > 980) sx = 500;
    if (sy < 20 || sy > 980) sy = 300;
    w.set_frame(start);
    const ObjectID base = g_next_probe_base(w);
    InitRandom(11);
    Object* source = spawn(w, "P370_Source" + std::to_string(variant), team_of(w, team), at(sx, sy, 7));
    ThePartitionManager->update();
    SpectreGunshipDeploymentUpdate* m = find_behavior<SpectreGunshipDeploymentUpdate>(source);
    if (!m) die("spectre module");
    std::vector<SpecialPowerModule*> powers = spms(source);

    Int last_next_id = 0;
    auto row = [&](long ret) {
        std::ostringstream out;
        out << GetGameLogicRandomSeedCRC() << ' ' << module_state(m) << ' ' << ret << ' '
            << m->getDisabledTypesToProcess().count();
        for (int d = 0; d < DISABLED_COUNT; ++d)
            out << static_cast<int>(m->getDisabledTypesToProcess().test(d));
        for (SpecialPowerModule* p : powers)
            out << ' ' << oracle_wp51::World::snapshot_save_hex(p);
        // Every object made since the source, in creation order.
        std::vector<Object*> all = live_objects();
        std::map<int, Object*> by_id;
        for (Object* o : all) by_id[static_cast<int>(o->getID())] = o;
        for (auto& it : by_id) {
            Object* o = it.second;
            if (o == source) continue;
            out << " G " << it.first << ' ' << static_cast<int>(o->getProducerID()) << ' '
                << static_cast<int>(o->getControllingPlayer()->getPlayerIndex());
            put(out, *o->getPosition());
            out << ' ' << bits(o->getOrientation());
            for (SpecialPowerModule* p : spms(o))
                out << ' ' << oracle_wp51::World::snapshot_save_hex(p);
        }
        for (int p = 0; p < 3; ++p) out << ' ' << selection_of(p);
        return out.str();
    };
    (void)last_next_id;
    for (int p = 0; p < 3; ++p) ThePlayerList->getNthPlayer(p)->setCurrentlySelectedAIGroup(nullptr);

    v.setup = {variant, static_cast<long>(start), static_cast<long>(base), team, sx, sy};
    v.rows.push_back(row(0));
    UnsignedInt frame = start;
    const int ops = 2 + static_cast<int>(pick(6));
    for (int k = 0; k < ops; ++k) {
        long code = pick(10);
        long a = pick(powers.size()), b = 0, c = 0, d = 0;
        if (k == 0 && pick(4) != 0) code = 0;
        if (code <= 4) {  // fire at a location
            code = 0;
            b = coordinate();
            c = pick(3) == 0 ? b : coordinate();
            if (pick(12) == 0) b = sx, c = sy;
            d = pick(3) == 0 ? 0x00040000 : (pick(3) == 0 ? 0x00040001 : (pick(2) ? 0 : 0x20));
        } else if (code == 5) {  // frames
            code = 1;
            a = 1 + pick(120);
        } else if (code == 6) {  // pause or unpause
            code = 2;
            b = pick(2);
        } else if (code == 7) {  // status of the source
            code = 3;
            a = pick(4);  // sold, under construction, dead, hacked
            b = pick(2);
        } else {  // direct update
            code = 4;
        }
        long ret = 0;
        w.set_frame(frame);
        if (code == 0) {
            const Coord3D target = at(b, c, 3);
            powers[a]->doSpecialPowerAtLocation(&target, INVALID_ANGLE, static_cast<UnsignedInt>(d));
            ThePartitionManager->update();
        } else if (code == 1) {
            for (long t = 0; t < a; ++t) {
                w.set_frame(++frame);
                step(m, frame);
            }
        } else if (code == 2) {
            powers[a]->pauseCountdown(b != 0);
        } else if (code == 3) {
            if (a == 0) {
                if (b) source->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_SOLD));
                else source->clearStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_SOLD));
            } else if (a == 1) {
                if (b) source->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
                else source->clearStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
            } else if (a == 2) {
                source->setEffectivelyDead(b != 0);
            } else {
                if (b) source->setDisabled(DISABLED_HACKED);
                else source->clearDisabled(DISABLED_HACKED);
            }
        } else {
            ret = static_cast<long>(m->update());
        }
        v.ops.push_back({code, a, b, c, d});
        v.rows.push_back(row(ret));
    }
    for (int p = 0; p < 3; ++p) ThePlayerList->getNthPlayer(p)->setCurrentlySelectedAIGroup(nullptr);
    clear_world(w);
}

// ---------------------------------------------------------------------------
// AssistedTargetingUpdate
// ---------------------------------------------------------------------------

static void assist_vector(oracle_wp51::World& w, Vector& v) {
    const long variant = pick(kAssist);
    const UnsignedInt start = 1 + pick(5000);
    const long container = pick(5) == 0 ? 1 + pick(2) : 0;
    const long can_contain = variant == 8 || variant == 5 || variant == 13;
    w.set_frame(start);
    const ObjectID base = g_next_probe_base(w);
    InitRandom(13);
    Object* me = spawn(w, "P304_" + std::to_string(variant), w.team_a, at(400, 400, 4));
    Object* requester = spawn(w, "P304_Victim", w.team_a, at(450, 410, 9));
    Object* victim = spawn(w, "P304_Victim", w.team_b, at(520, 470, 2));
    Object* bunker = nullptr;
    if (container && can_contain) {
        bunker = spawn(w, "P304_Bunker" + std::to_string(container - 1), w.team_a, at(600, 600));
        bunker->getContain()->addToContain(me);
    }
    AssistedTargetingUpdate* m = find_behavior<AssistedTargetingUpdate>(me);
    if (!m) die("assist module");
    std::set<int> mine = {static_cast<int>(me->getID()), static_cast<int>(requester->getID()),
                          static_cast<int>(victim->getID())};
    if (bunker) mine.insert(static_cast<int>(bunker->getID()));
    std::vector<Object*> riders;

    auto row = [&](long ret) {
        std::ostringstream out;
        out << GetGameLogicRandomSeedCRC() << ' ' << module_state(m) << ' ' << ret;
        WeaponSlotType cur = PRIMARY_WEAPON;
        const bool any = me->getCurrentWeapon(&cur) != nullptr;
        out << ' ' << static_cast<int>(any) << static_cast<int>(cur) << static_cast<int>(me->isCurWeaponLocked() != FALSE);
        for (int s = 0; s < 3; ++s) {
            Weapon* weapon = me->getWeaponInWeaponSlot(static_cast<WeaponSlotType>(s));
            if (!weapon) {
                out << " -";
                continue;
            }
            out << ' ' << static_cast<int>(weapon->*access_member(WStatusTag())) << ':'
                << weapon->*access_member(WAmmoTag());
        }
        out << g_orders.str();
        g_orders.str("");
        std::map<int, Object*> made;
        for (Object* o : live_objects()) {
            const int id = static_cast<int>(o->getID());
            if (!mine.count(id) && !o->isKindOf(KINDOF_INFANTRY)) made[id] = o;
        }
        for (auto& it : made) {
            Object* o = it.second;
            out << " L " << it.first << ' '
                << static_cast<int>(o->getControllingPlayer()->getPlayerIndex());
            put(out, *o->getPosition());
            static const NameKeyType key = NAMEKEY("LaserUpdate");
            LaserUpdate* laser = static_cast<LaserUpdate*>(o->getDrawable()->findClientUpdateModule(key));
            if (laser) {
                // The client update takes its start from the parent's drawable, not
                // from `from`, so only the parent and the end are comparable.
                Drawable* parent = TheGameClient->findDrawableByID(laser->*access_member(LaserParentTag()));
                out << ' ' << (parent && parent->getObject() ? static_cast<int>(parent->getObject()->getID()) : 0);
                put(out, *laser->getEndPos());
            } else {
                out << " -";
            }
        }
        // Lasers are looked at once, then removed.
        for (auto& it : made) TheGameLogic->destroyObject(it.second);
        w.flush_destroy_list();
        return out.str();
    };

    g_orders.str("");
    v.setup = {variant, static_cast<long>(start), static_cast<long>(base), bunker ? container : 0};
    v.rows.push_back(row(0));
    UnsignedInt frame = start;
    const int ops = 3 + static_cast<int>(pick(10));
    for (int k = 0; k < ops; ++k) {
        long code = pick(12);
        long a = 0, b = 0, c = 0, d = 0, e = 0;
        if (code <= 3) {  // ask: attack if free
            code = 0;
        } else if (code == 4) {  // assist without asking
            code = 1;
        } else if (code <= 6) {  // stage a weapon
            code = 2;
            a = pick(3);
            b = pick(4);                    // status
            c = pick(3);                    // ammo
            d = static_cast<long>(pick(7)) - 3;  // when it can fire again, from now
            e = pick(3) == 0 ? static_cast<long>(pick(5)) - 2 : -100;  // pre-attack end
        } else if (code == 7) {  // a status bit
            code = 3;
            a = pick(7);  // no attack, under construction, sold, can attack, subdued, hacked, emp
            b = pick(3) != 0;
        } else if (code == 8) {  // frames
            code = 4;
            a = 1 + pick(5);
        } else if (code == 9) {  // turret on or off
            code = 5;
            a = pick(2);
        } else if (code == 10) {  // a passenger in or out of my own hold
            code = 6;
            a = pick(2);
        } else {  // direct update
            code = 7;
        }
        long ret = 0;
        w.set_frame(frame);
        if (code == 0) {
            ret = m->isFreeToAssist();
            if (ret) m->assistAttack(requester, victim);
        } else if (code == 1) {
            m->assistAttack(requester, victim);
        } else if (code == 2) {
            Weapon* weapon = me->getWeaponInWeaponSlot(static_cast<WeaponSlotType>(a));
            if (weapon) {
                weapon->*access_member(WStatusTag()) = static_cast<WeaponStatus>(b);
                weapon->*access_member(WAmmoTag()) = static_cast<UnsignedInt>(c);
                weapon->*access_member(WAgainTag()) = static_cast<UnsignedInt>(static_cast<long>(frame) + d);
                weapon->*access_member(WPreTag()) =
                    e == -100 ? 0u : static_cast<UnsignedInt>(static_cast<long>(frame) + e);
            }
        } else if (code == 3) {
            const ObjectStatusTypes status[4] = {OBJECT_STATUS_NO_ATTACK, OBJECT_STATUS_UNDER_CONSTRUCTION,
                                                 OBJECT_STATUS_SOLD, OBJECT_STATUS_CAN_ATTACK};
            const DisabledType disabled[3] = {DISABLED_SUBDUED, DISABLED_HACKED, DISABLED_EMP};
            if (a < 4) {
                if (b) me->setStatus(MAKE_OBJECT_STATUS_MASK(status[a]));
                else me->clearStatus(MAKE_OBJECT_STATUS_MASK(status[a]));
            } else {
                if (b) me->setDisabled(disabled[a - 4]);
                else me->clearDisabled(disabled[a - 4]);
            }
        } else if (code == 4) {
            for (long t = 0; t < a; ++t) {
                w.set_frame(++frame);
                step(m, frame);
            }
        } else if (code == 5) {
            if (me->getAI()) me->getAI()->setTurretEnabled(TURRET_MAIN, a != 0);
        } else if (code == 6) {
            ContainModuleInterface* hold = me->getContain();
            if (hold && variant == 9) {
                if (a && riders.size() < 3) {
                    Object* rider = spawn(w, "P304_Rider", w.team_a, at(400, 400));
                    mine.insert(static_cast<int>(rider->getID()));
                    hold->addToContain(rider);
                    riders.push_back(rider);
                } else if (!a && !riders.empty()) {
                    hold->removeFromContain(riders.back());
                    riders.pop_back();
                }
            }
        } else {
            ret = static_cast<long>(m->update());
        }
        v.ops.push_back({code, a, b, c, d, e});
        v.rows.push_back(row(ret));
    }
    if (bunker) bunker->getContain()->removeAllContained(FALSE);
    if (me->getContain()) me->getContain()->removeAllContained(FALSE);
    clear_world(w);
}

static void edge_vector(Vector& v) {
    static SlopeTerrain* terrain = NEW SlopeTerrain;
    auto coordinate = [&]() -> long { return 50 * (static_cast<long>(pick(9)) - 4); };
    const long lx = pick(3) == 0 ? 0 : coordinate();
    const long ly = pick(3) == 0 ? 0 : coordinate();
    const long hx = lx + 50 * (1 + static_cast<long>(pick(8)));
    const long hy = pick(3) == 0 ? ly + (hx - lx) : ly + 50 * (1 + static_cast<long>(pick(8)));
    terrain->extent.lo = {static_cast<Real>(lx), static_cast<Real>(ly), 0.0f};
    terrain->extent.hi = {static_cast<Real>(hx), static_cast<Real>(hy), 100.0f};
    v.setup = {lx, ly, hx, hy};
    std::ostringstream first;
    first << '-';
    v.rows.push_back(first.str());
    for (int k = 0; k < 12; ++k) {
        long x = pick(3) == 0 ? (hx - lx) / 2 : coordinate();
        long y = pick(3) == 0 ? (hy - ly) / 2 : coordinate();
        const Coord3D p = at(x, y, 5);
        std::ostringstream out;
        put(out, terrain->findClosestEdgePoint(&p));
        put(out, terrain->findFarthestEdgePoint(&p));
        v.ops.push_back({x, y});
        v.rows.push_back(out.str());
    }
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_pkt336_370_306_304_leaves <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheTerrainLogic = NEW SlopeTerrain;
    TheTerrainLogic->init();
    (TheModuleFactory->*(&ExposedFactory::addModuleInternal))(
        make_rec_ai, AIUpdateInterface::friend_newModuleData, MODULETYPE_BEHAVIOR,
        "OracleLeafAI", AIUpdateInterface::getInterfaceMask());
    w.load_ini(fixture_ini());
    w.init_with_neutral_player();

    FILE* out = std::fopen(argv[1], "w");
    if (!out) return 1;
    const int kWorld = 1600;
    const int kTotal = kWorld + 200;
    std::fprintf(out, "{\"vector_count\":%d,\"vectors\":[", kTotal);
    for (int i = 0; i < kTotal; ++i) {
        const int kind = i < kWorld ? i % 4 : 4;
        Vector v;
        if (kind == 4) edge_vector(v);
        else if (kind == 0) horde_vector(w, v);
        else if (kind == 1) spectre_vector(w, v);
        else if (kind == 2) healing_vector(w, v);
        else assist_vector(w, v);
        std::fprintf(out, "%s\n{\"seed\":%d,\"kind\":%d,\"setup\":%s,\"units\":\"%s\",\"ops\":\"%s\",\"rows\":\"",
                     i ? "," : "", i, kind, ints(v.setup).c_str(), table(v.units).c_str(),
                     table(v.ops).c_str());
        for (size_t r = 0; r < v.rows.size(); ++r)
            std::fprintf(out, "%s%s", r ? "#" : "", v.rows[r].c_str());
        std::fprintf(out, "\"}");
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return 0;
}
