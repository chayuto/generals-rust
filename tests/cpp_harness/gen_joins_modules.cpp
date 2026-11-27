// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for ported modules working together, run on the unmodified reference
// modules in one small real world: ProductionUpdate with the Default, Queue and
// SupplyCenter production exits, SpawnBehavior and SlavedUpdate (budding through the
// producing factory's door), PhysicsBehavior, StealthUpdate, StealthDetectorUpdate,
// SupplyCenterDockUpdate, AutoDepositUpdate, PowerPlantUpdate, PowerPlantUpgrade,
// StatusBitsUpgrade, StealthUpgrade, DestroyDie and ActiveBody, driven by the engine's
// sleepy update loop, the partition manager and the destroy list.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_joins_modules.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/JOINS_Modules.json
//
// The only stand-in is the AI: a subclass that records its orders, never updates, never
// reacts to a collision, and answers the idle state and last command source as scripted
// (its constructor, state machine and locomotor choice are the real ones). The truck
// variant also holds a scripted box count. Rows are hashed after the first two vectors;
// JOINS_HEX=1 writes everything in full (also read by the Rust test).
//
// Not covered: a real AI, weapons and locomotion; power brownout (no KINDOF_POWERED
// object); timed disabling (`Object::checkDisabledStatus` is not part of the port);
// players with more than one team; bone positions; loading a save.

#include "oracle_wp51_world.h"

#include "Common/AcademyStats.h"
#include "Common/Energy.h"
#include "Common/Module.h"
#include "Common/ModuleFactory.h"
#include "Common/Money.h"
#include "Common/ScoreKeeper.h"
#include "GameClient/Drawable.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/DockUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/ProductionUpdate.h"
#include "GameLogic/Module/SpawnBehavior.h"
#include "GameLogic/Module/SupplyTruckAIUpdate.h"
#include "GameLogic/Module/UpdateModule.h"

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <sstream>
#include <string>
#include <vector>

#define ACCESS(tag, cls, field, member)             \
    struct tag { using type = field cls::*; };      \
    template struct MemberAccess<tag, &cls::member>; \
    inline tag::type access_member(tag);
#define M(obj, tag) ((obj)->*access_member(tag()))

ACCESS(PMoney, Money, UnsignedInt, m_money)
ACCESS(PEProd, Energy, Int, m_energyProduction)
ACCESS(PECons, Energy, Int, m_energyConsumption)
ACCESS(PESab, Energy, UnsignedInt, m_powerSabotagedTillFrame)
ACCESS(STech, ScoreKeeper, Int, m_totalTechBuildingsCaptured)
ACCESS(SFaction, ScoreKeeper, Int, m_totalFactionBuildingsCaptured)
ACCESS(SEarned, ScoreKeeper, Int, m_totalMoneyEarned)
ACCESS(SSpent, ScoreKeeper, Int, m_totalMoneySpent)
ACCESS(ASupply, AcademyStats, UnsignedInt, m_supplyCentersBuilt)
ACCESS(APeons, AcademyStats, UnsignedInt, m_peonsBuilt)
ACCESS(ALastUnit, AcademyStats, UnsignedInt, m_lastUnitBuiltFrame)
ACCESS(AGather, AcademyStats, UnsignedInt, m_gatherersBuilt)
ACCESS(ALastIncome, AcademyStats, UnsignedInt, m_lastIncomeFrame)
ACCESS(AMaxIncome, AcademyStats, UnsignedInt, m_maxFramesBetweenIncome)
ACCESS(ABurst, SpawnBehavior, UnsignedInt, m_initialBurstCountdown)
ACCESS(GHeap, GameLogic, std::vector<UpdateModulePtr>, m_sleepyUpdates)
ACCESS(GCur, GameLogic, UpdateModulePtr, m_curUpdateModule)

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }

static void die(const char* what) {
    std::fprintf(stderr, "oracle: %s\n", what);
    std::exit(1);
}

static uint32_t g_state = 0x1017C0DEu;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}

// --- logs ---------------------------------------------------------------------------
static std::string g_orders, g_paths, g_texts, g_prone;

static void record(Object* me, const AICommandParms* p) {
    std::ostringstream s;
    const int id = static_cast<int>(me->getID());
    const int src = static_cast<int>(p->m_cmdSource);
    const auto xyz = [&](const Coord3D& c) { s << ' ' << bits(c.x) << ' ' << bits(c.y) << ' ' << bits(c.z); };
    switch (p->m_cmd) {
    case AICMD_GO_PRONE: g_prone += std::to_string(id) + ";"; return;
    case AICMD_IDLE: s << id << " idle " << src; break;
    case AICMD_EXIT:
        s << id << " exit " << src << ' ' << (p->m_obj ? static_cast<int>(p->m_obj->getID()) : 0);
        break;
    case AICMD_MOVE_TO_POSITION: s << id << " move " << src; xyz(p->m_pos); break;
    case AICMD_FORCE_ATTACK_OBJECT:
        s << id << " force " << src << ' ' << (p->m_obj ? static_cast<int>(p->m_obj->getID()) : 0)
          << ' ' << p->m_intValue;
        break;
    case AICMD_ATTACK_POSITION: s << id << " attackpos " << src; xyz(p->m_pos); s << ' ' << p->m_intValue; break;
    case AICMD_FOLLOW_EXITPRODUCTION_PATH:
        s << id << ' ' << (p->m_obj ? static_cast<int>(p->m_obj->getID()) : 0) << ' ' << src;
        for (const Coord3D& c : p->m_coords) xyz(c);
        g_paths += s.str() + ";";
        return;
    default:
        std::fprintf(stderr, "oracle: unexpected AI command %d\n", static_cast<int>(p->m_cmd));
        std::exit(1);
    }
    g_orders += s.str() + ";";
}

class JoinAI : public AIUpdateInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(JoinAI, "OracleJoinAI", 128, 128)
public:
    Bool m_idle = TRUE;
    CommandSourceType m_source = CMD_FROM_AI;
    JoinAI(Thing* t, const ModuleData* d) : AIUpdateInterface(t, d) {}
    Bool isIdle() const override { return m_idle; }
    CommandSourceType getLastCommandSource() const override { return m_source; }
    void aiDoCommand(const AICommandParms* p) override { record(getObject(), p); }
    UpdateSleepTime update() override { return UPDATE_SLEEP_FOREVER; }
    DisabledMaskType getDisabledTypesToProcess() const override { return DISABLEDMASK_ALL; }
    Bool processCollision(PhysicsBehavior*, Object*) override { return FALSE; }
};
JoinAI::~JoinAI() {}

class JoinTruck : public SupplyTruckAIUpdate {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(JoinTruck, "OracleJoinTruck", 64, 64)
public:
    Bool m_idle = TRUE;
    CommandSourceType m_source = CMD_FROM_AI;
    Int m_boxes = 0;
    Bool m_wanting = FALSE;
    JoinTruck(Thing* t, const ModuleData* d) : SupplyTruckAIUpdate(t, d) {}
    Bool isIdle() const override { return m_idle; }
    CommandSourceType getLastCommandSource() const override { return m_source; }
    void aiDoCommand(const AICommandParms* p) override { record(getObject(), p); }
    UpdateSleepTime update() override { return UPDATE_SLEEP_FOREVER; }
    DisabledMaskType getDisabledTypesToProcess() const override { return DISABLEDMASK_ALL; }
    Bool processCollision(PhysicsBehavior*, Object*) override { return FALSE; }
    Int getNumberBoxes() const override { return m_boxes; }
    Int getMaxBoxes() const override { return 4; }
    Bool loseOneBox() override {
        if (m_boxes == 0) return FALSE;
        --m_boxes;
        return TRUE;
    }
    Int getUpgradedSupplyBoost() const override { return 20; }
    void setForceWantingState(Bool v) override { m_wanting = v; }
    Bool isForcedIntoWantingState() const override { return m_wanting; }
};
JoinTruck::~JoinTruck() {}

static Module* make_ai(Thing* t, const ModuleData* d) { return newInstance(JoinAI)(t, d); }
static Module* make_truck(Thing* t, const ModuleData* d) { return newInstance(JoinTruck)(t, d); }
struct ExposedFactory : public ModuleFactory { using ModuleFactory::addModuleInternal; };

class CashText : public oracle_wp51::NullGameText {
public:
    UnicodeString fetch(const Char* label, Bool* exists = nullptr) override {
        if (exists) *exists = TRUE;
        if (std::strcmp(label, "GUI:AddCash") != 0) return UnicodeString::TheEmptyString;
        return UnicodeString(L"%d");
    }
    UnicodeString fetch(AsciiString label, Bool* exists = nullptr) override { return fetch(label.str(), exists); }
};

class CashUI : public oracle_wp51::NullInGameUI {
public:
    void addFloatingText(const UnicodeString& text, const Coord3D* pos, Color) override {
        std::string digits;
        for (int i = 0; i < text.getLength(); ++i) digits += static_cast<char>(text.getCharAt(i));
        std::ostringstream s;
        s << bits(pos->x) << ' ' << bits(pos->y) << ' ' << bits(pos->z) << ' ' << digits << ';';
        g_texts += s.str();
    }
};

// --- fixture ------------------------------------------------------------------------
static const char* const kUpgradeIni =
    "Upgrade J_UpgPlant\n  Type = OBJECT\n  BuildTime = 0.1\n  BuildCost = 100\nEnd\n"
    "Upgrade J_UpgBoost\n  Type = PLAYER\n  BuildTime = 0.1\n  BuildCost = 50\nEnd\n"
    "Upgrade J_UpgSlave\n  Type = PLAYER\n  BuildTime = 0.1\n  BuildCost = 50\nEnd\n";

static const char* const kLocoName = "J_Ground";
static const char* const kLocoIni =
    "Locomotor J_Ground\n  Surfaces = GROUND\n  Speed = 30\n  TurnRate = 180\n"
    "  Acceleration = 30\n  Braking = 30\nEnd\n";

static std::string head(const char* name, const char* kinds, int radius, int health) {
    std::ostringstream s;
    s << "Object " << name << "\n  KindOf = PRELOAD " << kinds << "\n  Geometry = CYLINDER\n"
      << "  GeometryMajorRadius = " << radius << "\n  GeometryMinorRadius = " << radius
      << "\n  GeometryHeight = 6\n  GeometryIsSmall = No\n  VisionRange = 60\n"
      << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = " << health
      << "\n    InitialHealth = " << health << "\n  End\n"
      << "  Behavior = DestroyDie ModuleTag_Die\n  End\n";
    return s.str();
}

static std::string ai(const char* module = "JoinAI") {
    return std::string("  Behavior = ") + module + " ModuleTag_AI\n  End\n"
           "  Locomotor = SET_NORMAL " + kLocoName + "\n";
}

static std::string physics() { return "  Behavior = PhysicsBehavior ModuleTag_Phys\n  End\n"; }

static std::string stealth(bool innate, int delay, const char* forbidden) {
    std::ostringstream s;
    s << "  Behavior = StealthUpdate ModuleTag_Stealth\n    InnateStealth = " << (innate ? "Yes" : "No")
      << "\n    StealthDelay = " << delay << "\n    StealthForbiddenConditions = " << forbidden
      << "\n    MoveThresholdSpeed = 3\n  End\n";
    return s.str();
}

static std::string status_upgrade(const char* trigger, const char* status) {
    return std::string("  Behavior = StatusBitsUpgrade ModuleTag_Status\n    TriggeredBy = ") +
           trigger + "\n    StatusToSet = " + status + "\n  End\n";
}

static std::string production(const char* extra) {
    return std::string("  Behavior = ProductionUpdate ModuleTag_Prod\n    NumDoorAnimations = 1\n"
                       "    DoorOpeningTime = 100\n    DoorWaitOpenTime = 200\n"
                       "    DoorCloseTime = 100\n    ConstructionCompleteDuration = 100\n") +
           extra + "  End\n";
}

static const char* const kProducible[] = {"J_Tank", "J_Inf", "J_Truck", "J_Plain", "J_Var", "J_Mob"};
static const int kProducibleCount = 6;
static const int kFactories = 4;

static std::string fixture_ini() {
    std::ostringstream s;
    const auto cost = [](int c, const char* t) {
        return "  BuildCost = " + std::to_string(c) + "\n  BuildTime = " + t + "\n";
    };
    s << head("J_Tank", "VEHICLE SCORE", 5, 100) << cost(300, "0.2") << ai() << physics()
      << stealth(true, 200, "MOVING") << status_upgrade("J_UpgBoost", "HIJACKED") << "End\n";
    s << head("J_Inf", "INFANTRY SCORE", 3, 60) << cost(100, "0.1") << ai() << physics()
      << stealth(false, 100, "NONE") << "End\n";
    s << head("J_Truck", "VEHICLE HARVESTER SCORE", 5, 120) << cost(200, "0.1") << ai("JoinTruckAI")
      << physics() << stealth(false, 100, "NONE") << "End\n";
    // Every producible unit has a StealthUpdate: the supply center exit's grant
    // dereferences it unchecked.
    s << head("J_Plain", "VEHICLE SCORE_CREATE", 4, 80) << cost(0, "0.0") << stealth(false, 100, "NONE")
      << "End\n";
    s << head("J_Var", "INFANTRY DOZER", 3, 40) << cost(90, "0.1")
      << "  BuildVariations = J_Inf J_Tank J_Plain\nEnd\n";

    // Slaves: one innately stealthed with the guard, scout and attack ranges, one that
    // only gets stealth through an upgrade.
    s << head("J_Slave", "DRONE VEHICLE SCORE", 3, 70) << ai() << physics()
      << "  Behavior = SlavedUpdate ModuleTag_Slaved\n    GuardMaxRange = 35\n"
         "    GuardWanderRange = 10\n    ScoutRange = 60\n    ScoutWanderRange = 15\n"
         "    AttackRange = 75\n    AttackWanderRange = 20\n  End\n"
      << stealth(true, 100, "NONE") << status_upgrade("J_UpgSlave", "HIJACKED") << "End\n";
    s << head("J_Slave2", "DRONE INFANTRY", 2, 50) << ai() << physics()
      << "  Behavior = SlavedUpdate ModuleTag_Slaved\n    GuardMaxRange = 20\n"
         "    GuardWanderRange = 5\n  End\n"
      << stealth(false, 100, "NONE")
      << "  Behavior = StealthUpgrade ModuleTag_Grant\n    TriggeredBy = J_UpgSlave\n  End\nEnd\n";
    s << head("J_SlaveV", "DRONE", 2, 50) << "  BuildVariations = J_Slave J_Slave2\nEnd\n";

    // A producible master whose first spawns leave through its barracks' door.
    s << head("J_Mob", "VEHICLE SCORE", 6, 200) << cost(400, "0.2") << ai() << physics()
      << stealth(false, 100, "NONE")
      << "  Behavior = SpawnBehavior ModuleTag_Spawn\n    SpawnNumber = 3\n    InitialBurst = 2\n"
         "    ExitByBudding = Yes\n    SpawnReplaceDelay = 500\n    AggregateHealth = Yes\n"
         "    SpawnTemplateName = J_SlaveV\n  End\n"
         "  Behavior = QueueProductionExitUpdate ModuleTag_Exit\n    UnitCreatePoint = X:-4 Y:3 Z:0\n"
         "    NaturalRallyPoint = X:15 Y:2 Z:0\n    ExitDelay = 100\n  End\nEnd\n";

    for (int k = 0; k < 2; ++k) {
        s << head(("J_Hive" + std::to_string(k)).c_str(), "STRUCTURE", 10, 400) << ai()
          << "  EnergyProduction = " << (k ? -2 : 0) << "\n"
          << "  Behavior = SpawnBehavior ModuleTag_Spawn\n    SpawnNumber = " << (k ? 2 : 3) << "\n"
          << "    SpawnReplaceDelay = " << (k ? 200 : 700) << "\n"
          << (k ? "    AggregateHealth = Yes\n    SpawnedRequireSpawner = Yes\n" : "")
          << "    SpawnTemplateName = J_Slave J_Slave2\n  End\n";
        if (k == 0)
            s << "  Behavior = DefaultProductionExitUpdate ModuleTag_Exit\n"
                 "    UnitCreatePoint = X:6 Y:2 Z:0\n    NaturalRallyPoint = X:30 Y:0 Z:0\n  End\n";
        else
            s << "  Behavior = QueueProductionExitUpdate ModuleTag_Exit\n"
                 "    UnitCreatePoint = X:-4 Y:3 Z:1\n    NaturalRallyPoint = X:20 Y:5 Z:0\n"
                 "    ExitDelay = 200\n  End\n";
        s << "End\n";
    }

    for (int k = 0; k < kFactories; ++k) {
        const std::string name = "J_Fac" + std::to_string(k);
        s << head(name.c_str(), k >= 2 ? "STRUCTURE SCORE FS_SUPPLY_CENTER" : "STRUCTURE FS_WARFACTORY", 12, 800)
          << "  CommandSet = J_Set\n  EnergyProduction = " << (k == 1 ? -4 : (k == 2 ? 3 : 0)) << "\n";
        s << production(k == 0 ? "    QuantityModifier = J_Inf 2\n" : (k == 1 ? "    QuantityModifier = J_Truck 3\n" : ""));
        if (k == 0)
            s << "  Behavior = DefaultProductionExitUpdate ModuleTag_Exit\n"
                 "    UnitCreatePoint = X:6 Y:2 Z:0\n    NaturalRallyPoint = X:30 Y:0 Z:0\n  End\n";
        else if (k == 1)
            s << "  Behavior = QueueProductionExitUpdate ModuleTag_Exit\n"
                 "    UnitCreatePoint = X:8 Y:-3 Z:0\n    NaturalRallyPoint = X:25 Y:4 Z:0\n"
                 "    ExitDelay = 300\n    InitialBurst = 2\n  End\n";
        else {
            s << "  Behavior = SupplyCenterProductionExitUpdate ModuleTag_Exit\n"
                 "    UnitCreatePoint = X:10 Y:5 Z:0\n    NaturalRallyPoint = X:28 Y:-6 Z:0\n"
                 "    GrantTemporaryStealth = 500\n  End\n"
                 "  Behavior = SupplyCenterDockUpdate ModuleTag_Dock\n    NumberApproachPositions = 3\n"
                 "    GrantTemporaryStealth = 900\n  End\n";
            if (k == 2) s << stealth(true, 100, "NONE");
        }
        s << "End\n";
    }

    s << head("J_Detector", "STRUCTURE", 6, 300)
      << "  Behavior = StealthDetectorUpdate ModuleTag_Detect\n    DetectionRate = 200\n"
         "    DetectionRange = 90\n  End\nEnd\n";
    s << head("J_Plant", "STRUCTURE SCORE", 10, 500) << "  EnergyProduction = 10\n  EnergyBonus = 5\n"
      << "  Behavior = PowerPlantUpdate ModuleTag_Rods\n    RodsExtendTime = 200\n  End\n"
         "  Behavior = PowerPlantUpgrade ModuleTag_Bonus\n    TriggeredBy = J_UpgPlant\n  End\n"
         "  Behavior = AutoDepositUpdate ModuleTag_Cash\n    DepositTiming = 300\n"
         "    DepositAmount = 10\n    InitialCaptureBonus = 100\n"
         "    UpgradedBoost = UpgradeType:J_UpgBoost Boost:5\n  End\nEnd\n";
    s << head("J_Tech", "STRUCTURE", 8, 400) << "  EnergyProduction = -3\n"
      << "  Behavior = AutoDepositUpdate ModuleTag_Cash\n    DepositTiming = 500\n"
         "    DepositAmount = 25\n    InitialCaptureBonus = 200\n  End\nEnd\n";
    return s.str();
}

static const char* const kExtraSpawn[] = {"J_Plant", "J_Tech", "J_Detector", "J_Truck", "J_Tank", "J_Hive1"};
static const int kExtraCount = 6;
static const char* const kUpgradeNames[] = {"J_UpgPlant", "J_UpgBoost", "J_UpgSlave"};
static const DisabledType kDisabled[] = {DISABLED_EMP, DISABLED_HACKED, DISABLED_PARALYZED};

static std::string commands_ini() {
    std::string s;
    for (const char* u : kProducible)
        s += std::string("CommandButton J_B_") + u + "\n  Command = UNIT_BUILD\n  Object = " + u + "\nEnd\n";
    s += "CommandSet J_Set\n";
    int n = 1;
    for (const char* u : kProducible) s += "  " + std::to_string(n++) + " = J_B_" + u + "\n";
    return s + "End\n";
}

// --- observation --------------------------------------------------------------------
// Full text for the first vectors (and with JOINS_HEX), a hash otherwise.
static bool g_hex = false;
static bool g_full = false;
static const int kFullVectors = 2;

static std::string digest(const std::string& s) {
    uint32_t h = 2166136261u;
    for (char c : s) { h ^= static_cast<uint8_t>(c); h *= 16777619u; }
    char buf[9];
    std::snprintf(buf, sizeof buf, "%08x", h);
    return buf;
}

static std::vector<Object*> world_objects() {
    std::vector<Object*> out;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) out.push_back(o);
    std::sort(out.begin(), out.end(), [](Object* a, Object* b) { return a->getID() < b->getID(); });
    return out;
}

static int team_code(const Object* o) {
    return o->getControllingPlayer() ? o->getControllingPlayer()->getPlayerIndex() : -1;
}

static std::string object_row(Object* o) {
    std::ostringstream s;
    uint64_t status = 0;
    const ObjectStatusMaskType mask = o->getStatusBits();
    for (int b = 0; b < OBJECT_STATUS_COUNT; ++b)
        if (mask.test(b)) status |= uint64_t(1) << b;
    uint32_t disabled = 0;
    for (int b = 0; b < DISABLED_COUNT; ++b)
        if (o->getDisabledFlags().test(b)) disabled |= 1u << b;
    const Coord3D* p = o->getPosition();
    s << o->getTemplate()->getName().str() << ' ' << team_code(o) << ' ' << status << ' ' << disabled
      << ' ' << (o->isEffectivelyDead() ? 1 : 0) << ' ' << bits(p->x) << ' ' << bits(p->y) << ' '
      << bits(p->z) << ' ' << bits(o->getOrientation()) << ' '
      << bits(o->getBodyModule()->getHealth()) << ' ' << static_cast<int>(o->getProducerID()) << " [";
    const ModelConditionFlags& f = o->getDrawable()->getModelConditionFlags();
    for (int b = 0; b < MODELCONDITION_COUNT; ++b)
        if (f.test(b)) s << b << ',';
    s << ']';
    for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m) {
        const std::string tag = TheNameKeyGenerator->keyToName((*m)->getModuleTagNameKey()).str();
        if (tag.compare(0, 10, "ModuleTag_") != 0 || tag == "ModuleTag_AI") continue;
        if (tag.size() > 16 && tag.compare(tag.size() - 6, 6, "Helper") == 0) continue;
        const std::string save = oracle_wp51::World::snapshot_save_hex(*m);
        s << ' ' << tag.substr(10) << '=' << (g_hex ? save : digest(save)) << '/'
          << oracle_wp51::World::snapshot_crc(*m);
        if (SpawnBehavior* sb = dynamic_cast<SpawnBehavior*>(*m)) s << '+' << M(sb, ABurst);
    }
    if (AIUpdateInterface* ai_ = o->getAIUpdateInterface()) {
        if (JoinTruck* t = dynamic_cast<JoinTruck*>(ai_)) s << " truck " << t->m_boxes << (t->m_wanting ? 'w' : '-');
    }
    return s.str();
}

static std::string players_row() {
    std::ostringstream s;
    for (int i = 0; i < 3; ++i) {
        Player* p = ThePlayerList->getNthPlayer(i);
        ScoreKeeper* sk = p->getScoreKeeper();
        AcademyStats* a = p->getAcademyStats();
        s << p->getMoney()->countMoney() << ',' << M(p->getEnergy(), PEProd) << ',' << M(p->getEnergy(), PECons)
          << ',' << M(sk, SEarned) << ',' << M(sk, SSpent) << ',' << sk->getTotalUnitsBuilt() << ','
          << sk->getTotalBuildingsBuilt() << ',' << M(sk, STech) << ',' << M(sk, SFaction) << ','
          << M(a, ALastIncome) << ',' << M(a, AMaxIncome) << ',' << M(a, ASupply) << ',' << M(a, APeons)
          << ',' << M(a, AGather) << ',' << M(a, ALastUnit) << ';';
    }
    return s.str();
}

// --- the world loop -----------------------------------------------------------------
static void sift_down_root(std::vector<UpdateModulePtr>& heap) {
    const Int sz = static_cast<Int>(heap.size());
    Int i = 0;
    while (true) {
        Int child = 2 * i + 1;
        if (child >= sz) break;
        if (child + 1 < sz && heap[child]->friend_getPriority() > heap[child + 1]->friend_getPriority())
            ++child;
        if (!(heap[i]->friend_getPriority() > heap[child]->friend_getPriority())) break;
        std::swap(heap[i], heap[child]);
        heap[i]->friend_setIndexInLogic(i);
        heap[child]->friend_setIndexInLogic(child);
        i = child;
    }
}

// The sleepy loop, partition update, destroy list and frame step of `GameLogic::update`.
static void run_frame(oracle_wp51::World& w, UnsignedInt& frame) {
    w.set_frame(frame);
    auto& heap = TheGameLogic->*access_member(GHeap());
    auto& cur = TheGameLogic->*access_member(GCur());
    int guard = 0;
    while (!heap.empty()) {
        UpdateModulePtr u = heap.front();
        if (u->friend_getNextCallFrame() > frame) break;
        if (++guard > 4096) die("sleepy update guard exceeded");
        UpdateSleepTime sleep_len = UPDATE_SLEEP_NONE;
        const DisabledMaskType dis = u->friend_getObject()->getDisabledFlags();
        if (u->getDisabledTypesToProcess().testForAll(dis)) {
            cur = u;
            sleep_len = u->update();
            if (sleep_len < 1) sleep_len = UPDATE_SLEEP_NONE;
            cur = nullptr;
        }
        u->friend_setNextCallFrame(frame + sleep_len);
        sift_down_root(heap);
    }
    ThePartitionManager->update();
    TheGameLogic->processDestroyList();
    ++frame;
    w.set_frame(frame);
}

struct Op { uint32_t code, a, b, c; };

struct Scene {
    oracle_wp51::World& w;
    UnsignedInt frame = 1;
    std::map<int, std::string> last;
    std::string last_players;

    Team* team(uint32_t t) const { return t == 0 ? w.team_neutral : (t == 1 ? w.team_a : w.team_b); }

    std::vector<Object*> live() const {
        std::vector<Object*> out;
        for (Object* o : world_objects())
            if (!o->isDestroyed()) out.push_back(o);
        return out;
    }

    std::vector<Object*> live_of(const char* what) const {
        std::vector<Object*> out;
        for (Object* o : live()) {
            const std::string w_ = what;
            if (w_ == "factory") {
                if (o->getProductionUpdateInterface()) out.push_back(o);
            } else if (w_ == "exit") {
                if (o->getObjectExitInterface()) out.push_back(o);
            } else if (w_ == "truck") {
                if (o->getAIUpdateInterface() && o->getAIUpdateInterface()->getSupplyTruckAIInterface()) out.push_back(o);
            } else if (w_ == "dock") {
                if (o->findUpdateModule(NAMEKEY("SupplyCenterDockUpdate"))) out.push_back(o);
            } else if (w_ == "ai") {
                if (o->getAIUpdateInterface()) out.push_back(o);
            } else if (w_ == "physics") {
                if (o->getPhysics()) out.push_back(o);
            } else if (w_ == "plant") {
                if (o->getTemplate()->getName() == "J_Plant") out.push_back(o);
            }
        }
        return out;
    }

    // One row: the return value, changed objects, removed ids, players, orders, paths,
    // floating texts and the logic seed.
    std::string row(int ret) {
        std::ostringstream s;
        s << ret << ' ' << frame << '@';
        std::map<int, std::string> now;
        for (Object* o : world_objects()) now[static_cast<int>(o->getID())] = object_row(o);
        const auto text = [](const std::string& t) {
            return g_full || t.size() <= 24 ? t : "#" + digest(t);
        };
        for (const auto& kv : now) {
            auto it = last.find(kv.first);
            if (it == last.end() || it->second != kv.second) s << kv.first << ':' << text(kv.second) << '^';
        }
        for (const auto& kv : last)
            if (!now.count(kv.first)) s << '-' << kv.first << '^';
        last = now;
        const std::string players = players_row();
        s << '@' << (players == last_players ? std::string("=") : text(players));
        last_players = players;
        s << '@' << text(g_orders) << '@' << text(g_paths) << '@' << text(g_prone) << '@'
          << text(g_texts) << '@' << GetGameLogicRandomSeedCRC();
        g_orders.clear();
        g_paths.clear();
        g_texts.clear();
        g_prone.clear();
        return s.str();
    }

    Object* spawn(const char* name, uint32_t t, float x, float y, float angle) {
        Coord3D c;
        c.x = x;
        c.y = y;
        c.z = 0.0f;
        Object* o = w.spawn_object(name, team(t), c, angle);
        if (!o) die("spawn failed");
        return o;
    }

    int apply(const Op& op) {
        w.set_frame(frame);
        const auto choose = [](const std::vector<Object*>& v, uint32_t a) -> Object* {
            return v.empty() ? nullptr : v[a % v.size()];
        };
        switch (op.code) {
        case 0:
            for (uint32_t i = 0; i < op.a; ++i) run_frame(w, frame);
            return 0;
        case 1: {
            Object* f = choose(live_of("factory"), op.a);
            if (!f) return -1;
            ProductionUpdateInterface* pu = f->getProductionUpdateInterface();
            const ProductionID id = pu->requestUniqueUnitID();
            return pu->queueCreateUnit(TheThingFactory->findTemplate(kProducible[op.b]), id);
        }
        case 2: {
            Object* f = choose(live_of("factory"), op.a);
            if (!f) return -1;
            f->getProductionUpdateInterface()->cancelUnitCreate(static_cast<ProductionID>(op.b));
            return 0;
        }
        case 3: M(ThePlayerList->getNthPlayer(op.a)->getMoney(), PMoney) = op.b; return 0;
        case 4: {
            Object* f = choose(live_of("exit"), op.a);
            if (!f) return -1;
            Coord3D c;
            c.x = static_cast<float>(op.b);
            c.y = static_cast<float>(op.c);
            c.z = 0.0f;
            f->getObjectExitInterface()->setRallyPoint(&c);
            return 0;
        }
        case 5: {  // load a truck and dock it
            Object* d = choose(live_of("dock"), op.a);
            Object* t = choose(live_of("truck"), op.b);
            if (!d || !t) return -1;
            static_cast<JoinTruck*>(t->getAIUpdateInterface())->m_boxes = static_cast<Int>(op.c);
            DockUpdate* du = static_cast<DockUpdate*>(d->findUpdateModule(NAMEKEY("SupplyCenterDockUpdate")));
            return du->getDockUpdateInterface()->action(t, nullptr);
        }
        case 6: {
            Object* o = choose(live(), op.a);
            if (!o) return -1;
            Coord3D c;
            c.x = static_cast<float>(op.b);
            c.y = static_cast<float>(op.c);
            c.z = 0.0f;
            o->setPosition(&c);
            return static_cast<int>(o->getID());
        }
        case 7: {
            Object* o = choose(live_of("physics"), op.a);
            if (!o) return -1;
            Coord3D f;
            f.x = (static_cast<int>(op.b) - 8) * 0.5f;
            f.y = (static_cast<int>(op.c) - 8) * 0.5f;
            f.z = (op.b % 3) * 1.0f;
            o->getPhysics()->applyForce(&f);
            return static_cast<int>(o->getID());
        }
        case 8: {
            Object* o = choose(live(), op.a);
            if (!o) return -1;
            const int id = static_cast<int>(o->getID());
            TheGameLogic->destroyObject(o);
            return id;
        }
        case 9: {
            Object* o = choose(live(), op.a);
            if (!o) return -1;
            DamageInfo d;
            d.in.m_damageType = DAMAGE_UNRESISTABLE;
            d.in.m_deathType = DEATH_NORMAL;
            d.in.m_amount = static_cast<float>(op.b);
            o->attemptDamage(&d);
            return static_cast<int>(o->getID());
        }
        case 10: {
            Object* o = choose(live(), op.a);
            if (!o) return -1;
            if (op.c) o->setDisabled(kDisabled[op.b]);
            else o->clearDisabled(kDisabled[op.b]);
            return static_cast<int>(o->getID());
        }
        case 11: {
            Object* o = choose(live(), op.a);
            if (!o) return -1;
            o->setTeam(team(op.b));
            return static_cast<int>(o->getID());
        }
        case 12: {
            Object* o = choose(live_of("plant"), op.a);
            if (!o) return -1;
            const UpgradeTemplate* u = TheUpgradeCenter->findUpgrade(kUpgradeNames[0]);
            if (o->hasUpgrade(u)) return 0;
            o->giveUpgrade(u);
            return static_cast<int>(o->getID());
        }
        case 13: {
            Player* p = ThePlayerList->getNthPlayer(op.a);
            p->addUpgrade(TheUpgradeCenter->findUpgrade(kUpgradeNames[op.b]), UPGRADE_STATUS_COMPLETE);
            return 0;
        }
        case 14: {
            Object* o = choose(live_of("ai"), op.a);
            if (!o) return -1;
            AIUpdateInterface* ai_ = o->getAIUpdateInterface();
            const CommandSourceType src = op.c ? CMD_FROM_PLAYER : CMD_FROM_AI;
            if (JoinTruck* t = dynamic_cast<JoinTruck*>(ai_)) t->m_idle = op.b, t->m_source = src;
            else static_cast<JoinAI*>(ai_)->m_idle = op.b, static_cast<JoinAI*>(ai_)->m_source = src;
            return static_cast<int>(o->getID());
        }
        case 15: {
            Object* o = spawn(kExtraSpawn[op.a], op.b, 250.0f + (op.c % 100), 250.0f + (op.c / 100), 0.0f);
            return static_cast<int>(o->getID());
        }
        default: die("bad op"); return 0;
        }
    }

    Op random_op() {
        Op op = {0, 0, 0, 0};
        const uint32_t k = pick(100);
        if (k < 30) op = {0, 1 + (pick(4) ? pick(12) : pick(60)), 0, 0};
        else if (k < 48) op = {1, pick(4) ? 0 : pick(6), pick(kProducibleCount), 0};
        else if (k < 51) op = {2, pick(4), pick(5), 0};
        else if (k < 53) op = {3, 1 + pick(2), pick(3) ? 5000 : pick(300), 0};
        else if (k < 57) op = {4, pick(6), 200 + pick(200), 200 + pick(200)};
        else if (k < 66) op = {5, pick(3), pick(8), pick(6)};
        else if (k < 70) op = {6, pick(24), 270 + pick(120), 270 + pick(120)};
        else if (k < 74) op = {7, pick(16), pick(17), pick(17)};
        else if (k < 77) op = {8, pick(24), 0, 0};
        else if (k < 82) op = {9, pick(24), pick(3) ? 25 + pick(60) : 2000, 0};
        else if (k < 85) op = {10, pick(24), pick(3), pick(3) ? 1u : 0u};
        else if (k < 91) op = {11, pick(2) ? pick(6) : pick(24), pick(3), 0};
        else if (k < 93) op = {12, pick(3), 0, 0};
        else if (k < 95) op = {13, 1 + pick(2), 1 + pick(2), 0};
        else if (k < 98) op = {14, pick(16), pick(2), pick(2)};
        else op = {15, pick(kExtraCount), pick(3), pick(10000)};
        return op;
    }
};

static void reset_players() {
    for (int i = 0; i < 3; ++i) {
        Player* p = ThePlayerList->getNthPlayer(i);
        p->getMoney()->setPlayerIndex(i);
        M(p->getMoney(), PMoney) = 5000;
        M(p->getEnergy(), PEProd) = 0;
        M(p->getEnergy(), PECons) = 0;
        M(p->getEnergy(), PESab) = 0;
        p->getScoreKeeper()->reset(i);
        p->getAcademyStats()->init(p);
        for (const char* n : kUpgradeNames) {
            const UpgradeTemplate* u = TheUpgradeCenter->findUpgrade(n);
            if (p->hasUpgradeComplete(u)) p->removeUpgrade(u);
        }
    }
}

static std::string escape(const std::string& s) {
    std::string out;
    for (char c : s) out += c == '\n' ? '~' : c;
    return out;
}

int main(int argc, char** argv) {
    if (argc < 2) die("usage: gen_joins_modules <out.json>");
    g_hex = std::getenv("JOINS_HEX") != nullptr;
    oracle_wp51::World w;
    TheInGameUI = new CashUI;
    TheGameText = new CashText;
    (TheModuleFactory->*(&ExposedFactory::addModuleInternal))(
        make_ai, AIUpdateInterface::friend_newModuleData, MODULETYPE_BEHAVIOR, "JoinAI",
        AIUpdateInterface::getInterfaceMask());
    (TheModuleFactory->*(&ExposedFactory::addModuleInternal))(
        make_truck, SupplyTruckAIUpdate::friend_newModuleData, MODULETYPE_BEHAVIOR, "JoinTruckAI",
        SupplyTruckAIUpdate::getInterfaceMask());
    TheWritableGlobalData->m_baseValuePerSupplyBox = 75;
    w.load_ini(kUpgradeIni);
    w.load_ini(kLocoIni);
    w.load_ini(fixture_ini());
    w.mem_ini.parse_blocks(commands_ini(), INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
        if (std::strcmp(tok, "CommandButton") == 0) INI::parseCommandButtonDefinition(ini);
        else if (std::strcmp(tok, "CommandSet") == 0) INI::parseCommandSetDefinition(ini);
    });
    TheThingFactory->postProcessLoad();
    w.init_with_neutral_player();
    // A local player that never looks, so shroud edges stay out of the client.
    ThePlayerList->setLocalPlayer(ThePlayerList->getNthPlayer(5));

    std::FILE* out = std::fopen(argv[1], "wb");
    if (!out) die("cannot open output");
    std::fprintf(out, "{\"upgrades\":\"%s\",\n\"locomotor\":\"%s\",\n\"fixture\":\"%s\",\n\"vectors\":[\n",
                 escape(kUpgradeIni).c_str(), escape(kLocoIni).c_str(), escape(fixture_ini()).c_str());

    const int kVectors = 1000;
    int op_done[16] = {};
    int op_skipped[16] = {};
    long long total_rows = 0;
    for (int v = 0; v < kVectors; ++v) {
        Scene s{w};
        g_full = g_hex || v < kFullVectors;
        const uint32_t seed = 1 + pick(1000000);
        const uint32_t factory = static_cast<uint32_t>(v) % kFactories;
        const uint32_t hive = pick(2);
        InitRandom(seed);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        reset_players();
        w.set_frame(1);
        g_orders.clear();
        g_paths.clear();
        g_texts.clear();
        g_prone.clear();

        // Factory, hive, plant, tech building and detector, close enough to interact.
        std::vector<uint32_t> cfg = {seed, factory, hive, 250 + pick(40), 250 + pick(40), pick(8),
                                     pick(120), pick(120)};
        static const float kAngles[] = {0.0f, 1.5707964f, 3.1415927f, -1.5707964f, 0.5f, -2.75f, 6.0f, 0.001f};
        const float x = static_cast<float>(cfg[3]), y = static_cast<float>(cfg[4]);
        s.spawn(("J_Fac" + std::to_string(factory)).c_str(), 1, x, y, kAngles[cfg[5]]);
        s.spawn("J_Detector", 2, x + 20.0f + cfg[6], y - 40.0f + cfg[7], 0.0f);
        s.spawn(("J_Hive" + std::to_string(hive)).c_str(), 1, x - 40.0f, y + 30.0f, 0.5f);
        s.spawn("J_Plant", 1, x + 5.0f, y + 70.0f, 0.0f);
        s.spawn("J_Tech", 0, x - 70.0f, y - 20.0f, 0.0f);
        s.spawn("J_Truck", 1, x + 40.0f, y + 10.0f, 0.0f);

        std::vector<std::string> rows;
        rows.push_back(s.row(0));
        std::vector<Op> ops;
        const int n = 14 + pick(14);
        for (int i = 0; i < n; ++i) {
            Op op = s.random_op();
            // Open with some production so that units exist early.
            if (i == 0) op = {1, 0, pick(kProducibleCount), 0};
            if (i == 1) op = {1, 0, pick(3), 0};
            if (i == 2) op = {0, 5 + pick(30), 0, 0};
            const int ret = s.apply(op);
            ops.push_back(op);
            (ret == -1 ? op_skipped : op_done)[op.code]++;
            rows.push_back(s.row(ret));
        }

        std::fprintf(out, "%s{\"v\":%d,\"cfg\":[", v ? ",\n" : "", v);
        for (size_t i = 0; i < cfg.size(); ++i) std::fprintf(out, "%s%u", i ? "," : "", cfg[i]);
        std::fprintf(out, "],\"ops\":[");
        for (size_t i = 0; i < ops.size(); ++i)
            std::fprintf(out, "%s%u,%u,%u,%u", i ? "," : "", ops[i].code, ops[i].a, ops[i].b, ops[i].c);
        std::fprintf(out, "],\"rows\":[\"");
        for (size_t i = 0; i < rows.size(); ++i)
            std::fprintf(out, "%s%s", i ? "\",\"" : "", rows[i].c_str());
        std::fprintf(out, "\"]}");
        total_rows += static_cast<long long>(rows.size());

        for (int pass = 0; pass < 4; ++pass) {
            for (Object* o : world_objects())
                if (!o->isDestroyed()) TheGameLogic->destroyObject(o);
            TheGameLogic->processDestroyList();
        }
        if (TheGameLogic->getFirstObject() || !(TheGameLogic->*access_member(GHeap())).empty())
            die("world not empty after a vector");
        ThePartitionManager->update();
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    std::printf("rows %lld\n", total_rows);
    for (int i = 0; i < 16; ++i) std::printf("op %d: %d done, %d skipped\n", i, op_done[i], op_skipped[i]);
    return 0;
}
