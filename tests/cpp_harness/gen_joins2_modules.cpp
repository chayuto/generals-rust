// SPDX-License-Identifier: GPL-3.0-or-later
// Second set of golden vectors for ported modules working together, on the unmodified
// reference modules: MinefieldBehavior with AutoHealBehavior and ActiveBody under
// vehicles driven by PhysicsBehavior; ToppleUpdate and HelicopterSlowDeathBehavior with
// physics and die modules; BattlePlanUpdate with StealthDetectorUpdate and its special
// power module on several centers, with capture and destruction mid-plan;
// SpecialAbilityUpdate with StealthUpdate; HordeUpdate; ParkingPlaceBehavior with
// ProductionUpdate; and the scene of gen_joins_modules.cpp again. Every scene saves and
// loads module state mid-way (`snapshot_load_bytes` into the same instance, then
// `loadPostProcess`) and carries on; the bytes are in the vector for the Rust side.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_joins2_modules.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/JOINS2_Modules.json
//
// The fixtures, the AI stand-in and the observation of the first oracle are reused by
// including it.The AI stand-in also records the orders the ability updates give. JOINS_HEX=1 writes every row in full.
//
// Not covered: weapon damage (the mine's weapon has no radius and only its firing is
// recorded), FX and creation lists, helper modules, the heap rebuild of
// GameLogic::loadPostProcess (the harness keeps each update's heap index), and loading
// into a newly constructed object (lists the reference requires empty are cleared first).

#define main joins1_main
#include "gen_joins_modules.cpp"
#undef main

#include "Common/SpecialPower.h"
#include "GameLogic/Module/BattlePlanUpdate.h"
#include "GameLogic/Module/SpecialAbilityUpdate.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Weapon.h"

ACCESS(Q_Queue, ProductionUpdate, ProductionEntry*, m_productionQueue)
ACCESS(Q_Tail, ProductionUpdate, ProductionEntry*, m_productionQueueTail)
ACCESS(Q_SpawnIDs, SpawnBehavior, std::list<ObjectID>, m_spawnIDs)
ACCESS(Q_SpecialIDs, SpecialAbilityUpdate, std::list<ObjectID>, m_specialObjectIDList)
ACCESS(Q_Chose, AcademyStats, Bool, m_choseAStrategyForCenter)
ACCESS(Q_Academy, Player, AcademyStats, m_academyStats)
ACCESS(Q_Bomb, Player, Int, m_bombardBattlePlans)
ACCESS(Q_Hold, Player, Int, m_holdTheLineBattlePlans)
ACCESS(Q_Search, Player, Int, m_searchAndDestroyBattlePlans)
ACCESS(Q_Bonus, Player, BattlePlanBonuses*, m_battlePlanBonuses)
ACCESS(Q_Historic, WeaponTemplate, HistoricWeaponDamageList, m_historicDamage)

struct Q_LoadPost { using type = void (Snapshot::*)(); };
template struct MemberAccess<Q_LoadPost, &Snapshot::loadPostProcess>;
inline Q_LoadPost::type access_member(Q_LoadPost);

class J2AI : public JoinAI {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(J2AI, "OracleJoin2AI", 256, 256)
public:
    J2AI(Thing* t, const ModuleData* d) : JoinAI(t, d) {}
    void aiDoCommand(const AICommandParms* p) override {
        const char* name = nullptr;
        switch (p->m_cmd) {
        case AICMD_MOVE_TO_OBJECT: name = "moveobj"; break;
        case AICMD_ATTACK_OBJECT: name = "attack"; break;
        case AICMD_BUSY: name = "busy"; break;
        case AICMD_FACE_OBJECT: name = "faceobj"; break;
        case AICMD_FACE_POSITION: name = "facepos"; break;
        default: record(getObject(), p); return;
        }
        std::ostringstream s;
        s << static_cast<int>(getObject()->getID()) << ' ' << name << ' ' << static_cast<int>(p->m_cmdSource);
        if (p->m_cmd == AICMD_FACE_POSITION)
            s << ' ' << bits(p->m_pos.x) << ' ' << bits(p->m_pos.y) << ' ' << bits(p->m_pos.z);
        else if (p->m_cmd != AICMD_BUSY)
            s << ' ' << (p->m_obj ? static_cast<int>(p->m_obj->getID()) : 0);
        g_orders += s.str() + ";";
    }
};
J2AI::~J2AI() {}
static Module* make_ai2(Thing* t, const ModuleData* d) { return newInstance(J2AI)(t, d); }

// --- fixtures -------------------------------------------------------------------------
static const char* const kWeapons2 =
    "Weapon J2_Det\n  PrimaryDamage = 1.0\n  PrimaryDamageRadius = 0.0\n  AttackRange = 100.0\n"
    "  DelayBetweenShots = 1000\n  ClipSize = 0\n  HistoricBonusCount = 1000000\n  HistoricBonusTime = 33\n"
    "  HistoricBonusRadius = 50.0\nEnd\n";
static const char* const kPowers2 =
    "SpecialPower J2_PlanPower\n  Enum = SPECIAL_CHANGE_BATTLE_PLANS\n  ReloadTime = 0\nEnd\n"
    "SpecialPower J2_Hack\n  Enum = SPECIAL_BLACKLOTUS_DISABLE_VEHICLE_HACK\n  ReloadTime = 300\nEnd\n"
    "SpecialPower J2_Steal\n  Enum = SPECIAL_BLACKLOTUS_STEAL_CASH_HACK\n  ReloadTime = 500\nEnd\n";
static const char* const kLoco2 =
    "Locomotor J2_Air\n  Surfaces = AIR\n  Speed = 60\n  SpeedDamaged = 60\n  TurnRate = 180\n"
    "  Acceleration = 60\n  Braking = 60\n  Lift = 120\nEnd\n";

static std::string obj(const std::string& name, const char* kinds, int radius, int health, const std::string& body) {
    std::ostringstream s;
    s << "Object " << name << "\n  KindOf = PRELOAD " << kinds << "\n  Geometry = CYLINDER\n"
      << "  GeometryMajorRadius = " << radius << "\n  GeometryMinorRadius = " << radius
      << "\n  GeometryHeight = 6\n  GeometryIsSmall = No\n  VisionRange = 60\n  ShroudClearingRange = 80\n"
      << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = " << health << "\n    InitialHealth = " << health
      << "\n  End\n" << body << "End\n";
    return s.str();
}

static std::string die_behavior(const char* tag = "ModuleTag_Die", const char* extra = "") {
    return std::string("  Behavior = DestroyDie ") + tag + "\n" + extra + "  End\n";
}

static std::string mine(const char* fields, const char* heal_delay, int heal) {
    std::string s = std::string("  Behavior = MinefieldBehavior ModuleTag_Mine\n    DetonationWeapon = J2_Det\n") +
                    fields + "  End\n";
    if (heal_delay)
        s += "  Behavior = AutoHealBehavior ModuleTag_Heal\n    StartsActive = Yes\n    HealingAmount = " +
             std::to_string(heal) + "\n    HealingDelay = " + heal_delay + "\n  End\n";
    return s;
}

static std::string plan(int anim, int idle, int paralyze, const char* change, const char* health, int detects,
                        const char* valid) {
    std::ostringstream s;
    s << "  Behavior = StealthDetectorUpdate ModuleTag_Det\n    DetectionRate = 500\n  End\n"
         "  Behavior = SpecialAbility ModuleTag_Sp0\n    SpecialPowerTemplate = J2_PlanPower\n  End\n"
         "  Behavior = BattlePlanUpdate ModuleTag_Plan\n    SpecialPowerTemplate = J2_PlanPower\n"
      << "    BombardmentPlanAnimationTime = " << anim << "\n    HoldTheLinePlanAnimationTime = " << anim + 100
      << "\n    SearchAndDestroyPlanAnimationTime = " << anim * 2 << "\n    TransitionIdleTime = " << idle
      << "\n    BattlePlanChangeParalyzeTime = " << paralyze << "\n    ValidMemberKindOf = " << valid
      << "\n    HoldTheLinePlanArmorDamageScalar = 0.9\n    SearchAndDestroyPlanSightRangeScalar = 1.2"
      << "\n    StrategyCenterSearchAndDestroySightRangeScalar = 2.0"
      << "\n    StrategyCenterSearchAndDestroyDetectsStealth = " << (detects ? "Yes" : "No")
      << "\n    StrategyCenterHoldTheLineMaxHealthScalar = " << health
      << "\n    StrategyCenterHoldTheLineMaxHealthChangeType = " << change << "\n  End\n";
    return s.str();
}

static std::string ability(int k, const char* power, const char* extra) {
    const std::string n = std::to_string(k);
    return "  Behavior = SpecialAbility ModuleTag_Sp" + n + "\n    SpecialPowerTemplate = " + power + "\n  End\n" +
           "  Behavior = SpecialAbilityUpdate ModuleTag_Sau" + n + "\n    SpecialPowerTemplate = " + power + "\n" +
           extra + "  End\n";
}

static std::string horde(int rate, const char* kind, int count, int radius, int rub, const char* extra) {
    std::ostringstream s;
    s << "  Behavior = HordeUpdate ModuleTag_Horde\n    UpdateRate = " << rate << "\n    KindOf = " << kind
      << "\n    Count = " << count << "\n    Radius = " << radius << "\n    RubOffRadius = " << rub << "\n"
      << extra << "  End\n";
    return s.str();
}

static std::string topple(const char* extra) {
    return std::string("  Behavior = ToppleUpdate ModuleTag_Topple\n") + extra + "  End\n";
}

static std::string heli(const char* extra) {
    return std::string("  Behavior = HelicopterSlowDeathBehavior ModuleTag_Slow\n    DeathTypes = ALL -CRUSHED\n") +
           extra + "  End\n";
}

static std::string parking(int rows, int cols, bool runways, int heal) {
    std::ostringstream s;
    s << "  CommandSet = J2_Set\n"
         "  Behavior = ProductionUpdate ModuleTag_Prod\n    NumDoorAnimations = 4\n    DoorOpeningTime = 100\n"
         "    DoorWaitOpenTime = 200\n    DoorCloseTime = 100\n  End\n"
         "  Behavior = ParkingPlaceBehavior ModuleTag_Park\n    NumRows = " << rows << "\n    NumCols = " << cols
      << "\n    HasRunways = " << (runways ? "Yes" : "No") << "\n    ParkInHangars = " << (runways ? "Yes" : "No")
      << "\n    HealAmountPerSecond = " << heal << "\n    ApproachHeight = 30\n  End\n";
    return s.str();
}

static std::string fixture2_ini() {
    const std::string die = die_behavior();
    const std::string drive = ai("J2AI") + physics() + die;
    std::string s;
    // minefields and drivers
    s += obj("J2_Mine0", "MINE IMMOBILE", 8, 100,
             die + mine("    NumVirtualMines = 4\n    Regenerates = Yes\n", "300", 10));
    s += obj("J2_Mine1", "MINE IMMOBILE", 6, 100, die + mine("    NumVirtualMines = 3\n", "1000", 25));
    s += obj("J2_Mine2", "MINE IMMOBILE", 10, 60,
             die + mine("    NumVirtualMines = 1\n    DetonatedBy = ALLIES ENEMIES NEUTRAL\n", nullptr, 0));
    s += obj("J2_Mine3", "MINE IMMOBILE", 9, 120,
             die + mine("    NumVirtualMines = 5\n    Regenerates = Yes\n    WorkersDetonate = Yes\n"
                        "    StopsRegenAfterCreatorDies = Yes\n    DegenPercentPerSecondAfterCreatorDies = 20%\n",
                        "100", 5));
    s += obj("J2_Car", "VEHICLE SCORE", 5, 200, "  BuildCost = 100\n" + drive);
    s += obj("J2_Dozer", "VEHICLE DOZER", 4, 80, drive);
    // toppling and falling helicopters
    s += obj("J2_Tree0", "SHRUBBERY", 4, 50, "  CrushableLevel = 1\n" + die + topple(""));
    s += obj("J2_Tree1", "SHRUBBERY", 3, 5000,
             "  CrushableLevel = 1\n" + die +
                 topple("    KillWhenFinishedToppling = No\n    InitialVelocityPercent = 40%\n"
                        "    InitialAccelPercent = 5%\n    BounceVelocityPercent = 50%\n") +
                 physics());
    s += obj("J2_Tree2", "STRUCTURE", 5, 80,
             "  CrushableLevel = 0\n" + die +
                 topple("    KillWhenStartToppling = Yes\n    ToppleLeftOrRightOnly = Yes\n"
                        "    ReorientToppledRubble = Yes\n    KillStumpWhenToppled = Yes\n"));
    s += obj("J2_Crusher", "VEHICLE", 6, 400, "  CrusherLevel = 3\n" + drive);
    s += obj("J2_Rubble", "SHRUBBERY", 4, 30, die);
    const std::string crushed = die_behavior("ModuleTag_Die", "    DeathTypes = NONE +CRUSHED\n");
    s += obj("J2_Heli0", "VEHICLE AIRCRAFT SCORE", 6, 100,
             ai("J2AI") + physics() + crushed +
                 heli("    SpiralOrbitTurnRate = 140\n    SpiralOrbitForwardSpeed = 60\n"
                      "    SpiralOrbitForwardSpeedDamping = 0.9\n    MinSelfSpin = 90\n    MaxSelfSpin = 270\n"
                      "    SelfSpinUpdateDelay = 100\n    SelfSpinUpdateAmount = 10\n    FallHowFast = 40%\n"
                      "    MaxBraking = 20\n    FinalRubbleObject = J2_Rubble\n    DelayFromGroundToFinalDeath = 300\n"
                      "    DestructionDelay = 1000000\n"));
    s += obj("J2_Heli1", "VEHICLE AIRCRAFT", 5, 60,
             "  Behavior = J2AI ModuleTag_AI\n  End\n  Locomotor = SET_NORMAL J2_Air\n" + physics() + crushed +
                 heli("    SpiralOrbitTurnRate = 60\n    SpiralOrbitForwardSpeed = 20\n    MinSelfSpin = 10\n"
                      "    MaxSelfSpin = 40\n    SelfSpinUpdateDelay = 33\n    SelfSpinUpdateAmount = 5\n"
                      "    FallHowFast = 100%\n    SinkDelay = 100\n    SinkRate = 3.0\n    DestructionDelay = 2000\n    FinalRubbleObject = J2_Rubble\n"
                      "    MinBladeFlyOffDelay = 100\n    MaxBladeFlyOffDelay = 500\n"));
    // battle plans
    s += obj("J2_Center0", "STRUCTURE FS_STRATEGY_CENTER", 10, 1000,
             die + plan(200, 0, 500, "PRESERVE_RATIO", "2.0", 1, "INFANTRY VEHICLE"));
    s += obj("J2_Center1", "STRUCTURE", 10, 800,
             die + plan(100, 100, 300, "ADD_CURRENT_HEALTH_TOO", "1.5", 0, "INFANTRY VEHICLE STRUCTURE"));
    s += obj("J2_Center2", "STRUCTURE", 10, 600, die + plan(0, 200, 1000, "FULLY_HEAL", "1.3", 1, "VEHICLE"));
    const std::string quiet = stealth(true, 200, "MOVING");
    s += obj("J2_Soldier", "INFANTRY SCORE", 3, 90, drive + quiet);
    // special abilities
    s += obj("J2_Lotus", "INFANTRY", 3, 120,
             drive + stealth(true, 200, "MOVING USING_ABILITY") +
                 ability(0, "J2_Hack",
                         "    StartAbilityRange = 60\n    UnpackTime = 100\n    PreparationTime = 400\n"
                         "    PackTime = 100\n    EffectDuration = 1500\n    LoseStealthOnTrigger = Yes\n"
                         "    PreTriggerUnstealthTime = 200\n") +
                 ability(1, "J2_Steal",
                         "    StartAbilityRange = 40\n    UnpackTime = 200\n    PreparationTime = 600\n"
                         "    PackTime = 200\n    EffectValue = 300\n    LoseStealthOnTrigger = No\n"));
    s += obj("J2_Lotus2", "INFANTRY", 3, 100,
             drive + stealth(false, 100, "ATTACKING USING_ABILITY") +
                 ability(0, "J2_Hack",
                         "    StartAbilityRange = 150\n    PreparationTime = 200\n    PersistentPrepTime = 100\n"
                         "    EffectDuration = 700\n    AbilityAbortRange = 160\n    FlipOwnerAfterPacking = Yes\n"));
    s += obj("J2_Bank", "STRUCTURE FS_SUPPLY_CENTER CASH_GENERATOR", 10, 500, die);
    // hordes
    s += obj("J2_Zealot", "INFANTRY", 3, 80, drive + horde(500, "INFANTRY", 3, 40, 30, "    AlliesOnly = Yes\n"));
    s += obj("J2_Fanatic", "INFANTRY", 3, 80,
             drive + horde(300, "INFANTRY", 4, 30, 20, "    AlliesOnly = No\n    AllowedNationalism = No\n"));
    s += obj("J2_Brute", "VEHICLE", 5, 150, drive + horde(200, "VEHICLE", 2, 35, 25, "    ExactMatch = Yes\n"));
    // parking
    s += obj("J2_Field0", "STRUCTURE FS_AIRFIELD", 30, 1000, die + parking(2, 2, true, 10));
    s += obj("J2_Field1", "STRUCTURE", 25, 800, die + parking(1, 2, false, 25));
    s += obj("J2_Jet0", "VEHICLE AIRCRAFT SCORE", 5, 100, "  BuildCost = 200\n  BuildTime = 0.1\n" + ai("J2AI") + die);
    s += obj("J2_Jet1", "VEHICLE AIRCRAFT PRODUCED_AT_HELIPAD", 4, 90,
             "  BuildCost = 100\n  BuildTime = 0.2\n" + ai("J2AI") + die);
    return s;
}

static const char* const kJets[] = {"J2_Jet0", "J2_Jet1"};
static std::string commands2_ini() {
    std::string s;
    for (const char* u : kJets)
        s += std::string("CommandButton J2_B_") + u + "\n  Command = UNIT_BUILD\n  Object = " + u + "\nEnd\n";
    return s + "CommandSet J2_Set\n  1 = J2_B_J2_Jet0\n  2 = J2_B_J2_Jet1\nEnd\n";
}

static const char* const kTemplates[] = {
    "J2_Mine0", "J2_Mine1", "J2_Mine2", "J2_Mine3", "J2_Car", "J2_Dozer",                 // 0
    "J2_Tree0", "J2_Tree1", "J2_Tree2", "J2_Crusher", "J2_Heli0", "J2_Heli1",             // 6
    "J2_Center0", "J2_Center1", "J2_Center2", "J2_Soldier",                               // 12
    "J2_Lotus", "J2_Lotus2", "J2_Bank", "J_Detector",                                     // 16
    "J2_Zealot", "J2_Fanatic", "J2_Brute",                                                // 20
    "J2_Field0", "J2_Field1", "J2_Jet0", "J2_Jet1",                                       // 23
    "J_Fac0", "J_Fac1", "J_Fac2", "J_Fac3", "J_Hive0", "J_Hive1", "J_Plant", "J_Tech",    // 27
    "J_Truck", "J_Tank",                                                                  // 35
};
enum { kFamilies = 7 };
static const char* const kFamilyName[kFamilies] = {"mines", "topple", "plan", "ability", "horde", "parking", "first"};

// --- observation ----------------------------------------------------------------------
static std::string player_plans(int i) {
    Player* p = ThePlayerList->getNthPlayer(i);
    std::ostringstream s;
    s << p->*access_member(Q_Bomb()) << ',' << p->*access_member(Q_Hold()) << ',' << p->*access_member(Q_Search())
      << ',' << static_cast<int>((p->*access_member(Q_Academy())).*access_member(Q_Chose()));
    if (const BattlePlanBonuses* b = p->*access_member(Q_Bonus()))
        s << ',' << bits(b->m_armorScalar) << ',' << bits(b->m_sightRangeScalar) << ',' << b->m_bombardment << ','
          << b->m_holdTheLine << ',' << b->m_searchAndDestroy;
    return s.str() + ";";
}

// The weapon's historic damage list records each shot. Every shot first trims it by the
// current frame, so it is emptied after every frame.
static std::string g_shots;
static void take_shots() {
    WeaponTemplate* det = const_cast<WeaponTemplate*>(TheWeaponStore->findWeaponTemplate("J2_Det"));
    HistoricWeaponDamageList& shots = det->*access_member(Q_Historic());
    for (const auto& h : shots)
        g_shots += std::to_string(bits(h.location.x)) + ',' + std::to_string(bits(h.location.y)) + ',' +
                   std::to_string(bits(h.location.z)) + ';';
    shots.clear();
}

static std::string extra_row() {
    std::ostringstream s;
    for (Object* o : world_objects()) {
        BodyModuleInterface* body = o->getBodyModule();
        s << static_cast<int>(o->getID()) << ':' << bits(body->getMaxHealth()) << ',' << bits(body->getDamageScalar())
          << ',' << bits(o->getVisionRange()) << ',' << bits(o->getShroudClearingRange()) << ','
          << o->getWeaponBonusCondition() << ',' << o->getDisabledUntil(DISABLED_PARALYZED) << ','
          << o->getDisabledUntil(DISABLED_HACKED) << ','
          << (o->getAIUpdateInterface() ? static_cast<int>(o->getAIUpdateInterface()->getIgnoredObstacleID()) : 0)
          << ';';
    }
    s << '|';
    for (int i = 0; i < 3; ++i) s << player_plans(i);
    take_shots();
    s << '|' << g_shots;
    g_shots.clear();
    return s.str();
}

// --- save and load --------------------------------------------------------------------
static std::vector<uint8_t> unhex(const std::string& h) {
    std::vector<uint8_t> out;
    for (size_t i = 0; i + 1 < h.size(); i += 2)
        out.push_back(static_cast<uint8_t>(std::strtoul(h.substr(i, 2).c_str(), nullptr, 16)));
    return out;
}

static std::map<std::string, int> g_loaded;

// Save every tagged module of `o`, load the bytes back and post-process; `loads` gets
// one `tag=hex` per module.
static void reload(Object* o, std::string& loads) {
    for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m) {
        const std::string tag = TheNameKeyGenerator->keyToName((*m)->getModuleTagNameKey()).str();
        if (tag.compare(0, 10, "ModuleTag_") != 0 || tag == "ModuleTag_AI") continue;
        if (tag.size() > 16 && tag.compare(tag.size() - 6, 6, "Helper") == 0) continue;
        const std::string hex = oracle_wp51::World::snapshot_save_hex(*m);
        if (ProductionUpdate* pu = dynamic_cast<ProductionUpdate*>(*m)) {
            ProductionEntry* e = pu->*access_member(Q_Queue());
            while (e) {
                ProductionEntry* next = const_cast<ProductionEntry*>(pu->nextProduction(e));
                deleteInstance(e);
                e = next;
            }
            pu->*access_member(Q_Queue()) = nullptr;
            pu->*access_member(Q_Tail()) = nullptr;
        }
        if (SpawnBehavior* sb = dynamic_cast<SpawnBehavior*>(*m)) (sb->*access_member(Q_SpawnIDs())).clear();
        if (SpecialAbilityUpdate* sa = dynamic_cast<SpecialAbilityUpdate*>(*m))
            (sa->*access_member(Q_SpecialIDs())).clear();
        UpdateModule* update = dynamic_cast<UpdateModule*>(*m);
        const Int index = update ? update->friend_getIndexInLogic() : 0;
        oracle_wp51::World::snapshot_load_bytes(*m, unhex(hex));
        if (update) update->friend_setIndexInLogic(index);
        (static_cast<Snapshot*>(*m)->*access_member(Q_LoadPost()))();
        // The bytes loaded, and what the module saves afterwards.
        loads += std::to_string(static_cast<int>(o->getID())) + ":" + tag.substr(10) + "=" + hex + ">" +
                 oracle_wp51::World::snapshot_save_hex(*m) + " ";
        ++g_loaded[tag.substr(10)];
    }
}

// --- operations -----------------------------------------------------------------------
struct Scene2 {
    Scene s;
    int family = 0;
    std::string loads;  // filled by a reload

    std::vector<Object*> with_tag(const char* tag) const {
        std::vector<Object*> out;
        const NameKeyType key = TheNameKeyGenerator->nameToKey(tag);
        for (Object* o : s.live())
            for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m)
                if ((*m)->getModuleTagNameKey() == key) {
                    out.push_back(o);
                    break;
                }
        return out;
    }

    static Module* module_of(Object* o, const char* tag) {
        const NameKeyType key = TheNameKeyGenerator->nameToKey(tag);
        for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m)
            if ((*m)->getModuleTagNameKey() == key) return *m;
        return nullptr;
    }

    static Object* choose(const std::vector<Object*>& v, uint32_t a) { return v.empty() ? nullptr : v[a % v.size()]; }

    Object* spawn_at(uint32_t t, uint32_t team, float x, float y, float z, float angle) {
        Object* o = s.spawn(kTemplates[t], team, x, y, angle);
        if (z != 0.0f) {
            Coord3D c;
            c.x = x;
            c.y = y;
            c.z = z;
            o->setPosition(&c);
        }
        return o;
    }

    int apply(const Op& op) {
        s.w.set_frame(s.frame);
        loads.clear();
        switch (op.code) {
        case 0:
            for (uint32_t i = 0; i < op.a; ++i) {
                take_shots();
                run_frame(s.w, s.frame);
                take_shots();
            }
            return 0;
        case 9: {
            Object* o = choose(s.live(), op.a);
            if (!o) return -1;
            DamageInfo d;
            d.in.m_damageType = op.c ? DAMAGE_CRUSH : DAMAGE_UNRESISTABLE;
            d.in.m_deathType = op.c ? DEATH_CRUSHED : DEATH_NORMAL;
            d.in.m_amount = static_cast<float>(op.b);
            const int id = static_cast<int>(o->getID());
            o->attemptDamage(&d);
            return id;
        }
        case 16: {
            Object* o = choose(s.live(), op.a);
            if (!o) return -1;
            reload(o, loads);
            return static_cast<int>(o->getID());
        }
        case 17: {  // push one object toward another
            Object* o = choose(s.live_of("physics"), op.a);
            Object* t = choose(s.live(), op.b);
            if (!o || !t) return -1;
            const float k = static_cast<float>(op.c) / 64.0f;
            Coord3D f;
            f.x = (t->getPosition()->x - o->getPosition()->x) * k;
            f.y = (t->getPosition()->y - o->getPosition()->y) * k;
            f.z = 0.0f;
            o->getPhysics()->applyForce(&f);
            return static_cast<int>(o->getID());
        }
        case 18:
        case 19: {  // a special power, at an object or alone
            const std::string tag = "ModuleTag_Sp" + std::to_string(op.code == 18 ? op.c : 0);
            Object* o = choose(with_tag(tag.c_str()), op.a);
            if (!o) return -1;
            SpecialPowerModule* sp = static_cast<SpecialPowerModule*>(static_cast<BehaviorModule*>(module_of(o, tag.c_str())));
            if (op.code == 19) {
                sp->doSpecialPower(op.b);
            } else {
                Object* t = choose(s.live(), op.b);
                if (!t || t == o) return -1;
                sp->doSpecialPowerAtObject(t, 0);
            }
            return static_cast<int>(o->getID());
        }
        case 20: {  // a new object beside an old one
            Object* near = choose(s.live(), op.c);
            if (!near) return -1;
            const float x = near->getPosition()->x + static_cast<float>(op.c % 17) - 8.0f;
            const float y = near->getPosition()->y + static_cast<float>(op.c % 11) - 5.0f;
            return static_cast<int>(spawn_at(op.a, op.b, x, y, 0.0f, 0.5f)->getID());
        }
        case 21: {  // onto another object
            Object* o = choose(s.live_of("physics"), op.a);
            Object* t = choose(s.live(), op.b);
            if (!o || !t) return -1;
            Coord3D c = *t->getPosition();
            c.x += static_cast<float>(op.c % 7) - 3.0f;
            c.y += static_cast<float>(op.c / 7 % 7) - 3.0f;
            o->setPosition(&c);
            return static_cast<int>(o->getID());
        }
        case 22: {
            Object* f = choose(with_tag("ModuleTag_Park"), op.a);
            if (!f) return -1;
            ProductionUpdateInterface* pu = f->getProductionUpdateInterface();
            const ProductionID id = pu->requestUniqueUnitID();
            return pu->queueCreateUnit(TheThingFactory->findTemplate(kJets[op.b]), id);
        }
        default: return s.apply(op);
        }
    }

    Op random_op() {
        Op op = {0, 0, 0, 0};
        if (pick(100) < 8) return {16, pick(24), 0, 0};
        if (family == 6) return s.random_op();
        const uint32_t k = pick(100);
        const auto frames = [] { return Op{0, 1 + (pick(4) ? pick(10) : pick(50)), 0, 0}; };
        const auto common = [&]() -> Op {
            const uint32_t c = pick(100);
            if (c < 16) return {6, pick(24), 280 + pick(60), 280 + pick(60)};
            if (c < 24) return {7, pick(16), pick(17), pick(17)};
            if (c < 34) return {8, pick(24), 0, 0};
            if (c < 62) return {9, pick(24), pick(3) ? 5 + pick(60) : 2000, pick(5) == 0 ? 1u : 0u};
            if (c < 72) return {10, pick(24), pick(3), pick(3) ? 1u : 0u};
            if (c < 86) return {11, pick(24), pick(3), 0};
            return {14, pick(24), pick(2), pick(2)};
        };
        if (k < 34) return frames();
        if (k < 58) return common();
        switch (family) {
        case 0:
            if (k < 74) return {21, pick(8), pick(12), pick(49)};
            if (k < 88) return {17, pick(8), pick(12), 1 + pick(16)};
            if (k < 94) return {20, pick(6), pick(3), pick(1000)};
            return frames();
        case 1:
            if (k < 74) return {17, pick(8), pick(12), 8 + pick(40)};
            if (k < 80) return {9, 7 + pick(3), 2000, 0};  // mostly a helicopter
            if (k < 90) return {21, pick(8), pick(12), pick(49)};
            if (k < 94) return {20, 6 + pick(6), pick(3), pick(1000)};
            return frames();
        case 2:
            if (k < 78) return {19, pick(4), 1u << (17 + pick(3)), 0};
            if (k < 86) return {20, pick(4) ? 15 : (pick(2) ? 4 : 12 + pick(3)), pick(3), pick(1000)};
            return frames();
        case 3:
            if (k < 76) return {18, pick(3), pick(12), pick(2)};
            if (k < 80) return {17, pick(8), pick(12), 1 + pick(8)};
            if (k < 86) return {20, pick(2) ? 4 : 16 + pick(4), pick(3), pick(1000)};
            return frames();
        case 4:
            if (k < 70) return {21, pick(12), pick(12), pick(49)};
            if (k < 80) return {17, pick(8), pick(12), 1 + pick(16)};
            if (k < 90) return {20, 20 + pick(3), pick(3), pick(1000)};
            return frames();
        default:
            if (k < 82) return {22, pick(2), pick(2), 0};
            if (k < 84) return {2, pick(2), pick(5), 0};
            if (k < 87) return {20, 23 + pick(4), pick(3), pick(1000)};
            return frames();
        }
    }

    // template, team, x, y, z, angle index
    std::vector<uint32_t> layout() {
        std::vector<uint32_t> out;
        const auto put = [&](uint32_t t, uint32_t team, uint32_t x, uint32_t y, uint32_t z) {
            const uint32_t angle = pick(8);
            out.insert(out.end(), {t, team, x, y, z, angle});
        };
        const auto near = [&] { return 295 + pick(30); };
        switch (family) {
        case 0:
            for (int i = 0; i < 3; ++i) put(pick(4), pick(3), near(), near(), 0);
            for (int i = 0; i < 4; ++i) put(pick(4) ? 4 : 5, pick(3), near(), near(), 0);
            break;
        case 1:
            for (int i = 0; i < 4; ++i) put(6 + pick(3), pick(3), near(), near(), 0);
            for (int i = 0; i < 2; ++i) put(9, 1 + pick(2), near(), near(), 0);
            for (int i = 0; i < 2; ++i) put(10 + pick(2), 1 + pick(2), near(), near(), 20 + pick(60));
            break;
        case 2:
            for (int i = 0; i < 2; ++i) put(12 + pick(3), 1 + pick(5) / 4, near(), near(), 0);
            for (int i = 0; i < 3; ++i) put(15, 1 + pick(2), near() + 20, near(), 0);
            put(4, 1 + pick(2), near(), near() + 20, 0);
            break;
        case 3:
            put(16, 1, near(), near(), 0);
            put(17, 1 + pick(2), near(), near(), 0);
            for (int i = 0; i < 3; ++i) put(4, 2 - pick(5) / 4, near() + 30, near(), 0);
            put(18, 2, near(), near() + 40, 0);
            put(19, 2, near() + 60, near() + 60, 0);
            break;
        case 4:
            for (int i = 0; i < 8; ++i) put(pick(4) ? 20 : 21, 1 + pick(6) / 5, near(), near(), 0);
            for (int i = 0; i < 3; ++i) put(22, 1 + pick(2), near(), near(), 0);
            break;
        case 5:
            put(23, 1, near(), near(), 0);
            put(24, 1, near() + 80, near(), 0);
            for (int i = 0; i < 2; ++i) put(25 + pick(2), 1, near() + 40, near() + 40, 0);
            break;
        default: {
            const uint32_t x = 250 + pick(40), y = 250 + pick(40);
            put(27 + pick(4), 1, x, y, 0);
            put(19, 2, x + 20 + pick(120), y - 40 + pick(120), 0);
            put(31 + pick(2), 1, x - 40, y + 30, 0);
            put(33, 1, x + 5, y + 70, 0);
            put(34, 0, x - 70, y - 20, 0);
            put(35, 1, x + 40, y + 10, 0);
            break;
        }
        }
        return out;
    }
};

// The harness world has no terrain to block a line of sight.
class ClearTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    Bool isClearLineOfSight(const Coord3D&, const Coord3D&) const override { return TRUE; }
};

static const float kAngles2[] = {0.0f, 1.5707964f, 3.1415927f, -1.5707964f, 0.5f, -2.75f, 6.0f, 0.001f};

int main(int argc, char** argv) {
    if (argc < 2) die("usage: gen_joins2_modules <out.json>");
    g_hex = std::getenv("JOINS_HEX") != nullptr;
    g_state = 0x70150002u;
    oracle_wp51::World w;
    TheTerrainLogic = new ClearTerrain;
    TheInGameUI = new CashUI;
    TheGameText = new CashText;
    auto* factory = TheModuleFactory;
    (factory->*(&ExposedFactory::addModuleInternal))(make_ai, AIUpdateInterface::friend_newModuleData,
        MODULETYPE_BEHAVIOR, "JoinAI", AIUpdateInterface::getInterfaceMask());
    (factory->*(&ExposedFactory::addModuleInternal))(make_truck, SupplyTruckAIUpdate::friend_newModuleData,
        MODULETYPE_BEHAVIOR, "JoinTruckAI", SupplyTruckAIUpdate::getInterfaceMask());
    (factory->*(&ExposedFactory::addModuleInternal))(make_ai2, AIUpdateInterface::friend_newModuleData,
        MODULETYPE_BEHAVIOR, "J2AI", AIUpdateInterface::getInterfaceMask());
    TheWritableGlobalData->m_baseValuePerSupplyBox = 75;
    TheWritableGlobalData->m_historicDamageLimit = 1;
    w.load_ini(kUpgradeIni);
    w.load_ini(kLocoIni);
    w.load_ini(kLoco2);
    w.load_ini(kWeapons2);
    w.load_ini(kPowers2);
    w.load_ini(fixture_ini());
    w.load_ini(fixture2_ini());
    w.mem_ini.parse_blocks(commands_ini() + commands2_ini(), INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
        if (std::strcmp(tok, "CommandButton") == 0) INI::parseCommandButtonDefinition(ini);
        else if (std::strcmp(tok, "CommandSet") == 0) INI::parseCommandSetDefinition(ini);
    });
    TheThingFactory->postProcessLoad();
    w.init_with_neutral_player();
    ThePlayerList->setLocalPlayer(ThePlayerList->getNthPlayer(5));

    std::FILE* out = std::fopen(argv[1], "wb");
    if (!out) die("cannot open output");
    std::fprintf(out, "{\"upgrades\":\"%s\",\n\"locomotor\":\"%s\",\n\"locomotor2\":\"%s\",\n\"weapons\":\"%s\",\n"
                      "\"powers\":\"%s\",\n\"fixture\":\"%s\",\n\"templates\":\"",
                 escape(kUpgradeIni).c_str(), escape(kLocoIni).c_str(), escape(kLoco2).c_str(),
                 escape(kWeapons2).c_str(), escape(kPowers2).c_str(), escape(fixture_ini() + fixture2_ini()).c_str());
    for (const char* t : kTemplates) std::fprintf(out, "%s ", t);
    std::fprintf(out, "\",\n\"vectors\":[\n");

    const int kVectors = 1400;
    std::map<std::string, int> done, skipped;
    long long total_rows = 0;
    for (int v = 0; v < kVectors; ++v) {
        Scene2 scene{{w}};
        scene.family = v % kFamilies;
        g_full = g_hex || v < kFamilies;
        const uint32_t seed = 1 + pick(1000000);
        InitRandom(seed);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        reset_players();
        for (int i = 0; i < 3; ++i) {
            Player* p = ThePlayerList->getNthPlayer(i);
            (p->*access_member(Q_Academy())).*access_member(Q_Chose()) = FALSE;
        }
        w.set_frame(1);
        g_orders.clear();
        g_paths.clear();
        g_texts.clear();
        g_prone.clear();

        const std::vector<uint32_t> layout = scene.layout();
        for (size_t i = 0; i + 5 < layout.size(); i += 6)
            scene.spawn_at(layout[i], layout[i + 1], static_cast<float>(layout[i + 2]),
                           static_cast<float>(layout[i + 3]), static_cast<float>(layout[i + 4]),
                           kAngles2[layout[i + 5]]);
        const auto observe = [&](int ret) {
            // The cash hack's floating texts are presentation; the port reports them as a
            // client event, so only the money is compared.
            if (scene.family == 3) g_texts.clear();
            const std::string base = scene.s.row(ret);
            const std::string extra = extra_row();
            return base + '@' + (g_full ? extra : digest(extra));
        };
        std::vector<std::string> rows, loads;
        rows.push_back(observe(0));
        std::vector<Op> ops;
        const int n = 16 + pick(16);
        for (int i = 0; i < n; ++i) {
            Op op = scene.random_op();
            if (i == 0) op = {0, 1 + pick(30), 0, 0};
            const int ret = scene.apply(op);
            ops.push_back(op);
            if (op.code == 16 && ret != -1) loads.push_back(scene.loads);
            (ret == -1 ? skipped : done)[std::string(kFamilyName[scene.family]) + " " + std::to_string(op.code)]++;
            rows.push_back(observe(ret));
        }

        std::fprintf(out, "%s{\"v\":%d,\"seed\":%u,\"layout\":[", v ? ",\n" : "", v, seed);
        for (size_t i = 0; i < layout.size(); ++i) std::fprintf(out, "%s%u", i ? "," : "", layout[i]);
        std::fprintf(out, "],\"ops\":[");
        for (size_t i = 0; i < ops.size(); ++i)
            std::fprintf(out, "%s%u,%u,%u,%u", i ? "," : "", ops[i].code, ops[i].a, ops[i].b, ops[i].c);
        std::fprintf(out, "],\"loads\":[\"");
        for (size_t i = 0; i < loads.size(); ++i) std::fprintf(out, "%s%s", i ? "\",\"" : "", loads[i].c_str());
        std::fprintf(out, "\"],\"rows\":[\"");
        for (size_t i = 0; i < rows.size(); ++i) std::fprintf(out, "%s%s", i ? "\",\"" : "", rows[i].c_str());
        std::fprintf(out, "\"]}");
        total_rows += static_cast<long long>(rows.size());

        for (int pass = 0; pass < 6; ++pass) {
            for (Object* o : world_objects())
                if (!o->isDestroyed()) TheGameLogic->destroyObject(o);
            TheGameLogic->processDestroyList();
        }
        if (TheGameLogic->getFirstObject() || !(TheGameLogic->*access_member(GHeap())).empty())
            die("world not empty after a vector");
        ThePartitionManager->update();
        extra_row();
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    std::printf("rows %lld\n", total_rows);
    for (const auto& kv : done) std::printf("op %s: %d done, %d skipped\n", kv.first.c_str(), kv.second, skipped[kv.first]);
    for (const auto& kv : g_loaded) std::printf("loaded %s: %d\n", kv.first.c_str(), kv.second);
    return 0;
}
