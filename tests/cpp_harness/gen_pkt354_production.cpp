// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for ProductionUpdate (PKT-354), run on the unmodified reference module
// with the reference BuildAssistant, Player, UpgradeCenter and ThingFactory.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt354_production.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT354_ProductionUpdate.json
//
// The exit interface, the parking place and the unit's create module are stand-ins defined
// here, so that the door answers can be scripted; the Rust test defines the same three.
//
// Not covered: audio, UI and radar feedback; a producer without a controlling player; a
// producer without a drawable; the special power button (canMakeUnit returns early for it,
// exercised by the Rust test); superweapon restriction; Object::defect and selling, which
// call cancelAndRefundAllProduction from outside; cancelUnitCreate(0) with an upgrade at
// the head, which reads the upgrade pointer as a thing template; loading corrupt saves.

#include "oracle_wp51_world.h"

#include "Common/AcademyStats.h"
#include "Common/Energy.h"
#include "Common/Handicap.h"
#include "Common/Money.h"
#include "Common/ScoreKeeper.h"
#include "GameLogic/Module/CreateModule.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/ProductionUpdate.h"

#include <cstdio>
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
ACCESS(PType, Player, PlayerType, m_playerType)
ACCESS(PSci, Player, ScienceVec, m_sciences)
ACCESS(PCost, Player, ProductionChangeMap, m_productionCostChanges)
ACCESS(PTime, Player, ProductionChangeMap, m_productionTimeChanges)
ACCESS(PInProg, Player, UpgradeMaskType, m_upgradesInProgress)
ACCESS(ASupply, AcademyStats, UnsignedInt, m_supplyCentersBuilt)
ACCESS(ARadar, AcademyStats, Bool, m_researchedRadar)
ACCESS(APeons, AcademyStats, UnsignedInt, m_peonsBuilt)
ACCESS(AIdle, AcademyStats, UnsignedInt, m_idleBuildingUnitsMaxFrames)
ACCESS(ALastUnit, AcademyStats, UnsignedInt, m_lastUnitBuiltFrame)
ACCESS(ABought, AcademyStats, UnsignedInt, m_upgradesPurchased)
ACCESS(AGather, AcademyStats, UnsignedInt, m_gatherersBuilt)
ACCESS(AHeroes, AcademyStats, UnsignedInt, m_heroesBuilt)
ACCESS(ALastIncome, AcademyStats, UnsignedInt, m_lastIncomeFrame)
ACCESS(AMaxIncome, AcademyStats, UnsignedInt, m_maxFramesBetweenIncome)
struct SDone { using type = ListAsciiStringObjectID (ScriptEngine::*)[MAX_PLAYER_COUNT]; };
template struct MemberAccess<SDone, &ScriptEngine::m_completedUpgrades>;
inline SDone::type access_member(SDone);

// ---- the scripted stand-ins ----------------------------------------------------------
static int g_door = 0;        // answer of reserveDoorForExit
static int g_budget = -1;     // reservations left before the answer is "none"; -1: no limit
static int g_hold = -9;       // door to hold from inside exitObjectViaDoor; -9: do not
static int g_hold_flag = 0;
static int g_park_reserve = 0;
static int g_park_space = 1;
static std::string g_log;
static Object* g_factory = nullptr;
static Team* g_team = nullptr;

static std::string tname(const ThingTemplate* t) { return std::string(t->getName().str()); }

class ScriptedExit : public BehaviorModule, public ExitInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(ScriptedExit, "ScriptedExit", 32, 32)
public:
    ScriptedExit(Thing* t, const ModuleData* d) : BehaviorModule(t, d) {}
    static Module* make(Thing* t, const ModuleData* d) { return newInstance(ScriptedExit)(t, d); }
    NameKeyType getModuleNameKey() const override { return NAMEKEY("ScriptedExit"); }
    ExitInterface* getUpdateExitInterface() override { return this; }
    Bool isExitBusy() const override { return FALSE; }
    ExitDoorType reserveDoorForExit(const ThingTemplate* t, Object*) override {
        g_log += "R" + tname(t) + ";";
        if (g_budget == 0) return DOOR_NONE_AVAILABLE;
        if (g_budget > 0) --g_budget;
        return static_cast<ExitDoorType>(g_door);
    }
    void unreserveDoorForExit(ExitDoorType d) override { g_log += "U" + std::to_string((int)d) + ";"; }
    void exitObjectViaDoor(Object* o, ExitDoorType d) override {
        g_log += "X" + std::to_string((int)d) + "," + tname(o->getTemplate()) + "," +
                 std::to_string(o->getProducerID() == getObject()->getID()) +
                 std::to_string(o->getTeam() == g_team) + ";";
        if (g_hold != -9) {
            ProductionUpdateInterface* pu = getObject()->getProductionUpdateInterface();
            if (pu) pu->setHoldDoorOpen(static_cast<ExitDoorType>(g_hold), g_hold_flag != 0);
        }
    }
    void exitObjectByBudding(Object*, Object*) override {}
    void setRallyPoint(const Coord3D*) override {}
    const Coord3D* getRallyPoint() const override { return nullptr; }

protected:
    void crc(Xfer*) override {}
    void xfer(Xfer*) override {}
    void loadPostProcess() override {}
};
ScriptedExit::~ScriptedExit() {}

class ScriptedParking : public BehaviorModule, public ParkingPlaceBehaviorInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(ScriptedParking, "ScriptedParking", 32, 32)
public:
    ScriptedParking(Thing* t, const ModuleData* d) : BehaviorModule(t, d) {}
    static Module* make(Thing* t, const ModuleData* d) { return newInstance(ScriptedParking)(t, d); }
    NameKeyType getModuleNameKey() const override { return NAMEKEY("ScriptedParking"); }
    ParkingPlaceBehaviorInterface* getParkingPlaceBehaviorInterface() override { return this; }
    Bool shouldReserveDoorWhenQueued(const ThingTemplate*) const override { return g_park_reserve != 0; }
    Bool hasAvailableSpaceFor(const ThingTemplate*) const override { return g_park_space != 0; }
    Bool hasReservedSpace(ObjectID) const override { return FALSE; }
    Int getSpaceIndex(ObjectID) const override { return -1; }
    Bool reserveSpace(ObjectID, Real, PPInfo*) override { return FALSE; }
    void releaseSpace(ObjectID) override {}
    Bool reserveRunway(ObjectID, Bool) override { return FALSE; }
    void calcPPInfo(ObjectID, PPInfo*) override {}
    void releaseRunway(ObjectID) override {}
    Int getRunwayIndex(ObjectID) override { return -1; }
    Int getRunwayCount() const override { return 0; }
    ObjectID getRunwayReservation(Int, RunwayReservationType) override { return INVALID_ID; }
    void transferRunwayReservationToNextInLineForTakeoff(ObjectID) override {}
    Real getApproachHeight() const override { return 0.0f; }
    Real getLandingDeckHeightOffset() const override { return 0.0f; }
    void setHealee(Object*, Bool) override {}
    void killAllParkedUnits() override {}
    void defectAllParkedUnits(Team*, UnsignedInt) override {}
    Bool calcBestParkingAssignment(ObjectID, Coord3D*, Int*, Int*) override { return FALSE; }
    const std::vector<Coord3D>* getTaxiLocations(ObjectID) const override { return nullptr; }
    const std::vector<Coord3D>* getCreationLocations(ObjectID) const override { return nullptr; }

protected:
    void crc(Xfer*) override {}
    void xfer(Xfer*) override {}
    void loadPostProcess() override {}
};
ScriptedParking::~ScriptedParking() {}

// Logs onBuildComplete with the status the unit's upgrade module sets, so the order against
// the exit call and the upgrades of a new object are both visible.
class ScriptedBuilt : public BehaviorModule, public CreateModuleInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(ScriptedBuilt, "ScriptedBuilt", 64, 64)
public:
    ScriptedBuilt(Thing* t, const ModuleData* d) : BehaviorModule(t, d) {}
    static Module* make(Thing* t, const ModuleData* d) { return newInstance(ScriptedBuilt)(t, d); }
    NameKeyType getModuleNameKey() const override { return NAMEKEY("ScriptedBuilt"); }
    CreateModuleInterface* getCreate() override { return this; }
    void onCreate() override {}
    void onBuildComplete() override {
        g_log += "B" + tname(getObject()->getTemplate()) + "," +
                 std::to_string(getObject()->getStatusBits().test(OBJECT_STATUS_HIJACKED) ? 1 : 0) + ";";
    }
    Bool shouldDoOnBuildComplete() const override { return TRUE; }

protected:
    void crc(Xfer*) override {}
    void xfer(Xfer*) override {}
    void loadPostProcess() override {}
};
ScriptedBuilt::~ScriptedBuilt() {}

static ModuleData* make_data(INI* ini) {
    ModuleData* d = new ModuleData;
    if (ini) {
        static const FieldParse none[] = {{nullptr, nullptr, nullptr, 0}};
        ini->initFromINI(d, none);
    }
    return d;
}

struct ModuleFactoryAccess : ModuleFactory {
    using ModuleFactory::addModuleInternal;
};

static void add_module(NewModuleProc proc, const char* name, Int mask) {
    auto add = &ModuleFactoryAccess::addModuleInternal;
    (TheModuleFactory->*add)(proc, make_data, MODULETYPE_BEHAVIOR, name, mask);
}

// ---- fixture ---------------------------------------------------------------------------
static const int kUpgrades = 5;
static const char* const kUpgradeNames[kUpgrades] = {"UpgP1", "UpgP2", "UpgO1", "UpgO2", "UpgO3"};

static const int kUnits = 12;
static const char* const kUnitNames[kUnits] = {"U0", "U1", "U2", "U3", "U4", "U5",
                                               "U6", "U7", "U8", "U9", "UE", "UX"};

// An `InactiveBody` object is effectively dead, which `canBuildMoreOfType` skips.
static const char* const kLive = "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n  End\n";
static const char* const kDead = "  Body = InactiveBody ModuleTag_Body\n  End\n";

static std::string unit_ini(const char* name, const std::string& body) {
    const std::string n = name;
    return "Object " + n + "\n" + body + (n == "U1" || n == "U3" ? kDead : kLive) +
           "  Behavior = StatusBitsUpgrade ModuleTag_Upg\n    TriggeredBy = UpgP1\n"
           "    StatusToSet = HIJACKED\n  End\n"
           "  Behavior = ScriptedBuilt ModuleTag_Built\n  End\nEnd\n";
}

static std::string upgrades_ini() {
    return "Upgrade UpgP1\n  Type = PLAYER\n  BuildTime = 0.2\n  BuildCost = 300\nEnd\n"
           "Upgrade UpgP2\n  Type = PLAYER\n  BuildTime = 0.0\n  BuildCost = 50\n"
           "  AcademyClassify = ACT_UPGRADE_RADAR\nEnd\n"
           "Upgrade UpgO1\n  Type = OBJECT\n  BuildTime = 0.1\n  BuildCost = 400\nEnd\n"
           "Upgrade UpgO2\n  Type = OBJECT\n  BuildTime = 0.3\n  BuildCost = 100\nEnd\n"
           "Upgrade UpgO3\n  Type = OBJECT\n  BuildTime = 0.1\n  BuildCost = 0\nEnd\n";
}

static std::string templates_ini() {
    std::string s = "Science SCIENCE_P354\nEnd\n";
    s += "Object P_Pre\n  KindOf = STRUCTURE\n  Body = InactiveBody ModuleTag_Body\n  End\nEnd\n";
    s += unit_ini("U0", "  KindOf = VEHICLE SCORE\n  BuildCost = 500\n  BuildTime = 0.2\n");
    s += unit_ini("U1", "  KindOf = INFANTRY\n  BuildCost = 150\n  BuildTime = 0.0\n");
    s += unit_ini("U2", "  KindOf = INFANTRY SCORE_CREATE HERO\n  BuildCost = 70\n  BuildTime = 0.3\n"
                        "  Prerequisites\n    Object = P_Pre\n  End\n");
    s += unit_ini("U3", "  KindOf = DOZER VEHICLE\n  BuildCost = 0\n  BuildTime = 0.1\n");
    s += unit_ini("U4", "  KindOf = VEHICLE HARVESTER\n  BuildCost = 60\n  BuildTime = 0.1\n"
                        "  MaxSimultaneousOfType = 3\n  MaxSimultaneousLinkKey = P354Link\n");
    s += unit_ini("UL", "  KindOf = VEHICLE\n  MaxSimultaneousLinkKey = P354Link\n");
    s += unit_ini("U5", "  KindOf = INFANTRY\n  BuildCost = 90\n  BuildTime = 0.1\n"
                        "  BuildVariations = U0 U1 U3\n  BuildCompletion = PLACED_BY_PLAYER\n");
    s += unit_ini("U6", "  KindOf = INFANTRY\n  BuildCost = 33\n  BuildTime = 0.4\n"
                        "  Prerequisites\n    Science = SCIENCE_P354\n    Object = P_Pre U0\n  End\n");
    s += unit_ini("U7", "  KindOf = VEHICLE SCORE\n  BuildCost = 40\n  BuildTime = 0.1\n"
                        "  Buildable = Only_By_AI\n");
    s += unit_ini("U8", "  KindOf = STRUCTURE\n  BuildCost = 1000\n  BuildTime = 0.2\n"
                        "  Buildable = Ignore_Prerequisites\n  MaxSimultaneousOfType = 1\n"
                        "  Prerequisites\n    Object = P_Pre\n  End\n");
    s += unit_ini("U9", "  KindOf = INFANTRY\n  BuildCost = 20\n  BuildTime = 0.1\n"
                        "  Buildable = No\n  BuildVariations = UE\n");
    s += unit_ini("UE", "  KindOf = INFANTRY\n  BuildCost = 25\n  BuildTime = 0.1\n"
                        "  MaxSimultaneousOfType = DeterminedBySuperweaponRestriction\n");
    s += unit_ini("UX", "  KindOf = INFANTRY\n  BuildCost = 25\n  BuildTime = 0.1\n");
    return s;
}

// name, door animations, opening, waiting, closing, complete duration (ms), queue limit,
// has exit, has parking, quantity modifiers, disabled types
struct Variant {
    const char* body;
    int has_exit;
    int has_parking;
};
static const Variant kVariants[] = {
    {"", 1, 0},
    {"NumDoorAnimations = 1\nDoorOpeningTime = 100\nDoorWaitOpenTime = 200\nDoorCloseTime = 100\n"
     "ConstructionCompleteDuration = 100\n", 1, 0},
    {"NumDoorAnimations = 4\nDoorOpeningTime = 0\nDoorWaitOpenTime = 0\nDoorCloseTime = 300\n"
     "QuantityModifier = U1\n", 1, 0},
    {"NumDoorAnimations = 2\nDoorOpeningTime = 66\nDoorWaitOpenTime = 33\nDoorCloseTime = 400\n"
     "QuantityModifier = U1 3\nQuantityModifier = U1 5\nQuantityModifier = U0 2\n"
     "ConstructionCompleteDuration = 300\n", 1, 0},
    {"MaxQueueEntries = 2\nQuantityModifier = U5 2\nQuantityModifier = U3\n"
     "QuantityModifier = NoSuchThing 7\nDisabledTypesToProcess = DISABLED_EMP\n", 1, 0},
    {"MaxQueueEntries = -1\nQuantityModifier = U1 0\nQuantityModifier = U2 -1\n"
     "NumDoorAnimations = -1\nDisabledTypesToProcess = +DISABLED_EMP\n", 1, 0},
    {"MaxQueueEntries = 0\n", 1, 0},
    {"NumDoorAnimations = 1\nDoorOpeningTime = 100\nDoorWaitOpenTime = 100\nDoorCloseTime = 100\n", 0, 0},
    {"NumDoorAnimations = 4\nDoorOpeningTime = 66\nDoorWaitOpenTime = 66\nDoorCloseTime = 66\n"
     "QuantityModifier = U0 4\n", 1, 1},
    {"QuantityModifier = U1 3\nQuantityModifier = U0 2\n", 1, 1},
    {"MaxQueueEntries = 3\n", 0, 1},
};
static const int kVariantCount = sizeof(kVariants) / sizeof(kVariants[0]);

static std::string factory_ini() {
    std::string s;
    for (int i = 0; i < kVariantCount; ++i) {
        s += "Object F" + std::to_string(i) + "\n  KindOf = STRUCTURE\n  CommandSet = P354Set\n"
             + kLive +
             "  Behavior = ProductionUpdate ModuleTag_Prod\n" + kVariants[i].body + "  End\n";
        if (kVariants[i].has_exit) s += "  Behavior = ScriptedExit ModuleTag_Exit\n  End\n";
        if (kVariants[i].has_parking) s += "  Behavior = ScriptedParking ModuleTag_Park\n  End\n";
        s += "  Behavior = StatusBitsUpgrade ModuleTag_O1\n    TriggeredBy = UpgO1\n"
             "    StatusToSet = HIJACKED\n  End\n"
             "  Behavior = StatusBitsUpgrade ModuleTag_O2\n    TriggeredBy = UpgO2 UpgP2\n"
             "    RequiresAllTriggers = Yes\n    StatusToSet = CAN_STEALTH\n  End\n"
             "  Behavior = StatusBitsUpgrade ModuleTag_P1\n    TriggeredBy = UpgP1\n"
             "    StatusToSet = BOOBY_TRAPPED\n  End\nEnd\n";
    }
    return s;
}

// Command set P354Set: the order of U9's button and UE's matters (last equivalent wins).
static const char* const kSetUnits[] = {"U0", "U1", "U2", "U3", "U4", "U5", "U6", "U7", "U8", "UE", "U9"};
static const int kAltUnits[] = {1, 3};

static std::string commands_ini() {
    std::string s;
    for (const char* u : kSetUnits) {
        s += std::string("CommandButton P354_") + u + "\n  Command = " +
             (std::string(u) == "U8" ? "DOZER_CONSTRUCT" : "UNIT_BUILD") + "\n  Object = " + u + "\nEnd\n";
    }
    for (const char* u : kUpgradeNames) {
        s += std::string("CommandButton P354_") + u + "\n  Command = " +
             (u[3] == 'P' ? "PLAYER_UPGRADE" : "OBJECT_UPGRADE") + "\n  Upgrade = " + u + "\nEnd\n";
    }
    s += "CommandSet P354Set\n";
    int n = 1;
    for (const char* u : kSetUnits) s += "  " + std::to_string(n++) + " = P354_" + u + "\n";
    for (const char* u : kUpgradeNames) s += "  " + std::to_string(n++) + " = P354_" + u + "\n";
    s += "End\nCommandSet P354Alt\n  1 = P354_U1\n  2 = P354_U3\n  3 = P354_UpgP2\nEnd\n";
    return s;
}

static const float kPenalty[] = {0.0f, 0.4f, 1.0f, 3.0f};
static const float kMinSpeed[] = {0.0f, 0.2f, 0.5f};
static const float kMaxSpeed[] = {0.0f, 0.8f, 0.95f};
static const float kMultiple[] = {0.0f, 0.85f, 1.3f};
static const float kHandicap[] = {1.0f, 0.5f, 1.5f};
static const float kChange[] = {0.0f, -0.25f, 0.1f};
static const int kEnergy[][2] = {{0, 0}, {5, 0}, {10, 10}, {5, 10}, {1, 100}, {20, 10}};

static uint32_t g_state = 0x35435435u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}

static const ModelConditionFlagType kFlags[] = {
    MODELCONDITION_DOOR_1_OPENING, MODELCONDITION_DOOR_1_CLOSING, MODELCONDITION_DOOR_1_WAITING_OPEN,
    MODELCONDITION_DOOR_1_WAITING_TO_CLOSE,
    MODELCONDITION_DOOR_2_OPENING, MODELCONDITION_DOOR_2_CLOSING, MODELCONDITION_DOOR_2_WAITING_OPEN,
    MODELCONDITION_DOOR_2_WAITING_TO_CLOSE,
    MODELCONDITION_DOOR_3_OPENING, MODELCONDITION_DOOR_3_CLOSING, MODELCONDITION_DOOR_3_WAITING_OPEN,
    MODELCONDITION_DOOR_3_WAITING_TO_CLOSE,
    MODELCONDITION_DOOR_4_OPENING, MODELCONDITION_DOOR_4_CLOSING, MODELCONDITION_DOOR_4_WAITING_OPEN,
    MODELCONDITION_DOOR_4_WAITING_TO_CLOSE,
    MODELCONDITION_ACTIVELY_CONSTRUCTING, MODELCONDITION_CONSTRUCTION_COMPLETE};

static const ThingTemplate* tmpl(const char* name) {
    const ThingTemplate* t = TheThingFactory->findTemplate(name);
    if (!t) {
        std::fprintf(stderr, "missing template %s\n", name);
        std::exit(1);
    }
    return t;
}
static const UpgradeTemplate* upg(int i) {
    const UpgradeTemplate* u = TheUpgradeCenter->findUpgrade(kUpgradeNames[i]);
    if (!u) {
        std::fprintf(stderr, "missing upgrade %s\n", kUpgradeNames[i]);
        std::exit(1);
    }
    return u;
}

static ProductionUpdate* production_of(Object* o) {
    for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m)
        if (ProductionUpdate* p = dynamic_cast<ProductionUpdate*>(*m)) return p;
    std::fprintf(stderr, "no production module\n");
    std::exit(1);
}

struct Run {
    oracle_wp51::World& w;
    Player* player;
    Object* factory = nullptr;
    ProductionUpdate* pu = nullptr;
    std::vector<Object*> extras;
    int variant = 0;
    UnsignedInt frame = 1;

    std::string row(int ret) {
        std::string s = std::to_string(ret) + " " + (g_log.empty() ? "-" : g_log) + " ";
        g_log.clear();
        s += std::to_string(player->getMoney()->countMoney()) + " ";
        for (int i = 0; i < kUpgrades; ++i) {
            s += player->hasUpgradeComplete(upg(i)) ? '1' : '0';
            s += player->hasUpgradeInProduction(upg(i)) ? '1' : '0';
        }
        s += " ";
        for (int i = 0; i < kUpgrades; ++i) {
            s += factory->hasUpgrade(upg(i)) ? '1' : '0';
            s += factory->affectedByUpgrade(upg(i)) ? '1' : '0';
            s += factory->canProduceUpgrade(upg(i)) ? '1' : '0';
            s += pu->isUpgradeInQueue(upg(i)) ? '1' : '0';
        }
        s += factory->getStatusBits().test(OBJECT_STATUS_HIJACKED) ? '1' : '0';
        s += factory->getStatusBits().test(OBJECT_STATUS_CAN_STEALTH) ? '1' : '0';
        s += factory->getStatusBits().test(OBJECT_STATUS_BOOBY_TRAPPED) ? '1' : '0';
        s += " ";
        const ModelConditionFlags& f = factory->getDrawable()->getModelConditionFlags();
        for (ModelConditionFlagType t : kFlags) s += f.test(t) ? '1' : '0';
        s += " " + std::to_string(GetGameLogicRandomSeedCRC()) + " ";
        for (const char* u : kUnitNames)
            s += std::to_string((int)TheBuildAssistant->canMakeUnit(factory, tmpl(u)));
        s += std::to_string((int)pu->canQueueUpgrade(upg(0)));
        s += "," + std::to_string(pu->getProductionCount());
        for (const char* u : kUnitNames) s += "," + std::to_string(pu->countUnitTypeInQueue(tmpl(u)));
        ScoreKeeper* sk = player->getScoreKeeper();
        s += " " + std::to_string(sk->getTotalMoneySpent()) + "," + std::to_string(sk->getTotalUnitsBuilt()) +
             "," + std::to_string(sk->getTotalBuildingsBuilt());
        AcademyStats* a = player->getAcademyStats();
        const UnsignedInt fields[] = {M(a, ASupply), (UnsignedInt)M(a, ARadar), M(a, APeons), M(a, AIdle),
                                      M(a, ALastUnit), M(a, ABought), M(a, AGather), M(a, AHeroes),
                                      M(a, ALastIncome), M(a, AMaxIncome)};
        s += " ";
        for (UnsignedInt v : fields) s += std::to_string(v) + ",";
        ListAsciiStringObjectID* done = TheScriptEngine->*access_member(SDone());
        s += " " + std::to_string(done[player->getPlayerIndex()].size());
        for (const AsciiStringObjectIDPair& p : done[player->getPlayerIndex()])
            s += std::string(",") + p.first.str() + ":" + std::to_string(p.second == factory->getID());
        s += " " + std::to_string(oracle_wp51::World::snapshot_crc(pu));
        s += " " + oracle_wp51::World::snapshot_save_hex(pu);
        return s;
    }

    void update_frames(uint32_t n) {
        for (uint32_t i = 0; i < n; ++i) {
            w.set_frame(++frame);
            const int r = (int)pu->update();
            if (r != (int)UPDATE_SLEEP_NONE) std::exit(3);
        }
    }

    // Returns the value the row records.
    int apply(uint32_t code, uint32_t a, uint32_t b) {
        w.set_frame(frame);
        switch (code) {
            case 0: {  // queue a unit
                const ProductionID id = pu->requestUniqueUnitID();
                return pu->queueCreateUnit(tmpl(kUnitNames[a]), id);
            }
            case 1: return pu->queueUpgrade(upg(a));
            case 2: return pu->cancelUnitCreate(static_cast<ProductionID>(a));
            case 3: return pu->cancelUpgrade(upg(a));
            case 4: pu->cancelAllUnitsOfType(tmpl(kUnitNames[a])); return 0;
            case 5: pu->cancelAndRefundAllProduction(); return 0;
            case 6: {
                DamageInfo d;
                pu->onDie(&d);
                return 0;
            }
            case 7: update_frames(a); return 0;
            case 8: pu->setHoldDoorOpen(static_cast<ExitDoorType>((int)a - 2), b != 0); return 0;
            case 9: g_door = (int)a - 2; g_budget = (int)b - 1; return 0;
            case 10: g_hold = a == 0 ? -9 : (int)a - 3; g_hold_flag = (int)b; return 0;
            case 11: M(player->getMoney(), PMoney) = a; return 0;
            case 12: {
                Energy* e = player->getEnergy();
                M(e, PEProd) = kEnergy[a][0];
                M(e, PECons) = kEnergy[a][1];
                M(e, PESab) = b == 0 ? 0 : frame + b;
                return 0;
            }
            case 13: factory->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_SOLD), a != 0); return 0;
            case 14: player->setCanBuildUnits(a != 0); player->setCanBuildBase(b != 0); return 0;
            case 15: factory->setScriptStatus(a ? OBJECT_STATUS_SCRIPT_UNPOWERED : OBJECT_STATUS_SCRIPT_DISABLED, b != 0); return 0;
            case 16: {  // another object of the player
                static const char* const kExtra[] = {"P_Pre", "U4", "UL", "U0", "U2", "U8", "F0"};
                Object* o = w.spawn_object(a == 6 ? ("F" + std::to_string(b)).c_str() : kExtra[a], g_team);
                if (a != 6 && b != 0) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
                extras.push_back(o);
                g_log.clear();
                return 0;
            }
            case 17: {  // destroy one of them
                if (extras.empty()) return 0;
                const size_t i = a % extras.size();
                TheGameLogic->destroyObject(extras[i]);
                w.flush_destroy_list();
                extras.erase(extras.begin() + i);
                g_log.clear();
                return 0;
            }
            case 18: {  // a unit queued in another factory counts against the limit
                for (Object* o : extras)
                    for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m)
                        if (ProductionUpdate* other = dynamic_cast<ProductionUpdate*>(*m)) {
                            const int r = other->queueCreateUnit(tmpl("U4"), other->requestUniqueUnitID());
                            g_log.clear();
                            return r;
                        }
                return 0;
            }
            case 19: {  // load the save into a fresh module and save again
                const std::string before = oracle_wp51::World::snapshot_save_hex(pu);
                std::vector<uint8_t> bytes;
                for (size_t i = 0; i < before.size(); i += 2)
                    bytes.push_back((uint8_t)std::stoul(before.substr(i, 2), nullptr, 16));
                Object* o = w.spawn_object(("F" + std::to_string(variant)).c_str(), w.team_b);
                ProductionUpdate* fresh = production_of(o);
                // `UpdateModule::xfer` forgets the heap index on load; keep it so the
                // object can be destroyed again.
                const Int index = fresh->friend_getIndexInLogic();
                const UnsignedInt next = fresh->friend_getNextCallFrame();
                oracle_wp51::World::snapshot_load_bytes(fresh, bytes);
                fresh->friend_setIndexInLogic(index);
                fresh->friend_setNextCallFrame(next);
                const std::string after = oracle_wp51::World::snapshot_save_hex(fresh);
                TheGameLogic->destroyObject(o);
                w.flush_destroy_list();
                g_log.clear();
                // The header holds the live schedule, which differs after the restore above.
                return before.substr(40) == after.substr(40);
            }
            case 20: {
                ScienceVec& v = M(player, PSci);
                v.clear();
                if (a) v.push_back(TheScienceStore->getScienceFromInternalName("SCIENCE_P354"));
                return 0;
            }
            case 21: g_park_reserve = (int)a; g_park_space = (int)b; return 0;
            case 22: player->addUpgrade(upg(a), UPGRADE_STATUS_COMPLETE); return 0;  // granted elsewhere
            case 23: factory->giveUpgrade(upg(a)); return 0;
            case 24: factory->setCommandSetStringOverride(a ? "P354Alt" : ""); return 0;
            case 25: TheGameLogic->setBuildableStatusOverride(tmpl(kUnitNames[a]), static_cast<BuildableStatus>(b)); return 0;
            case 26: {
                if (extras.empty()) return 0;
                extras[a % extras.size()]->setEffectivelyDead(b != 0);
                return 0;
            }
            default: std::exit(4);
        }
    }
};

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_pkt354_production <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    add_module(ScriptedExit::make, "ScriptedExit", 0);
    add_module(ScriptedParking::make, "ScriptedParking", 0);
    add_module(ScriptedBuilt::make, "ScriptedBuilt", MODULEINTERFACE_CREATE);
    w.load_ini(upgrades_ini());
    w.load_ini(templates_ini());
    w.load_ini(factory_ini());
    w.mem_ini.parse_blocks(commands_ini(), INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
        if (std::strcmp(tok, "CommandButton") == 0) INI::parseCommandButtonDefinition(ini);
        else if (std::strcmp(tok, "CommandSet") == 0) INI::parseCommandSetDefinition(ini);
    });
    TheThingFactory->postProcessLoad();  // resolves the prerequisite names
    w.init_with_neutral_player();
    g_team = w.team_a;
    Player* player = g_team->getControllingPlayer();
    // `Player::init` sets it only with a template; without it a deposit records its income
    // on player 0.
    player->getMoney()->setPlayerIndex(player->getPlayerIndex());

    FILE* out = std::fopen(argv[1], "w");
    if (!out) return 1;
    const int kTotal = 1000;
    std::fprintf(out, "{\"vector_count\":%d,\"vectors\":[", kTotal);
    for (int i = 0; i < kTotal; ++i) {
        Run run{w, player};
        run.variant = i < kVariantCount * 4 ? i % kVariantCount : (int)pick(kVariantCount);
        if (run.variant == 6 && pick(3) != 0) run.variant = (int)pick(6);
        else if (i >= kVariantCount * 4 && pick(4) == 0) run.variant = 8;
        run.frame = pick(8) == 0 ? 0x7ffffff0u + pick(32) : 1 + pick(2000);

        const uint32_t seed = 1 + pick(1000000);
        const uint32_t penalty = pick(4), min_speed = pick(3), max_speed = pick(3), multiple = pick(3);
        const uint32_t handicap = pick(4) == 0 ? 1 + pick(2) : 0;
        const uint32_t cost_change = pick(4) == 0 ? 1 + pick(2) : 0;
        const uint32_t time_change = pick(4) == 0 ? 1 + pick(2) : 0;
        const uint32_t kind_change = pick(4) == 0 ? 1 + pick(2) : 0;
        const uint32_t human = pick(2);
        const uint32_t money = pick(6) == 0 ? pick(600) : 3000 + pick(20000);
        const uint32_t energy = pick(6);

        InitRandom(seed);
        w.set_frame(run.frame);
        g_door = 0;
        g_budget = -1;
        g_hold = -9;
        g_hold_flag = 0;
        g_park_reserve = 0;
        g_park_space = 1;
        g_log.clear();
        TheWritableGlobalData->m_LowEnergyPenaltyModifier = kPenalty[penalty];
        TheWritableGlobalData->m_MinLowEnergyProductionSpeed = kMinSpeed[min_speed];
        TheWritableGlobalData->m_MaxLowEnergyProductionSpeed = kMaxSpeed[max_speed];
        TheWritableGlobalData->m_MultipleFactory = kMultiple[multiple];

        // Reset the player.
        for (int u = 0; u < kUpgrades; ++u) player->removeUpgrade(upg(u));
        M(player, PInProg).clear();
        player->getHandicap()->init();
        if (handicap) {
            Dict d;
            d.setReal(NAMEKEY("HANDICAP_BUILDCOST_GENERIC"), kHandicap[handicap]);
            d.setReal(NAMEKEY("HANDICAP_BUILDTIME_BUILDINGS"), kHandicap[handicap]);
            d.setReal(NAMEKEY("HANDICAP_BUILDTIME_GENERIC"), kHandicap[3 - handicap]);
            player->getHandicap()->readFromDict(&d);
        }
        M(player, PCost).clear();
        M(player, PTime).clear();
        if (cost_change) M(player, PCost)[NAMEKEY("U0")] = kChange[cost_change];
        if (time_change) M(player, PTime)[NAMEKEY("U0")] = kChange[time_change];
        KindOfMaskType vehicles = MAKE_KINDOF_MASK(KINDOF_VEHICLE);
        if (kind_change) player->addKindOfProductionCostChange(vehicles, kChange[kind_change]);
        M(player, PType) = human ? PLAYER_HUMAN : PLAYER_COMPUTER;
        player->setCanBuildUnits(TRUE);
        player->setCanBuildBase(TRUE);
        M(player, PSci).clear();
        for (const char* u : kUnitNames) {
            const ThingTemplate* t = tmpl(u);
            TheGameLogic->setBuildableStatusOverride(t, (BuildableStatus)0);
        }
        TheGameLogic->setBuildableStatusOverride(tmpl("U7"), BSTATUS_ONLY_BY_AI);
        TheGameLogic->setBuildableStatusOverride(tmpl("U8"), BSTATUS_IGNORE_PREREQUISITES);
        TheGameLogic->setBuildableStatusOverride(tmpl("U9"), BSTATUS_NO);
        player->getScoreKeeper()->reset(player->getPlayerIndex());
        AcademyStats* a = player->getAcademyStats();
        M(a, ASupply) = 0; M(a, ARadar) = FALSE; M(a, APeons) = 0; M(a, AIdle) = 0;
        M(a, ALastUnit) = 0; M(a, ABought) = 0; M(a, AGather) = 0; M(a, AHeroes) = 0;
        M(a, ALastIncome) = 0; M(a, AMaxIncome) = 0;
        (TheScriptEngine->*access_member(SDone()))[player->getPlayerIndex()].clear();
        M(player->getMoney(), PMoney) = money;
        M(player->getEnergy(), PEProd) = kEnergy[energy][0];
        M(player->getEnergy(), PECons) = kEnergy[energy][1];
        M(player->getEnergy(), PESab) = 0;

        run.factory = w.spawn_object(("F" + std::to_string(run.variant)).c_str(), g_team);
        run.pu = production_of(run.factory);
        g_factory = run.factory;
        g_log.clear();

        std::fprintf(out,
                     "%s\n{\"v\":%d,\"variant\":%d,\"frame\":%u,\"cfg\":[%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u],"
                     "\"disabled\":%u,\"spawn\":\"%s\",\"ops\":[",
                     i ? "," : "", i, run.variant, run.frame, seed, penalty, min_speed, max_speed, multiple,
                     handicap, cost_change, time_change, kind_change, human, money, energy,
                     (unsigned)run.pu->getDisabledTypesToProcess().toUnsignedInt(), run.row(0).c_str());

        std::vector<std::string> rows;
        std::vector<std::string> ops;
        auto emit = [&](uint32_t code, uint32_t x, uint32_t y) {
            const int ret = run.apply(code, x, y);
            ops.push_back("[" + std::to_string(code) + "," + std::to_string(x) + "," + std::to_string(y) + "]");
            rows.push_back(run.row(ret));
        };
        // Set the scene so that most queue requests can succeed.
        if (pick(2)) emit(16, 0, pick(6) == 0);
        if (pick(4) == 0) emit(16, 3, 0);
        if (kVariants[run.variant].has_parking && pick(3)) emit(21, 1, pick(5) != 0);
        if (pick(3) == 0) emit(20, 1, 0);
        if (pick(2)) emit(9, 2 + pick(4), 0);

        // Short scripts for states that random operations rarely reach.
        const uint32_t scenario = pick(10);
        const int v = run.variant;
        if (scenario < 3 && (v == 3 || v == 4 || v == 8 || v == 9)) {
            // A multiple build that stops part way, then a way of cancelling it.
            emit(9, 2 + pick(4), 2);
            const uint32_t unit = v == 4 ? 5 : v == 8 || pick(2) ? 0 : 1;
            emit(0, unit, 0);
            emit(7, 7 + pick(12), 0);
            const uint32_t how = pick(4);
            if (how == 0) emit(2, 1, 0);
            else if (how == 1) emit(4, unit, 0);
            else if (how == 2) emit(5, 0, 0);
            else emit(7, 20, 0);
        } else if (scenario == 3) {  // exactly enough money for a player upgrade
            const uint32_t u = pick(2);
            emit(11, u ? 50 : 300, 0);
            emit(1, u, 0);
        } else if (scenario == 4) {  // several facilities shorten the build
            for (uint32_t n = 2 + pick(3); n > 0; --n) emit(16, 0, 0);
            emit(0, 2, 0);
            emit(7, 6 + pick(8), 0);
        }

        const int count = 5 + (int)pick(10);
        for (int k = 0; k < count; ++k) {
            const uint32_t r = pick(100);
            // Aim some cancellations at what is queued.
            std::vector<uint32_t> ids, ups;
            for (const ProductionEntry* e = run.pu->firstProduction(); e; e = run.pu->nextProduction(e)) {
                if (e->getProductionType() == PRODUCTION_UNIT) ids.push_back((uint32_t)e->getProductionID());
                else
                    for (int u = 0; u < kUpgrades; ++u)
                        if (upg(u) == e->getProductionUpgrade()) ups.push_back(u);
            }
            static const uint32_t kQueueable[] = {0, 1, 0, 1, 3, 5, 2, 1, 2, 4, 6, 7, 8, 9, 10, 11};
            if (r < 24) emit(0, kQueueable[pick(16)], 0);
            else if (r < 33) emit(1, pick(kUpgrades), 0);
            else if (r < 58) emit(7, 1 + pick(pick(3) == 0 ? 30 : 7), 0);
            else if (r < 62) emit(2, !ids.empty() && pick(4) ? ids[pick(ids.size())] : 1 + pick(5), 0);
            else if (r < 64) emit(3, !ups.empty() && pick(4) ? ups[pick(ups.size())] : pick(kUpgrades), 0);
            else if (r < 66) emit(4, kQueueable[pick(10)], 0);
            else if (r < 68) emit(5, 0, 0);
            else if (r < 69) emit(6, 0, 0);
            else if (r < 73) emit(8, pick(8), pick(2));
            else if (r < 77) emit(9, pick(3) == 0 ? pick(2) : 2 + pick(4), pick(3) == 0 ? 1 + pick(3) : 0);
            else if (r < 79) emit(10, pick(3) == 0 ? 0 : 1 + pick(6), pick(2));
            else if (r < 81) {
                static const uint32_t kPrices[] = {0, 20, 25, 33, 40, 50, 60, 70, 90, 100, 150, 300, 400, 500, 1000};
                emit(11, pick(3) == 0 ? kPrices[pick(15)] : 5000 + pick(10000), 0);
            }
            else if (r < 83) emit(12, pick(6), pick(3) == 0 ? 1 + pick(8) : 0);
            else if (r < 84) emit(13, pick(3) != 0, 0);
            else if (r < 85) emit(14, pick(2), pick(2));
            else if (r < 86) emit(15, pick(2), pick(3) != 0);
            else if (r < 88) { const uint32_t kind = pick(2) ? 1 + pick(2) : pick(7); emit(16, kind, kind == 6 ? pick(2) : pick(5) == 0); }
            else if (r < 89) emit(26, pick(8), pick(3) != 0);
            else if (r < 90) emit(17, pick(8), 0);
            else if (r < 91) emit(18, 0, 0);
            else if (r < 93) emit(19, 0, 0);
            else if (r < 94) emit(20, pick(2), 0);
            else if (r < 96) emit(21, pick(2), pick(3) != 0);
            else if (r < 97) emit(22, pick(kUpgrades), 0);
            else if (r < 98) emit(23, 2 + pick(3), 0);
            else if (r < 99) emit(24, pick(2), 0);
            else emit(25, pick(kUnits), pick(4));
        }

        for (size_t k = 0; k < ops.size(); ++k) std::fprintf(out, "%s%s", k ? "," : "", ops[k].c_str());
        std::fprintf(out, "],\"rows\":[");
        for (size_t k = 0; k < rows.size(); ++k) std::fprintf(out, "%s\"%s\"", k ? "," : "", rows[k].c_str());
        std::fprintf(out, "]}");

        for (Object* o : run.extras) TheGameLogic->destroyObject(o);
        // Everything of the player goes, the produced units included.
        std::vector<Object*> all;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) all.push_back(o);
        for (Object* o : all)
            if (!o->testStatus(OBJECT_STATUS_DESTROYED)) TheGameLogic->destroyObject(o);
        w.flush_destroy_list();
        if (kind_change) player->removeKindOfProductionCostChange(vehicles, kChange[kind_change]);
        g_log.clear();
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return 0;
}
