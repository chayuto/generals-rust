// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for the construction of the dozer and worker AI modules (the part of
// AI plan slice S16 a game start needs): the unmodified reference DozerAIUpdate and
// WorkerAIUpdate constructed by `ThingFactory::newObject` in a small world.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s16_dozer_worker.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/S16_DozerWorker.json
//
// Observed per object: the logic random seed after it is made, the AI module's save
// bytes and CRC, the weapon set flag the machines set and the weapons chosen by it.
// The pathfinder's locomotor choice answers the first locomotor of the set.
//
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder30chooseBestLocomotorForPositionE17PathfindLayerEnumP12LocomotorSetPK7Coord3D
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder10updateGoalEP6ObjectPK7Coord3D17PathfindLayerEnum
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder9updatePosEP6ObjectPK7Coord3D
//
// Not covered: frame updates, orders, building, repairing, supply gathering, loading.

#include "PreRTS.h"

#include "oracle_wp51_world.h"

#include "Common/ActionManager.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/LocomotorSet.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/Weapon.h"

#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

// --- member access ----------------------------------------------------------

using LocoVector = std::vector<Locomotor*>;
struct LocoVectorTag { using type = LocoVector LocomotorSet::*; };
template struct MemberAccess<LocoVectorTag, &LocomotorSet::m_locomotors>;
inline LocoVector LocomotorSet::* access_member(LocoVectorTag);

// --- answers ----------------------------------------------------------------

Locomotor* Pathfinder::chooseBestLocomotorForPosition(PathfindLayerEnum, LocomotorSet* set, const Coord3D*) {
    LocoVector& all = set->*access_member(LocoVectorTag());
    return all.empty() ? nullptr : all[0];
}
void Pathfinder::updateGoal(Object*, const Coord3D*, PathfindLayerEnum) {}
void Pathfinder::updatePos(Object*, const Coord3D*) {}

// --- fixture ----------------------------------------------------------------

static uint32_t g_state = 0x9e3779b9u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}

static std::string hex32(uint32_t v) {
    char b[9];
    std::snprintf(b, sizeof(b), "%08x", v);
    return b;
}

static std::string locomotor(const char* name, int speed) {
    std::ostringstream s;
    s << "Locomotor " << name << "\n  Surfaces = GROUND\n  Appearance = FOUR_WHEELS\n  Speed = " << speed
      << "\n  TurnRate = 180\n  Acceleration = 60\n  Braking = 60\n  MinSpeed = 0\nEnd\n";
    return s.str();
}

static std::string locomotors_ini() { return locomotor("S16_Wheels", 60) + locomotor("S16_Fast", 90); }

static std::string weapon(const char* name, int damage) {
    std::ostringstream s;
    s << "Weapon " << name << "\n  PrimaryDamage = " << damage
      << "\n  AttackRange = 50\n  DelayBetweenShots = Min:100 Max:900\n  ClipSize = 3\n"
      << "  ClipReloadTime = 500\nEnd\n";
    return s.str();
}

static std::string weapons_ini() { return weapon("S16_Gun", 5) + weapon("S16_Sweeper", 7); }

static const char* const kWithMine =
    "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY S16_Gun\n  End\n"
    "  WeaponSet\n    Conditions = MINE_CLEARING_DETAIL\n    Weapon = PRIMARY S16_Sweeper\n  End\n";
static const char* const kOnlyNone = "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY S16_Gun\n  End\n";

static std::string object_ini(const char* name, const char* kind_of, const char* module, const char* fields,
                              const char* sets, const char* weapons) {
    std::ostringstream s;
    s << "Object " << name << "\n  KindOf = " << kind_of << "\n  Geometry = CYLINDER\n"
      << "  GeometryMajorRadius = 5\n  GeometryHeight = 8\n  GeometryIsSmall = No\n  VisionRange = 100\n"
      << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
      << "  Behavior = " << module << " ModuleTag_AI\n" << fields << "  End\n"
      << weapons << sets << "End\n";
    return s.str();
}

static const char* const kTemplates[] = {"S16_Dozer",       "S16_DozerBare", "S16_Worker",
                                         "S16_WorkerPlain", "S16_Unit",      "S16_DozerStuck"};
static const int kKinds = 6;

static const char* const kDozerFields =
    "    RepairHealthPercentPerSecond = 2%\n    BoredTime = 5000\n    BoredRange = 150\n";
static const char* const kWorkerFields =
    "    MaxBoxes = 4\n    RepairHealthPercentPerSecond = 3%\n    BoredTime = 7000\n    BoredRange = 90\n"
    "    SupplyCenterActionDelay = 300\n    SupplyWarehouseActionDelay = 400\n"
    "    SupplyWarehouseScanDistance = 250\n    UpgradedSupplyBoost = 8\n"
    "    AutoAcquireEnemiesWhenIdle = YES\n";

static std::string objects_ini() {
    const char* one = "  Locomotor = SET_NORMAL S16_Wheels\n";
    const char* two = "  Locomotor = SET_NORMAL S16_Wheels S16_Fast\n";
    return object_ini("S16_Dozer", "SELECTABLE VEHICLE DOZER", "DozerAIUpdate", kDozerFields, one, kWithMine) +
           object_ini("S16_DozerBare", "SELECTABLE VEHICLE DOZER", "DozerAIUpdate", "", two, "") +
           object_ini("S16_Worker", "SELECTABLE INFANTRY DOZER HARVESTER", "WorkerAIUpdate", kWorkerFields, two,
                      kWithMine) +
           object_ini("S16_WorkerPlain", "SELECTABLE INFANTRY DOZER", "WorkerAIUpdate", "", one, kOnlyNone) +
           object_ini("S16_Unit", "SELECTABLE VEHICLE", "AIUpdateInterface", "", one, kWithMine) +
           object_ini("S16_DozerStuck", "SELECTABLE VEHICLE DOZER", "DozerAIUpdate", "", "", kOnlyNone);
}

// --- observation ------------------------------------------------------------

static std::string observe(Object* o) {
    AIUpdateInterface* ai = o->getAI();
    uint32_t unused = 0;
    const std::string save = oracle_wp51::World::snapshot_save_hex(ai, &unused);
    const uint32_t crc = oracle_wp51::World::snapshot_crc(ai);
    std::ostringstream s;
    s << hex32(GetGameLogicRandomSeedCRC()) << "," << hex32(crc) << ","
      << (o->testWeaponSetFlag(WEAPONSET_MINE_CLEARING_DETAIL) ? 1 : 0) << ",";
    const Weapon* w = o->getWeaponInWeaponSlot(PRIMARY_WEAPON);
    s << (w ? w->getName().str() : "-") << "," << save;
    return s.str();
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_s16_dozer_worker <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheTerrainLogic = NEW oracle_wp51::OracleTerrainLogic;
    TheTerrainLogic->init();
    TheWritableGlobalData->m_gravity = -1.0f;
    TheActionManager = NEW ActionManager;
    w.load_ini(locomotors_ini());
    w.load_ini(weapons_ini());
    w.load_ini(objects_ini());
    w.init_with_neutral_player();

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* out = std::fopen(tmp.c_str(), "w");
    if (!out) return 1;
    auto escape = [](const std::string& text) {
        std::string escaped;
        for (char c : text) {
            if (c == '\n') escaped += "\\n";
            else escaped += c;
        }
        return escaped;
    };

    const int kScenarios = 160;
    std::fprintf(out, "{\"vector_count\":%d,\"locomotors\":\"%s\",\"weapons\":\"%s\",\"objects\":\"%s\",\"vectors\":[\n",
                 kScenarios, escape(locomotors_ini()).c_str(), escape(weapons_ini()).c_str(),
                 escape(objects_ini()).c_str());

    for (int scenario = 0; scenario < kScenarios; ++scenario) {
        const UnsignedInt frame = pick(4) == 0 ? 0 : 1 + pick(100000);
        const UnsignedInt seed = 1 + pick(1000000000);
        TheGameLogic->*access_member(Wp51GLFrameTag()) = frame;
        InitRandom(seed);
        const ObjectID first_id = static_cast<ObjectID>(1000);
        TheGameLogic->setObjectIDCounter(first_id);
        const int units = 1 + static_cast<int>(pick(4));
        std::ostringstream record;
        std::vector<ObjectID> ids;
        for (int i = 0; i < units; ++i) {
            const int kind = i == 0 ? scenario % kKinds : static_cast<int>(pick(kKinds));
            const int team = static_cast<int>(pick(4) == 0 ? 1 : 0);
            const ThingTemplate* tt = TheThingFactory->findTemplate(kTemplates[kind]);
            if (!tt) return 3;
            Object* o = TheThingFactory->newObject(tt, team ? w.team_b : w.team_a);
            if (!o || !o->getAI()) return 4;
            ids.push_back(o->getID());
            record << (i ? "#" : "") << kind << "," << team << "," << observe(o);
        }
        std::fprintf(out, "%s{\"i\":%d,\"frame\":%u,\"seed\":%u,\"units\":\"%s\"}", scenario ? ",\n" : "", scenario,
                     static_cast<unsigned>(frame), static_cast<unsigned>(seed), record.str().c_str());
        std::fflush(out);
        for (ObjectID id : ids)
            if (Object* o = TheGameLogic->findObjectByID(id)) TheGameLogic->destroyObject(o);
        w.flush_destroy_list();
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return std::rename(tmp.c_str(), argv[1]) == 0 ? 0 : 1;
}
