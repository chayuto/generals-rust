// PKT-390 `Object` lifecycle gaps: weapon set updates, battle plan bonuses on a new
// object, mines on construction and on defection, capture and the pathfind map.
// Compiles against libz_gameengine.a via scripts/build_oracle.py. Argument: the
// output file, one vector per line.

#include "oracle_wp51_world.h"

#include "GameLogic/AIPathfind.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BattlePlanUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/DozerAIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Pathfinder/PathfindCell.h"
#include "GameLogic/WeaponSet.h"
#include "Common/ScoreKeeper.h"

#include <cstdio>
#include <map>

struct WeaponSetTag { using type = WeaponSet Object::*; };
template struct MemberAccess<WeaponSetTag, &Object::m_weaponSet>;
inline WeaponSet Object::*access_member(WeaponSetTag);

static std::FILE* g_out;
static std::map<ObjectID, std::string> g_names;

static void row(const std::string& s) { std::fprintf(g_out, "%s\n", s.c_str()); }
static std::string num(long long v) { return std::to_string(v); }
static std::string bits(Real v) {
    uint32_t u;
    std::memcpy(&u, &v, 4);
    return num(u);
}
static std::string who(ObjectID id) {
    if (id == INVALID_ID) return "-";
    auto it = g_names.find(id);
    return it == g_names.end() ? "?" + num(id) : it->second;
}

static Object* make(oracle_wp51::World& w, const char* name, const char* tmpl, Team* team, Real x, Real y) {
    Coord3D p = {x, y, 0.0f};
    Object* o = w.spawn_object(tmpl, team, p, 0.0f);
    if (!o) {
        std::fprintf(stderr, "oracle: no template %s\n", tmpl);
        std::abort();
    }
    g_names[o->getID()] = name;
    return o;
}

static const char* INI_TEXT =
    "Weapon WA\n  PrimaryDamage = 5.0\n  AttackRange = 100.0\n  ClipSize = 4\n  ClipReloadTime = 1000\n  DelayBetweenShots = 200\nEnd\n"
    "Weapon WB\n  PrimaryDamage = 7.0\n  AttackRange = 50.0\n  DamageType = FLAME\n  AntiAirborneVehicle = Yes\nEnd\n"
    "Weapon WC\n  PrimaryDamage = 9.0\n  AttackRange = 150.0\n  ClipSize = 2\n  ClipReloadTime = 500\n  DamageType = ARMOR_PIERCING\nEnd\n"
    "Locomotor LifeLegs\n  Surfaces = GROUND\n  Appearance = TWO_LEGS\n  Speed = 30\nEnd\n"
    "Object WSUnit\n  KindOf = VEHICLE\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
    "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY WA\n    Weapon = SECONDARY WB\n  End\n"
    "  WeaponSet\n    Conditions = VETERAN\n    ShareWeaponReloadTime = Yes\n    Weapon = PRIMARY WC\n    Weapon = TERTIARY WB\n  End\n"
    "  WeaponSet\n    Conditions = ELITE\n    WeaponLockSharedAcrossSets = Yes\n    Weapon = PRIMARY WA\n    Weapon = SECONDARY WC\n  End\n"
    "End\n"
    "Object PlanVeh\n  KindOf = VEHICLE\n  VisionRange = 100\n  ShroudClearingRange = 120\n  Geometry = CYLINDER\n  GeometryMajorRadius = 5\n  GeometryHeight = 5\n"
    "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\nEnd\n"
    "Object PlanInf\n  KindOf = INFANTRY\n  VisionRange = 100\n  ShroudClearingRange = 120\n"
    "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\nEnd\n"
    "Object Bld\n  KindOf = STRUCTURE\n  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 20\n"
    "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 500\n    InitialHealth = 500\n  End\nEnd\n"
    "Object Mine\n  KindOf = MINE IMMOBILE\n  Geometry = CYLINDER\n  GeometryMajorRadius = 3\n  GeometryHeight = 2\n"
    "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\nEnd\n"
    "Object Rock\n  KindOf = IMMOBILE\n  Geometry = CYLINDER\n  GeometryMajorRadius = 3\n  GeometryHeight = 2\n"
    "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\nEnd\n"
    "Object Walker\n  KindOf = INFANTRY\n  TransportSlotCount = 1\n  Geometry = CYLINDER\n  GeometryMajorRadius = 5\n  GeometryHeight = 10\n"
    "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
    "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n  Locomotor = SET_NORMAL LifeLegs\nEnd\n"
    "Object Truck\n  KindOf = VEHICLE\n  Geometry = CYLINDER\n  GeometryMajorRadius = 5\n  GeometryHeight = 10\n"
    "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
    "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 4\n    AllowInsideKindOf = INFANTRY\n  End\n"
    "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n  Locomotor = SET_NORMAL LifeLegs\nEnd\n"
    "Object Tech\n  KindOf = STRUCTURE\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 500\n    InitialHealth = 500\n  End\nEnd\n"
    "Object Dozer\n  KindOf = VEHICLE DOZER\n  Geometry = CYLINDER\n  GeometryMajorRadius = 5\n  GeometryHeight = 10\n"
    "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
    "  Behavior = DozerAIUpdate ModuleTag_AI\n    RepairHealthPercentPerSecond = 2%\n    BoredTime = 5000\n    BoredRange = 200\n  End\n"
    "  Locomotor = SET_NORMAL LifeLegs\nEnd\n";

// ---- weapon sets ----------------------------------------------------------------

static void weapons_row(const char* op, Object* o) {
    WeaponSet& set = o->*access_member(WeaponSetTag());
    row(std::string("weapons ") + op + " " + oracle_wp51::World::snapshot_save_hex(&set));
}

static void weapon_sets(oracle_wp51::World& w) {
    w.set_frame(7);
    Object* o = make(w, "ws", "WSUnit", w.team_a, 50.0f, 50.0f);
    weapons_row("spawn", o);
    w.set_frame(11);
    o->setWeaponLock(SECONDARY_WEAPON, LOCKED_PERMANENTLY);
    weapons_row("lock2", o);
    w.set_frame(20);
    o->setWeaponSetFlag(WEAPONSET_VETERAN);
    weapons_row("+vet", o);
    w.set_frame(25);
    o->setWeaponSetFlag(WEAPONSET_VETERAN);
    weapons_row("+vet-again", o);
    o->setWeaponLock(TERTIARY_WEAPON, LOCKED_TEMPORARILY);
    w.set_frame(31);
    o->clearWeaponSetFlag(WEAPONSET_VETERAN);
    weapons_row("-vet", o);
    o->setWeaponLock(SECONDARY_WEAPON, LOCKED_PERMANENTLY);
    w.set_frame(42);
    o->setWeaponSetFlag(WEAPONSET_ELITE);
    weapons_row("+elite", o);
    w.set_frame(53);
    o->setWeaponSetFlag(WEAPONSET_VETERAN);
    weapons_row("+vet-on-elite", o);
    w.set_frame(64);
    o->setWeaponSetFlag(WEAPONSET_HERO);
    weapons_row("+hero", o);
    w.set_frame(75);
    o->clearWeaponSetFlag(WEAPONSET_ELITE);
    weapons_row("-elite", o);
    w.set_frame(86);
    o->clearWeaponSetFlag(WEAPONSET_VETERAN);
    weapons_row("-vet-hero-left", o);
}

// ---- battle plans ---------------------------------------------------------------

static void plan_row(const char* op, Object* o) {
    BodyModuleInterface* body = o->getBodyModule();
    UnsignedInt cond = o->getWeaponBonusCondition();
    row(std::string("plan ") + op + " " + who(o->getID()) + " " + bits(body->getDamageScalar()) + " " +
        bits(o->getVisionRange()) + " " + bits(o->getShroudClearingRange()) + " " + num(cond));
}

static void battle_plans(oracle_wp51::World& w) {
    w.set_frame(100);
    Player* a = w.team_a->getControllingPlayer();
    BattlePlanBonusesData bonus;
    bonus.m_armorScalar = 0.8f;
    bonus.m_sightRangeScalar = 1.25f;
    bonus.m_bombardment = 1;
    bonus.m_validKindOf = MAKE_KINDOF_MASK(KINDOF_VEHICLE);
    bonus.m_invalidKindOf.clear();
    a->changeBattlePlan(PLANSTATUS_BOMBARDMENT, 1, &bonus);
    Object* veh = make(w, "veh", "PlanVeh", w.team_a, 100.0f, 50.0f);
    Object* inf = make(w, "inf", "PlanInf", w.team_a, 110.0f, 50.0f);
    Object* other = make(w, "other", "PlanVeh", w.team_b, 120.0f, 50.0f);
    plan_row("spawn", veh);
    plan_row("spawn", inf);
    plan_row("spawn", other);
    veh->setTeam(w.team_b);
    plan_row("to-b", veh);
    veh->setTeam(w.team_a);
    plan_row("to-a", veh);
    a->changeBattlePlan(PLANSTATUS_BOMBARDMENT, -1, &bonus);
    plan_row("removed", veh);
}

// ---- mines on construction ------------------------------------------------------

static void mine_row(const char* op, Object* o) {
    row(std::string("mine ") + op + " " + who(o->getID()) + " " + num(o->isDestroyed()) + " " +
        num(o->isEffectivelyDead()) + " " + bits(o->getBodyModule()->getHealth()));
}

static void construction_mines(oracle_wp51::World& w) {
    w.set_frame(200);
    for (int round = 0; round < 3; ++round) {
        const Real x = 200.0f + 100.0f * round;
        const std::string tag = num(round);
        std::vector<Object*> mines;
        mines.push_back(make(w, ("enemy" + tag).c_str(), "Mine", w.team_b, x + 5, 205));
        mines.push_back(make(w, ("own" + tag).c_str(), "Mine", w.team_a, x - 5, 205));
        mines.push_back(make(w, ("neutral" + tag).c_str(), "Mine", w.team_neutral, x, 195));
        mines.push_back(make(w, ("edge" + tag).c_str(), "Mine", w.team_b, x + 22, 200));
        mines.push_back(make(w, ("far" + tag).c_str(), "Mine", w.team_b, x + 40, 200));
        mines.push_back(make(w, ("rock" + tag).c_str(), "Rock", w.team_b, x, 205));
        Object* bld = make(w, ("bld" + tag).c_str(), "Bld", w.team_a, x, 200);
        if (round == 2) {
            // Already under construction: only clearing it is a change.
            for (size_t i = 0; i < mines.size(); ++i) mine_row(("placed" + tag).c_str(), mines[i]);
        }
        ThePartitionManager->update();
        bld->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
        if (round == 0) {
            for (size_t i = 0; i < mines.size(); ++i) mine_row("set", mines[i]);
        } else if (round == 1) {
            std::vector<Object*> late;
            late.push_back(make(w, "late-enemy", "Mine", w.team_b, x + 8, 195));
            late.push_back(make(w, "late-own", "Mine", w.team_a, x - 8, 195));
            ThePartitionManager->update();
            bld->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
            for (size_t i = 0; i < late.size(); ++i) mine_row("same", late[i]);
            bld->clearStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
            for (size_t i = 0; i < late.size(); ++i) mine_row("cleared", late[i]);
        } else {
            for (size_t i = 0; i < mines.size(); ++i) mine_row("set2", mines[i]);
        }
        w.flush_destroy_list();
    }
}

// ---- defection ------------------------------------------------------------------

static std::string player_of(Object* o) {
    return num(o->getControllingPlayer()->getPlayerIndex());
}

static void defection(oracle_wp51::World& w) {
    w.set_frame(300);
    Object* bld = make(w, "dbld", "Bld", w.team_a, 600, 400);
    Object* mine1 = make(w, "dmine1", "Mine", w.team_a, 640, 400);
    Object* mine2 = make(w, "dmine2", "Mine", w.team_a, 660, 400);
    Object* rock = make(w, "drock", "Rock", w.team_a, 680, 400);
    mine1->setProducer(bld);
    rock->setProducer(bld);
    bld->defect(w.team_a, 0);
    row("defect same " + player_of(bld) + " " + player_of(mine1) + " " + player_of(mine2) + " " + player_of(rock));
    bld->defect(w.team_b, 0);
    row("defect enemy " + player_of(bld) + " " + player_of(mine1) + " " + player_of(mine2) + " " + player_of(rock));
}

// ---- capture --------------------------------------------------------------------

static void capture_row(const char* op, oracle_wp51::World& w, Object* o) {
    ScoreKeeper* a = w.team_a->getControllingPlayer()->getScoreKeeper();
    ScoreKeeper* b = w.team_b->getControllingPlayer()->getScoreKeeper();
    row(std::string("capture ") + op + " " + player_of(o) + " " +
        num(a->getTotalTechBuildingsCaptured()) + " " + num(b->getTotalTechBuildingsCaptured()) + " " +
        num(o->testScriptStatusBit(OBJECT_STATUS_SCRIPT_UNSELLABLE)));
}

static void dozer_row(const char* op, Object* dozer) {
    DozerAIInterface* ai = dozer->getAIUpdateInterface()->getDozerAIInterface();
    row(std::string("dozer ") + op + " " + player_of(dozer) + " " + num(ai->getCurrentTask()) + " " +
        num(ai->isTaskPending(DOZER_TASK_REPAIR)) + " " + who(ai->getTaskTarget(DOZER_TASK_REPAIR)));
}

static void captures(oracle_wp51::World& w) {
    w.set_frame(400);
    TheGameLogic->enableScoring(TRUE);
    Dict dict;
    TheTeamFactory->initTeam("TeamA2", AsciiString::TheEmptyString, TRUE, &dict);
    Team* team_a2 = TheTeamFactory->findTeam("TeamA2");
    team_a2->setControllingPlayer(w.team_a->getControllingPlayer());

    Object* tech = make(w, "tech", "Tech", w.team_b, 700, 500);
    tech->setScriptStatus(OBJECT_STATUS_SCRIPT_UNSELLABLE);
    capture_row("start", w, tech);
    tech->setTeam(w.team_a);
    capture_row("to-a", w, tech);
    tech->setScriptStatus(OBJECT_STATUS_SCRIPT_UNSELLABLE);
    tech->setTeam(team_a2);
    capture_row("to-a2", w, tech);
    tech->setTeam(team_a2);
    capture_row("again", w, tech);

    Object* hurt = make(w, "hurt", "Tech", w.team_a, 750, 500);
    DamageInfo damage;
    damage.in.m_damageType = DAMAGE_UNRESISTABLE;
    damage.in.m_amount = 100.0f;
    hurt->attemptDamage(&damage);
    Object* dozer = make(w, "dozer", "Dozer", w.team_a, 760, 520);
    dozer->getAIUpdateInterface()->getDozerAIInterface()->newTask(DOZER_TASK_REPAIR, hurt);
    dozer_row("tasked", dozer);
    dozer->setTeam(team_a2);
    dozer_row("same-player", dozer);
    dozer->setTeam(w.team_b);
    dozer_row("enemy", dozer);
}

// ---- pathfind map ---------------------------------------------------------------

static void cells_row(const char* op) {
    std::string out = std::string("cells ") + op;
    for (int x = 60; x <= 90; ++x) {
        for (int y = 60; y <= 90; ++y) {
            PathfindCell* c = TheAI->pathfinder()->getCell(LAYER_GROUND, x, y);
            if (c && (c->getGoalUnit() != INVALID_ID || c->getPosUnit() != INVALID_ID)) {
                out += " " + num(x) + "." + num(y) + "." + who(c->getGoalUnit()) + "." + who(c->getPosUnit());
            }
        }
    }
    row(out);
}

static void reserve(Object* o, Real x, Real y) {
    Coord3D at = {x, y, 0.0f};
    TheAI->pathfinder()->updateGoal(o, &at, LAYER_GROUND);
    TheAI->pathfinder()->updatePos(o, o->getPosition());
}

static void pathfind(oracle_wp51::World& w) {
    w.set_frame(500);
    Object* gone = make(w, "gone", "Walker", w.team_a, 655, 655);
    reserve(gone, 705, 655);
    cells_row("reserved");
    TheGameLogic->destroyObject(gone);
    cells_row("destroyed");
    w.flush_destroy_list();
    cells_row("flushed");

    Object* rider = make(w, "rider", "Walker", w.team_a, 655, 755);
    Object* truck = make(w, "truck", "Truck", w.team_a, 805, 805);
    reserve(rider, 705, 755);
    cells_row("rider");
    truck->getContain()->addToContain(rider);
    cells_row("contained");
    truck->getContain()->removeFromContain(rider, FALSE);
    cells_row("removed");
}

int main(int argc, char** argv) {
    g_out = std::fopen(argc > 1 ? argv[1] : "docs/golden_vectors/PKT-390_lifecycle.txt", "w");
    if (!g_out) return 1;
    oracle_wp51::World w;
    w.load_ini(INI_TEXT);
    w.init_with_neutral_player();

    weapon_sets(w);
    battle_plans(w);
    construction_mines(w);
    defection(w);
    captures(w);
    pathfind(w);
    std::fclose(g_out);
    return 0;
}
