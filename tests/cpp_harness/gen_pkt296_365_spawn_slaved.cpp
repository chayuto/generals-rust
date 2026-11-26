// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for SpawnBehavior (PKT-296) and SlavedUpdate (PKT-365), run on
// the unmodified reference modules in a small real world, with the real
// production exits, bodies (HiveStructureBody included), ProneUpdate,
// StealthUpdate and DestroyDie around them.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt296_365_spawn_slaved.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT296_365_SpawnSlaved.json
//
// The AI is a subclass that records the orders; only aiGoProne is carried out, by
// the real privateGoProne. Update modules are called directly, in object id order,
// while the object is not disabled. A row lists the objects whose state changed,
// the removed objects, the AI orders, the answers to queries and a random probe.
// The fixture templates are carried in the file ('~' is a line break). The
// reference is built with RETAIL_COMPATIBLE_XFER_SAVE, so SpawnBehavior saves
// version 2; its burst countdown is a column of its own.
//
// Not covered: scoreTheKill (no playable side in this world); the group selection
// of computeAggregateStates (nothing is selected); welding particle systems (no
// templates); loading a save; slaves that are not drones being disabled as
// unmanned; a master without an AI under a scouting slave and an unknown spawn
// template (the reference crashes or throws); disabled states that expire.

#include "oracle_wp51_world.h"

#include "Common/Module.h"
#include "Common/ModuleFactory.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "GameLogic/LogicRandomValue.h"
#include "Common/ScoreKeeper.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "GameClient/Drawable.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/SlavedUpdate.h"
#include "GameLogic/Module/SpawnBehavior.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/WeaponSet.h"

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <sstream>
#include <string>
#include <vector>

// --- member access ----------------------------------------------------------

struct PathTag { using type = Path* AIUpdateInterface::*; };
template struct MemberAccess<PathTag, &AIUpdateInterface::m_path>;
PathTag::type access_member(PathTag);

struct LocoFlagsTag { using type = UnsignedInt Locomotor::*; };
template struct MemberAccess<LocoFlagsTag, &Locomotor::m_flags>;
LocoFlagsTag::type access_member(LocoFlagsTag);

struct HealthBoxTag { using type = Coord3D Object::*; };
template struct MemberAccess<HealthBoxTag, &Object::m_healthBoxOffset>;
HealthBoxTag::type access_member(HealthBoxTag);

struct BurstTag { using type = UnsignedInt SpawnBehavior::*; };
template struct MemberAccess<BurstTag, &SpawnBehavior::m_initialBurstCountdown>;
BurstTag::type access_member(BurstTag);

struct ExposedFactory : public ModuleFactory { using ModuleFactory::addModuleInternal; };

static void die(const char* what) {
    std::fprintf(stderr, "oracle: %s\n", what);
    std::exit(1);
}

static uint32_t bits(float f) {
    uint32_t u;
    std::memcpy(&u, &f, 4);
    return u;
}
static float real_of(uint32_t u) {
    float f;
    std::memcpy(&f, &u, 4);
    return f;
}

// --- recording AI -----------------------------------------------------------

static std::vector<std::string> g_commands;

class RecAI : public AIUpdateInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(RecAI, "OracleSpawnRecAI", 64, 64)
public:
    Bool m_idle = TRUE;
    CommandSourceType m_source = CMD_FROM_AI;

    RecAI(Thing* thing, const ModuleData* data) : AIUpdateInterface(thing, data) {}
    Bool isIdle() const override { return m_idle; }
    CommandSourceType getLastCommandSource() const override { return m_source; }
    void aiDoCommand(const AICommandParms* p) override {
        std::ostringstream s;
        s << static_cast<int>(getObject()->getID()) << ' ';
        switch (p->m_cmd) {
        case AICMD_IDLE:
            s << "idle " << static_cast<int>(p->m_cmdSource);
            break;
        case AICMD_MOVE_TO_POSITION:
            s << "move " << static_cast<int>(p->m_cmdSource) << ' ' << bits(p->m_pos.x) << ' '
              << bits(p->m_pos.y) << ' ' << bits(p->m_pos.z);
            break;
        case AICMD_FORCE_ATTACK_OBJECT:
            s << "force " << static_cast<int>(p->m_cmdSource) << ' '
              << static_cast<int>(p->m_obj->getID()) << ' ' << p->m_intValue;
            break;
        case AICMD_ATTACK_POSITION:
            s << "attackpos " << static_cast<int>(p->m_cmdSource) << ' ' << bits(p->m_pos.x) << ' '
              << bits(p->m_pos.y) << ' ' << bits(p->m_pos.z) << ' ' << p->m_intValue;
            break;
        case AICMD_GO_PRONE:
            s << "prone";
            AIUpdateInterface::aiDoCommand(p);
            break;
        case AICMD_FOLLOW_EXITPRODUCTION_PATH:
            return;  // the production exits have their own vectors
        default:
            die("unexpected AI command");
        }
        g_commands.push_back(s.str());
    }
};
RecAI::~RecAI() {}

static Module* make_rec_ai(Thing* t, const ModuleData* d) { return newInstance(RecAI)(t, d); }

static RecAI* rec_ai(Object* o) { return static_cast<RecAI*>(o->getAIUpdateInterface()); }

// --- terrain ----------------------------------------------------------------

static bool g_slope = false;

class SlopeTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return g_slope ? x * 0.01f + y * 0.02f : 0.0f;
    }
    Real getLayerHeight(Real x, Real y, PathfindLayerEnum, Coord3D* normal = nullptr,
                        Bool = true) const override {
        return getGroundHeight(x, y, normal);
    }
};

// --- random -----------------------------------------------------------------

static uint32_t g_state = 0x296365a1u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
static float real_in(float lo, float hi) {
    return lo + (hi - lo) * (static_cast<float>(pick(1000003)) / 1000003.0f);
}

// --- fixture ----------------------------------------------------------------

static const char* const kLocomotors =
    "Locomotor P_Drone\n  Surfaces = GROUND AIR\n  Speed = 60\n  TurnRate = 180\n"
    "  Acceleration = 60\n  Braking = 60\n  MinSpeed = 0\n  Lift = 100\nEnd\n"
    "Locomotor P_Panic\n  Surfaces = GROUND AIR\n  Speed = 90\n  TurnRate = 300\n"
    "  Acceleration = 90\n  Braking = 90\n  MinSpeed = 0\n  Lift = 100\nEnd\n";

static std::string object_head(const std::string& name, const std::string& kind_of, int radius,
                               int height) {
    std::ostringstream s;
    s << "Object " << name << "\n  KindOf = " << kind_of << "\n  Geometry = CYLINDER\n"
      << "  GeometryMajorRadius = " << radius << "\n  GeometryHeight = " << height
      << "\n  GeometryIsSmall = No\n";
    return s.str();
}

static std::string active_body(const char* module, int health, const std::string& extra = "") {
    std::ostringstream s;
    s << "  Body = " << module << " ModuleTag_Body\n    MaxHealth = " << health
      << "\n    InitialHealth = " << health << "\n" << extra << "  End\n";
    return s.str();
}

struct SlaveSpec {
    const char* fields;  // nullptr: no SlavedUpdate
    int ai;              // 0 none, 1 AI with locomotors, 2 AI without
    bool prone;
    int stealth;         // 0 none, 1 not innate, 2 innate
    bool destroy_die;
    int radius;
    int health;
    const char* kind_of;
};

static const char* const kGuard =
    "    GuardMaxRange = 35\n    GuardWanderRange = 10\n";
static const char* const kAttack =
    "    GuardMaxRange = 35\n    AttackRange = 75\n    AttackWanderRange = 20\n"
    "    DistToTargetToGrantRangeBonus = 100\n";
static const char* const kScout =
    "    GuardMaxRange = 40\n    ScoutRange = 60\n    ScoutWanderRange = 15\n";
static const char* const kRepair =
    "    GuardMaxRange = 30\n    RepairRange = 8\n    RepairMinAltitude = 15\n"
    "    RepairMaxAltitude = 25\n    RepairRatePerSecond = 90\n"
    "    RepairWhenBelowHealth% = 60\n    RepairMinReadyTime = 100\n"
    "    RepairMaxReadyTime = 300\n    RepairMinWeldTime = 200\n"
    "    RepairMaxWeldTime = 500\n    RepairWeldingSys = WeldSparks\n"
    "    RepairWeldingFXBone = Muzzle\n    StayOnSameLayerAsMaster = Yes\n";
static const char* const kAll =
    "    GuardMaxRange = 25\n    GuardWanderRange = 5\n    AttackRange = 50\n"
    "    ScoutRange = 90\n    DistToTargetToGrantRangeBonus = 30\n"
    "    RepairMinAltitude = 12\n    RepairMaxAltitude = 12\n"
    "    RepairRatePerSecond = 33.3\n    RepairWhenBelowHealth% = 40\n"
    "    RepairMinReadyTime = 200\n    RepairMaxReadyTime = 200\n";
// Repairs only a badly damaged master, and can also attack and scout from a repair.
static const char* const kMedic =
    "    GuardMaxRange = 30\n    AttackRange = 60\n    ScoutRange = 70\n"
    "    RepairRange = 5\n    RepairRatePerSecond = 45\n    RepairWhenBelowHealth% = 60\n"
    "    RepairMinWeldTime = 100\n    RepairMaxWeldTime = 200\n";
static const char* const kNone = "";

static const int kSlaves = 11;
static const SlaveSpec kSlave[kSlaves] = {
    {kGuard, 1, true, 1, false, 3, 100, "PRELOAD DRONE VEHICLE SCORE"},
    {kAttack, 1, false, 0, false, 5, 80, "PRELOAD DRONE CAN_ATTACK"},
    {kScout, 1, true, 0, true, 2, 60, "PRELOAD DRONE INFANTRY SCORE"},
    {kRepair, 1, false, 0, false, 4, 70, "PRELOAD DRONE"},
    {kAll, 1, true, 2, false, 3, 120, "PRELOAD DRONE VEHICLE"},
    {kNone, 1, false, 0, true, 0, 50, "PRELOAD DRONE"},
    {kGuard, 0, false, 0, false, 3, 90, "PRELOAD DRONE"},
    {kAttack, 2, false, 0, false, 3, 90, "PRELOAD DRONE"},
    {nullptr, 1, true, 2, false, 3, 40, "PRELOAD DRONE CAN_ATTACK"},
    {kMedic, 1, true, 0, true, 6, 55, "PRELOAD DRONE VEHICLE SCORE"},
    {kAll, 1, false, 1, false, 4, 65, "PRELOAD DRONE"},
};

static const int kSpawns = 8;
static const char* const kSpawn[kSpawns] = {
    "    SpawnNumber = 3\n    SpawnReplaceDelay = 1000\n    SpawnTemplateName = P_S0 P_S1\n",
    "    SpawnNumber = 2\n    OneShot = Yes\n    InitialBurst = 2\n    ExitByBudding = Yes\n"
    "    SpawnTemplateName = P_S2\n",
    "    SpawnNumber = 4\n    AggregateHealth = Yes\n    SpawnReplaceDelay = 500\n"
    "    SpawnTemplateName = P_S3\n    SpawnTemplateName = P_S4 P_S5\n",
    "    SpawnNumber = 3\n    SpawnedRequireSpawner = Yes\n    CanReclaimOrphans = Yes\n"
    "    SlavesHaveFreeWill = Yes\n    SpawnTemplateName = P_S6 P_S7 P_S8\n",
    "    SpawnNumber = 5\n    InitialBurst = 3\n    ExitByBudding = Yes\n"
    "    AggregateHealth = Yes\n    SpawnedRequireSpawner = Yes\n    SpawnReplaceDelay = 300\n"
    "    SpawnTemplateName = P_S9 P_S10\n",
    "    SpawnNumber = 1\n    OneShot = Yes\n    SpawnReplaceDelay = 2000\n"
    "    DeathTypes = NONE +EXPLODED\n    SpawnedRequireSpawner = Yes\n"
    "    SpawnTemplateName = P_SV\n",
    "    SpawnNumber = 2\n    SpawnReplaceDelay = 100\n    InitialBurst = 1\n"
    "    AggregateHealth = Yes\n    ExemptStatus = STEALTHED\n"
    "    SpawnTemplateName = P_S4 P_S0\n",
    "    SpawnNumber = 0\n    AggregateHealth = Yes\n    SpawnTemplateName = P_S1\n",
};

static const int kExits = 3;  // default, queue, none

static std::string exit_ini(int kind) {
    if (kind == 0)
        return "  Behavior = DefaultProductionExitUpdate ModuleTag_Exit\n"
               "    UnitCreatePoint = X:6 Y:2 Z:0\n    NaturalRallyPoint = X:30 Y:0 Z:0\n  End\n";
    if (kind == 1)
        return "  Behavior = QueueProductionExitUpdate ModuleTag_Exit\n"
               "    UnitCreatePoint = X:-4 Y:3 Z:1\n    NaturalRallyPoint = X:20 Y:5 Z:0\n"
               "    ExitDelay = 200\n  End\n";
    return "";
}

static bool master_is_drone(int spawn, int exit) { return (spawn + exit) % 2 == 0; }
static bool master_is_hive(int spawn, int exit) { return (spawn + 2 * exit) % 3 == 0; }

static std::string fixture_ini() {
    std::ostringstream s;
    for (int i = 0; i < kSlaves; ++i) {
        const SlaveSpec& u = kSlave[i];
        s << object_head("P_S" + std::to_string(i), u.kind_of, u.radius, 4)
          << active_body("ActiveBody", u.health);
        if (u.ai) s << "  Behavior = OracleAI ModuleTag_AI\n  End\n";
        if (u.ai == 1)
            s << "  Locomotor = SET_NORMAL P_Drone\n  Locomotor = SET_PANIC P_Panic\n";
        if (u.fields)
            s << "  Behavior = SlavedUpdate ModuleTag_Slaved\n" << u.fields << "  End\n";
        if (u.prone)
            s << "  Behavior = ProneUpdate ModuleTag_Prone\n    DamageToFramesRatio = 2\n  End\n";
        if (u.stealth)
            s << "  Behavior = StealthUpdate ModuleTag_Stealth\n    InnateStealth = "
              << (u.stealth == 2 ? "Yes" : "No") << "\n    StealthForbiddenConditions = "
              << "FIRING_PRIMARY\n  End\n";
        if (u.destroy_die) s << "  Behavior = DestroyDie ModuleTag_Die\n  End\n";
        s << "End\n";
    }
    s << object_head("P_SV", "PRELOAD DRONE", 3, 4) << "  BuildVariations = P_S0 P_S2 P_S3\n"
      << active_body("ActiveBody", 100) << "End\n";
    for (int b = 0; b < kSpawns; ++b) {
        for (int e = 0; e < kExits; ++e) {
            std::string kind_of = "PRELOAD";
            if (master_is_drone(b, e)) kind_of += " DRONE";
            if (b % 2 == 1) kind_of += " SPAWNS_ARE_THE_WEAPONS";
            if (b == 3 || b == 4) kind_of += " STRUCTURE";
            s << object_head("P_M" + std::to_string(b) + "_" + std::to_string(e), kind_of,
                             4 + b, 10);
            if (master_is_hive(b, e))
                s << active_body("HiveStructureBody", 500,
                                 "    PropagateDamageTypesToSlavesWhenExisting = NONE +SMALL_ARMS "
                                 "+SNIPER\n    SwallowDamageTypesIfSlavesNotExisting = NONE +SNIPER\n");
            else
                s << active_body("ActiveBody", 500);
            s << "  Behavior = OracleAI ModuleTag_AI\n  End\n";
            s << "  Behavior = SpawnBehavior ModuleTag_Spawn\n" << kSpawn[b] << "  End\n";
            s << exit_ini(e) << "End\n";
        }
    }
    // Barracks: structures with each exit, and a vehicle that is not a structure.
    for (int k = 0; k < 4; ++k) {
        s << object_head("P_Barracks" + std::to_string(k),
                         k == 3 ? "PRELOAD VEHICLE" : "PRELOAD STRUCTURE", 12, 8)
          << active_body("ActiveBody", 300) << exit_ini(k == 3 ? 0 : k) << "End\n";
    }
    s << object_head("P_Enemy", "PRELOAD VEHICLE", 5, 4) << active_body("ActiveBody", 1000)
      << "End\n";
    s << object_head("P_Plane", "PRELOAD AIRCRAFT", 5, 4) << active_body("ActiveBody", 1000)
      << "End\n";
    return s.str();
}

// Weapons only steer the real attack queries, whose answers go into the vectors.
static std::string with_weapons(std::string ini) {
    static const char* const owners[] = {"P_S1\n", "P_S4\n", "P_S8\n", "P_S2\n"};
    for (const char* name : owners) {
        const std::string head = std::string("Object ") + name;
        const size_t at = ini.find(head);
        if (at == std::string::npos) die("weapon owner");
        ini.insert(at + head.size(),
                   std::string("  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY ") +
                       (std::strcmp(name, "P_S2\n") == 0 ? "P_Short" : "P_Gun") + "\n  End\n");
    }
    return ini;
}

static const char* const kWeapons =
    "Weapon P_Gun\n  PrimaryDamage = 10\n  AttackRange = 120\n  DamageType = SMALL_ARMS\n"
    "  AntiGround = Yes\nEnd\n"
    "Weapon P_Short\n  PrimaryDamage = 10\n  AttackRange = 15\n  MinimumAttackRange = 0\n"
    "  DamageType = SMALL_ARMS\n  AntiGround = Yes\nEnd\n";

// --- scene ------------------------------------------------------------------

struct Scene {
    ObjectID base = INVALID_ID;
    std::map<int, std::string> last;
    int built0[3] = {0, 0, 0};
};

static Object* find(int id) { return TheGameLogic->findObjectByID(static_cast<ObjectID>(id)); }

static std::vector<int> scene_ids(const Scene& s) {
    std::vector<int> ids;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject())
        if (o->getID() >= s.base) ids.push_back(static_cast<int>(o->getID()));
    std::sort(ids.begin(), ids.end());
    return ids;
}

static BehaviorModule* module_named(Object* o, const char* name) {
    const NameKeyType key = TheNameKeyGenerator->nameToKey(name);
    for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m)
        if ((*m)->getModuleNameKey() == key) return *m;
    return nullptr;
}

static void put_module(std::ostringstream& out, Object* o, const char* name) {
    BehaviorModule* m = module_named(o, name);
    if (!m) {
        out << " -";
        return;
    }
    UpdateModule* u = static_cast<UpdateModule*>(m);
    out << ' ' << oracle_wp51::World::snapshot_save_hex(u) << ' '
        << oracle_wp51::World::snapshot_crc(u);
}

static std::string object_row(const Scene& s, Object* o) {
    std::ostringstream out;
    const Coord3D* p = o->getPosition();
    out << static_cast<int>(o->getProducerID()) << ' '
        << (o->getTeam() == nullptr ? -1 : o->getControllingPlayer()->getPlayerIndex()) << ' '
        << bits(p->x) << ' ' << bits(p->y) << ' ' << bits(p->z) << ' ' << bits(o->getOrientation())
        << ' ' << static_cast<int>(o->getLayer()) << ' ';
    static const ObjectStatusTypes status[] = {
        OBJECT_STATUS_UNSELECTABLE, OBJECT_STATUS_STEALTHED, OBJECT_STATUS_CAN_STEALTH,
        OBJECT_STATUS_MASKED, OBJECT_STATUS_NO_ATTACK, OBJECT_STATUS_DETECTED,
        OBJECT_STATUS_UNDER_CONSTRUCTION, OBJECT_STATUS_SOLD, OBJECT_STATUS_RECONSTRUCTING};
    for (ObjectStatusTypes t : status) out << (o->testStatus(t) ? 1 : 0);
    out << ' ';
    static const DisabledType disabled[] = {DISABLED_HELD, DISABLED_UNMANNED, DISABLED_EMP,
                                            DISABLED_HACKED, DISABLED_SUBDUED};
    for (DisabledType t : disabled) out << (o->isDisabledByType(t) ? 1 : 0);
    BodyModuleInterface* body = o->getBodyModule();
    out << ' ' << static_cast<int>(o->getVeterancyLevel()) << ' ' << bits(body->getHealth()) << ' '
        << bits(body->getMaxHealth()) << ' ' << (o->isEffectivelyDead() ? 1 : 0);
    const Coord3D& box = o->*access_member(HealthBoxTag());
    out << ' ' << bits(box.x) << ' ' << bits(box.y) << ' ' << bits(box.z) << ' ';
    static const ModelConditionFlagType model[] = {
        MODELCONDITION_PACKING, MODELCONDITION_UNPACKING, MODELCONDITION_FIRING_B,
        MODELCONDITION_FIRING_C, MODELCONDITION_BETWEEN_FIRING_SHOTS_B,
        MODELCONDITION_BETWEEN_FIRING_SHOTS_C, MODELCONDITION_RELOADING_B,
        MODELCONDITION_RELOADING_C, MODELCONDITION_PRONE};
    Drawable* d = o->getDrawable();
    if (!d) die("object without a drawable");
    for (ModelConditionFlagType t : model) out << (d->getModelConditionFlags().test(t) ? 1 : 0);
    out << ' ' << ((o->getWeaponBonusCondition() >> WEAPONBONUSCONDITION_DRONE_SPOTTING) & 1u);
    AIUpdateInterface* ai = o->getAIUpdateInterface();
    if (ai && ai->getCurLocomotor()) {
        const UnsignedInt flags = ai->getCurLocomotor()->*access_member(LocoFlagsTag());
        out << ' ' << static_cast<int>(ai->getCurLocomotorSetType()) << ' ' << ((flags >> 3) & 1u)
            << ' ' << ((flags >> 6) & 1u);
    } else {
        out << " - - -";
    }
    put_module(out, o, "SpawnBehavior");
    // The reference saves version 2, which leaves out the burst countdown.
    if (BehaviorModule* sb = module_named(o, "SpawnBehavior"))
        out << ' ' << static_cast<SpawnBehavior*>(sb)->*access_member(BurstTag());
    put_module(out, o, "SlavedUpdate");
    put_module(out, o, "ProneUpdate");
    put_module(out, o, "StealthUpdate");
    return out.str();
}

static int units_built(int player) {
    return ThePlayerList->getNthPlayer(player)->getScoreKeeper()->getTotalUnitsBuilt();
}

static std::string row(Scene& s, const std::string& query) {
    std::vector<std::string> parts;
    const std::vector<int> ids = scene_ids(s);
    for (int id : ids) {
        const std::string now = object_row(s, find(id));
        auto it = s.last.find(id);
        if (it == s.last.end() || it->second != now) {
            parts.push_back("O " + std::to_string(id) + " " + now);
            s.last[id] = now;
        }
    }
    for (auto it = s.last.begin(); it != s.last.end();) {
        if (!std::binary_search(ids.begin(), ids.end(), it->first)) {
            parts.push_back("X " + std::to_string(it->first));
            it = s.last.erase(it);
        } else {
            ++it;
        }
    }
    for (const std::string& c : g_commands) parts.push_back("C " + c);
    g_commands.clear();
    if (!query.empty()) parts.push_back("Q " + query);
    std::ostringstream tail;
    tail << "R " << GameLogicRandomValue(0, 1000000);
    for (int p = 0; p < 3; ++p) tail << ' ' << units_built(p) - s.built0[p];
    parts.push_back(tail.str());
    std::string out;
    for (size_t i = 0; i < parts.size(); ++i) out += (i ? ";" : "") + parts[i];
    return out;
}

static bool runs_while_disabled(Object* o, UpdateModule* u) {
    if (!o->isDisabled()) return true;
    DisabledMaskType mine = o->getDisabledFlags();
    return u->getDisabledTypesToProcess().testForAll(mine);
}

static void tick(Scene& s, UnsignedInt frame, oracle_wp51::World& w) {
    w.set_frame(frame);
    static const char* const updates[] = {"SpawnBehavior", "SlavedUpdate", "ProneUpdate",
                                          "QueueProductionExitUpdate"};
    for (int id : scene_ids(s)) {
        Object* o = find(id);
        if (!o || o->isDestroyed()) continue;
        for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m) {
            for (const char* name : updates) {
                if ((*m)->getModuleNameKey() != TheNameKeyGenerator->nameToKey(name)) continue;
                UpdateModule* u = static_cast<UpdateModule*>(*m);
                if (!o->isDestroyed() && runs_while_disabled(o, u))
                    static_cast<UpdateModuleInterface*>(u)->update();
            }
        }
    }
    w.flush_destroy_list();
}

static Coord3D at(float x, float y, float z) {
    Coord3D c;
    c.x = x;
    c.y = y;
    c.z = z;
    return c;
}

static float ground(float x, float y) { return g_slope ? x * 0.01f + y * 0.02f : 0.0f; }

static int g_op_counts[32];

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_pkt296_365_spawn_slaved <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheTerrainLogic = NEW SlopeTerrain;
    TheTerrainLogic->init();
    (TheModuleFactory->*(&ExposedFactory::addModuleInternal))(
        make_rec_ai, AIUpdateInterface::friend_newModuleData, MODULETYPE_BEHAVIOR, "OracleAI",
        AIUpdateInterface::getInterfaceMask());
    w.load_ini(kLocomotors);
    w.load_ini(kWeapons);
    const std::string fixture = fixture_ini();
    w.load_ini(with_weapons(fixture));
    w.init_with_neutral_player();
    ThePlayerList->setLocalPlayer(ThePlayerList->getNthPlayer(5));
    Team* teams[3] = {w.team_neutral, w.team_a, w.team_b};

    FILE* out = std::fopen(argv[1], "w");
    if (!out) return 1;
    std::string escaped = fixture;
    for (char& c : escaped)
        if (c == '\n') c = '~';
    std::string locos = kLocomotors;
    for (char& c : locos)
        if (c == '\n') c = '~';
    const int kTotal = 1200;
    std::fprintf(out, "{\"vector_count\":%d,\"locomotors\":\"%s\",\"objects\":\"%s\",\"vectors\":[",
                 kTotal, locos.c_str(), escaped.c_str());

    for (int i = 0; i < kTotal; ++i) {
        g_slope = pick(3) == 0;
        UnsignedInt frame = 1 + pick(5000);
        w.set_frame(frame);
        const UnsignedInt seed = 1 + pick(1000000000);
        InitRandom(seed);
        g_commands.clear();

        Scene s;
        for (int p = 0; p < 3; ++p) s.built0[p] = units_built(p);

        const int spawn = i % kSpawns;
        const int exit = static_cast<int>(pick(pick(4) == 0 ? 3 : 2));
        const int barracks_kind = pick(2) == 0 ? static_cast<int>(pick(4)) : -1;
        const int team = pick(10) == 0 ? 2 : 1;
        const float mx = real_in(250.0f, 750.0f), my = real_in(250.0f, 750.0f);
        const float angle = pick(2) ? 0.0f : real_in(-3.1f, 3.1f);
        const float mz = pick(4) == 0 ? real_in(0.0f, 20.0f) : ground(mx, my);
        const float reach = pick(2) ? 70.0f : 200.0f;
        const float ex = mx + real_in(-reach, reach), ey = my + real_in(-reach, reach);
        const float px = mx + real_in(-120.0f, 120.0f), py = my + real_in(-120.0f, 120.0f);

        // A producer that no longer exists also counts as runtime production.
        const int orphaned = barracks_kind < 0 && pick(5) == 0;

        Object* master = nullptr;
        Object* barracks = nullptr;
        const std::string mname = "P_M" + std::to_string(spawn) + "_" + std::to_string(exit);
        Object* enemy = w.spawn_object("P_Enemy", w.team_b, at(ex, ey, ground(ex, ey)), 0.0f);
        s.base = enemy->getID();
        Object* plane = w.spawn_object("P_Plane", w.team_b, at(px, py, 60.0f), 0.0f);
        if (barracks_kind >= 0) {
            const std::string bname = "P_Barracks" + std::to_string(barracks_kind);
            barracks = w.spawn_object(bname.c_str(), w.team_a,
                                      at(mx - 40.0f, my, ground(mx - 40.0f, my)), 0.0f);
        }
        master = w.spawn_object(mname.c_str(), teams[team], at(mx, my, mz), angle);
        if (!enemy || !plane || !master) die("missing template");
        if (barracks) master->setProducer(barracks);
        if (orphaned) master->setProducer(plane), TheGameLogic->destroyObject(plane),
            w.flush_destroy_list(), plane = nullptr;
        const int master_id = static_cast<int>(master->getID());
        const int enemy_id = static_cast<int>(enemy->getID());
        const int plane_id = plane ? static_cast<int>(plane->getID()) : 0;

        std::fprintf(out,
                     "%s\n{\"frame\":%u,\"seed\":%u,\"slope\":%d,\"id\":%u,\"master\":\"%s\","
                     "\"team\":%d,\"barracks\":%d,\"orphaned\":%d,\"x\":%u,\"y\":%u,\"z\":%u,"
                     "\"angle\":%u,\"ex\":%u,\"ey\":%u,\"px\":%u,\"py\":%u",
                     i ? "," : "", frame, seed, g_slope ? 1 : 0, static_cast<unsigned>(s.base),
                     mname.c_str(), team, barracks_kind, orphaned, bits(mx), bits(my), bits(mz),
                     bits(angle), bits(ex), bits(ey), bits(px), bits(py));
        const std::string spawn_row = row(s, "");

        std::vector<std::string> ops, rows;
        const int n_ops = 8 + static_cast<int>(pick(20));
        for (int k = 0; k < n_ops; ++k) {
            const std::vector<int> ids = scene_ids(s);
            std::vector<int> slaves;
            for (int id : ids) {
                Object* o = find(id);
                const char* n = o->getTemplate()->getName().str();
                if (std::strncmp(n, "P_S", 3) == 0) slaves.push_back(id);
            }
            auto any_slave = [&]() -> int {
                return slaves.empty() ? 0 : slaves[pick(static_cast<uint32_t>(slaves.size()))];
            };
            auto any_object = [&]() -> int {
                return ids.empty() ? 0 : ids[pick(static_cast<uint32_t>(ids.size()))];
            };
            Object* m = find(master_id);
            const Coord3D home = m ? *m->getPosition() : at(mx, my, mz);

            uint32_t code = 0;
            const uint32_t roll = pick(100);
            if (k == 0) code = pick(8) == 0 ? 6 : 0;
            else if (roll < 30) code = 0;
            else if (roll < 37) code = 1;
            else if (roll < 40) code = 2;
            else if (roll < 52) code = 3;
            else if (roll < 58) code = 4;
            else if (roll < 62) code = 5;
            else if (roll < 67) code = 6;
            else if (roll < 71) code = 7;
            else if (roll < 75) code = 8;
            else if (roll < 77) code = 9;
            else if (roll < 84) code = 10;
            else if (roll < 89) code = 11;
            else if (roll < 91) code = 12;
            else if (roll < 93) code = 13;
            else if (roll < 94) code = 14;
            else if (roll < 96) code = 15;
            else if (roll < 98) code = 16;
            else code = 17;
            ++g_op_counts[code];

            int a = 0, b = 0, c = 0, d = 0, e = 0;
            std::string extra;
            switch (code) {
            case 0:  // run frames
                a = 1 + static_cast<int>(pick(pick(3) == 0 ? 70 : 12));
                break;
            case 1: {  // damage: target, amount, type, source, death type
                a = pick(3) == 0 ? master_id : (pick(8) == 0 ? any_object() : any_slave());
                if (pick(4) == 0) a = master_id;
                // Whole multiples of 5 land on exact percentages of the master's health;
                // a quarter point sits just under full health.
                const uint32_t amount = pick(7);
                b = static_cast<int>(bits(
                    amount == 0 ? 100000.0f
                    : amount == 1 ? real_in(100.0f, 400.0f)
                    : amount == 2 ? 5.0f * static_cast<float>(1 + pick(99))
                    : amount == 3 ? 0.25f
                                  : real_in(1.0f, 60.0f)));
                static const DamageType types[] = {DAMAGE_SMALL_ARMS, DAMAGE_SNIPER,
                                                   DAMAGE_EXPLOSION, DAMAGE_UNRESISTABLE};
                c = static_cast<int>(types[pick(4)]);
                d = pick(3) == 0 ? 0 : (pick(6) == 0 ? any_slave() : enemy_id);
                e = static_cast<int>(pick(2) ? DEATH_EXPLODED : DEATH_NORMAL);
                break;
            }
            case 2:  // destroy
                a = pick(2) ? master_id : any_slave();
                break;
            case 3: {  // teleport, mostly a slave near its master
                a = pick(5) == 0 ? any_object() : any_slave();
                if (pick(8) == 0) a = master_id;
                const float spread = pick(2) ? 14.0f : (pick(2) ? 80.0f : 240.0f);
                const float x = home.x + real_in(-spread, spread), y = home.y + real_in(-spread, spread);
                float z = pick(3) == 0 ? home.z + real_in(-30.0f, 30.0f) : ground(x, y);
                if (pick(6) == 0) z = home.z;
                b = static_cast<int>(bits(x));
                c = static_cast<int>(bits(y));
                d = static_cast<int>(bits(z));
                // A bridge layer now and then, for StayOnSameLayerAsMaster.
                e = pick(4) == 0 ? 2 + static_cast<int>(pick(3)) : (pick(4) == 0 ? 1 : 0);
                break;
            }
            case 4:  // AI state: object, idle, command source, victim
                a = pick(2) ? master_id : any_slave();
                b = static_cast<int>(pick(3) != 0);
                c = static_cast<int>(pick(3) == 0 ? CMD_FROM_PLAYER : CMD_FROM_AI);
                d = pick(2) ? 0 : (pick(2) ? enemy_id : (pick(2) ? plane_id : any_slave()));
                break;
            case 5: {  // path: object, present, destination
                a = master_id;
                b = static_cast<int>(pick(3) != 0);
                const float spread = pick(2) ? 30.0f : 150.0f;
                c = static_cast<int>(bits(home.x + real_in(-spread, spread)));
                d = static_cast<int>(bits(home.y + real_in(-spread, spread)));
                break;
            }
            case 6: {  // status: object, which, set
                a = pick(4) == 0 ? any_slave() : master_id;
                static const ObjectStatusTypes status[] = {
                    OBJECT_STATUS_UNDER_CONSTRUCTION, OBJECT_STATUS_SOLD,
                    OBJECT_STATUS_RECONSTRUCTING, OBJECT_STATUS_STEALTHED};
                b = static_cast<int>(status[pick(4)]);
                c = static_cast<int>(pick(5) < 2);
                if (k == 0) { a = master_id; b = static_cast<int>(status[pick(3)]); c = 1; }
                break;
            }
            case 7:  // capture
                a = pick(4) == 0 ? any_slave() : master_id;
                b = static_cast<int>(pick(3));
                break;
            case 8: {  // disable or clear
                a = pick(4) == 0 ? any_slave() : master_id;
                static const DisabledType types[] = {DISABLED_EMP, DISABLED_HACKED, DISABLED_SUBDUED,
                                                     DISABLED_HELD, DISABLED_UNMANNED};
                const bool drone = find(a) && find(a)->isKindOf(KINDOF_DRONE);
                b = static_cast<int>(types[pick(drone ? 5 : 4)]);
                c = static_cast<int>(pick(3) != 0);
                d = static_cast<int>(frame + 100000 + pick(3));
                break;
            }
            case 9:  // veterancy
                a = pick(2) ? master_id : any_slave();
                b = static_cast<int>(pick(4));
                break;
            case 10: {  // queries: position, ratio, target
                a = static_cast<int>(bits(home.x + real_in(-100.0f, 100.0f)));
                b = static_cast<int>(bits(home.y + real_in(-100.0f, 100.0f)));
                static const float ratios[] = {0.0f, 0.25f, 0.5f, 1.0f, 2.0f, -1.0f, 0.34f};
                c = static_cast<int>(bits(ratios[pick(7)]));
                d = pick(3) == 0 && plane_id && find(plane_id) ? plane_id : enemy_id;
                break;
            }
            case 11:  // orders: kind, target, max shots, command source
                a = static_cast<int>(pick(3));
                b = pick(2) ? enemy_id : (plane_id && find(plane_id) ? plane_id : enemy_id);
                c = static_cast<int>(pick(9)) - 1;
                d = static_cast<int>(pick(2) ? CMD_FROM_PLAYER : CMD_FROM_SCRIPT);
                break;
            case 12:  // give or take the stealth upgrade
                a = static_cast<int>(pick(3) != 0);
                break;
            case 13:  // reveal the slaves; a = mark their stealth granted first
                a = static_cast<int>(pick(2));
                break;
            case 14:  // stop or start spawning
                a = static_cast<int>(pick(2));
                break;
            case 15:  // enslave: slave, slaver
                a = any_slave();
                b = pick(3) == 0 ? any_object() : (pick(4) == 0 ? 0 : master_id);
                // A scouting slave dereferences the AI of its slaver.
                if (a && find(b) && !find(b)->getAIUpdateInterface()) {
                    const char* n = find(a)->getTemplate()->getName().str();
                    if (!std::strcmp(n, "P_S2") || !std::strcmp(n, "P_S4") ||
                        !std::strcmp(n, "P_S9") || !std::strcmp(n, "P_S10"))
                        b = master_id;
                }
                break;
            case 16:  // the slaver died, as SlowDeathBehavior tells a slave
                a = any_slave();
                break;
            case 17:  // orphan a slave
                a = any_slave();
                break;
            }

            w.set_frame(frame);
            std::string query;
            Object* target_a = find(a);
            switch (code) {
            case 0:
                for (int t = 0; t < a; ++t) tick(s, ++frame, w);
                break;
            case 1:
                if (target_a) {
                    DamageInfo info;
                    info.in.m_amount = real_of(static_cast<uint32_t>(b));
                    info.in.m_damageType = static_cast<DamageType>(c);
                    info.in.m_sourceID = static_cast<ObjectID>(d);
                    info.in.m_deathType = static_cast<DeathType>(e);
                    target_a->attemptDamage(&info);
                    query = "damage " + std::to_string(bits(info.out.m_actualDamageDealt)) + ' ' +
                            std::to_string(bits(info.out.m_actualDamageClipped));
                }
                break;
            case 2:
                if (target_a) TheGameLogic->destroyObject(target_a);
                break;
            case 3:
                if (target_a) {
                    const Coord3D pos = at(real_of(static_cast<uint32_t>(b)),
                                           real_of(static_cast<uint32_t>(c)),
                                           real_of(static_cast<uint32_t>(d)));
                    target_a->setPosition(&pos);
                    if (e) target_a->setLayer(static_cast<PathfindLayerEnum>(e));
                }
                break;
            case 4:
                if (target_a && target_a->getAIUpdateInterface()) {
                    RecAI* ai = rec_ai(target_a);
                    ai->m_idle = b;
                    ai->m_source = static_cast<CommandSourceType>(c);
                    ai->setCurrentVictim(find(d));
                }
                break;
            case 5:
                if (target_a && target_a->getAIUpdateInterface()) {
                    AIUpdateInterface* ai = target_a->getAIUpdateInterface();
                    ai->destroyPath();
                    if (b) {
                        Path* path = newInstance(Path);
                        const Coord3D first = *target_a->getPosition();
                        Coord3D last = at(real_of(static_cast<uint32_t>(c)),
                                          real_of(static_cast<uint32_t>(d)), 0.0f);
                        path->appendNode(&first, LAYER_GROUND);
                        path->appendNode(&last, LAYER_GROUND);
                        ai->*access_member(PathTag()) = path;
                    }
                }
                break;
            case 6:
                if (target_a)
                    target_a->setStatus(
                        MAKE_OBJECT_STATUS_MASK(static_cast<ObjectStatusTypes>(b)), c);
                break;
            case 7:
                if (target_a) target_a->setTeam(teams[b]);
                break;
            case 8:
                if (target_a) {
                    if (c) target_a->setDisabledUntil(static_cast<DisabledType>(b),
                                                      static_cast<UnsignedInt>(d));
                    else target_a->clearDisabled(static_cast<DisabledType>(b));
                }
                break;
            case 9:
                if (target_a)
                    target_a->getExperienceTracker()->setVeterancyLevel(
                        static_cast<VeterancyLevel>(b));
                break;
            case 10:
            case 11:
            case 12:
            case 13:
            case 14: {
                Object* mm = find(master_id);
                if (!mm) break;
                SpawnBehaviorInterface* sbi = mm->getSpawnBehaviorInterface();
                if (code == 10) {
                    const Coord3D pos = at(real_of(static_cast<uint32_t>(a)),
                                           real_of(static_cast<uint32_t>(b)), 0.0f);
                    Object* target = find(d);
                    if (!target) die("missing query target");
                    std::ostringstream abilities, q;
                    for (int id : scene_ids(s)) {
                        Object* o = find(id);
                        if (std::strncmp(o->getTemplate()->getName().str(), "P_S", 3) != 0) continue;
                        abilities << ' ' << id << ' ' << (o->isAbleToAttack() ? 1 : 0) << ' '
                                  << static_cast<int>(o->getAbleToAttackSpecificObject(
                                         ATTACK_NEW_TARGET, target, CMD_FROM_PLAYER))
                                  << ' ' << static_cast<int>(o->getAbleToUseWeaponAgainstTarget(
                                                 ATTACK_NEW_TARGET, target, &pos, CMD_FROM_PLAYER));
                    }
                    extra = abilities.str();
                    Object* closest = sbi->getClosestSlave(&pos);
                    q << "spawn " << sbi->getSlaveCount() << ' ' << (sbi->doSlavesHaveFreedom() ? 1 : 0)
                      << ' ' << (closest ? static_cast<int>(closest->getID()) : 0) << ' '
                      << (sbi->maySpawnSelfTaskAI(real_of(static_cast<uint32_t>(c))) ? 1 : 0) << ' '
                      << (sbi->canAnySlavesAttack() ? 1 : 0) << ' '
                      << static_cast<int>(sbi->getCanAnySlavesAttackSpecificTarget(
                             ATTACK_NEW_TARGET, target, CMD_FROM_PLAYER))
                      << ' ' << static_cast<int>(sbi->getCanAnySlavesUseWeaponAgainstTarget(
                                    ATTACK_NEW_TARGET, target, &pos, CMD_FROM_PLAYER));
                    q << ' ' << (sbi->areAllSlavesStealthed() ? 1 : 0);
                    query = q.str();
                } else if (code == 11) {
                    Object* target = find(b);
                    if (a == 0 && target)
                        sbi->orderSlavesToAttackTarget(target, c, static_cast<CommandSourceType>(d));
                    else if (a == 1 && target)
                        sbi->orderSlavesToAttackPosition(target->getPosition(), c,
                                                         static_cast<CommandSourceType>(d));
                    else if (a == 2)
                        sbi->orderSlavesToGoIdle(static_cast<CommandSourceType>(d));
                } else if (code == 12) {
                    sbi->giveSlavesStealthUpgrade(a);
                } else if (code == 13) {
                    if (a) {
                        for (int id : scene_ids(s)) {
                            StealthUpdate* st = find(id)->getStealth();
                            if (st) st->receiveGrant(TRUE, 0);
                        }
                    }
                    sbi->revealSlaves();
                } else {
                    SpawnBehavior* sb = static_cast<SpawnBehavior*>(module_named(mm, "SpawnBehavior"));
                    if (a) sb->startSpawning();
                    else sb->stopSpawning();
                }
                break;
            }
            case 15:
            case 16:
                if (target_a) {
                    for (BehaviorModule** u = target_a->getBehaviorModules(); *u; ++u) {
                        SlavedUpdateInterface* sdu = (*u)->getSlavedUpdateInterface();
                        if (!sdu) continue;
                        if (code == 15) sdu->onEnslave(find(b));
                        else sdu->onSlaverDie(nullptr);
                        query = "slaver " + std::to_string(static_cast<int>(sdu->getSlaverID())) + ' ' +
                                std::to_string(sdu->isSelfTasking() ? 1 : 0);
                        break;
                    }
                }
                break;
            case 17:
                if (target_a) target_a->setProducer(nullptr);
                break;
            }
            w.flush_destroy_list();

            std::ostringstream op;
            op << code << ' ' << a << ' ' << b << ' ' << c << ' ' << d << ' ' << e << extra;
            ops.push_back(op.str());
            rows.push_back(row(s, query));
        }

        std::fprintf(out, ",\"spawn\":\"%s\",\"ops\":\"", spawn_row.c_str());
        for (size_t k = 0; k < ops.size(); ++k)
            std::fprintf(out, "%s%s", k ? ";" : "", ops[k].c_str());
        std::fprintf(out, "\",\"rows\":\"");
        for (size_t r = 0; r < rows.size(); ++r)
            std::fprintf(out, "%s%s", r ? "|" : "", rows[r].c_str());
        std::fprintf(out, "\"}");

        // The slaves go first, so the master's onDelete finds nothing left to destroy.
        for (int id : scene_ids(s))
            if (find(id)) TheGameLogic->destroyObject(find(id));
        w.flush_destroy_list();
        g_commands.clear();
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    for (int i = 0; i < 18; ++i) std::printf("op %d: %d\n", i, g_op_counts[i]);
    return 0;
}
