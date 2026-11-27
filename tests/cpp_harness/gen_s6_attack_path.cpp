// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for the path requests that end in the S6 searches: the
// unmodified reference AIUpdateInterface serving attack, approach, safe, stuck
// and move-away requests on the real pathfind map, with every search real
// (computeAttackPath, findAttackPath, findClosestPath, patchPath, findSafePath,
// getMoveAwayFromPath). doPathfind is called right after each request; the game
// does not step.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s6_attack_path.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/S6_AttackPath.json
//
// Not covered: bridges and the wall layer, aircraft that adjust destinations.

#include "PreRTS.h"

#include "oracle_wp51_world.h"

#include "Common/FramePacer.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/Recorder.h"
#include "Common/MessageStream.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/AIStateMachine.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/LocomotorSet.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Pathfinder/PathfindCell.h"
#include "GameLogic/Pathfinder/PathfindCellInfo.h"
#include "GameLogic/Pathfinder/Path.h"
#include "GameLogic/Pathfinder/PathNode.h"
#include "GameLogic/Weapon.h"

#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

#define S6_TAG(Tag, Type, Class, Member)                       \
    struct Tag { using type = Type Class::*; };                \
    template struct MemberAccess<Tag, &Class::Member>;         \
    Tag::type access_member(Tag);

S6_TAG(GoalCellTag, ICoord2D, AIUpdateInterface, m_pathfindGoalCell)
S6_TAG(CurCellTag, ICoord2D, AIUpdateInterface, m_pathfindCurCell)
S6_TAG(StampTag, UnsignedInt, AIUpdateInterface, m_pathTimestamp)
S6_TAG(WaitingTag, Bool, AIUpdateInterface, m_waitingForPath)
S6_TAG(IgnoreTag, UnsignedInt, AIUpdateInterface, m_ignoreCollisionsUntil)
S6_TAG(AttackTag, Bool, AIUpdateInterface, m_isAttackPath)
S6_TAG(ApproachTag, Bool, AIUpdateInterface, m_isApproachPath)
S6_TAG(SafeTag, Bool, AIUpdateInterface, m_isSafePath)
S6_TAG(StuckTag, Bool, AIUpdateInterface, m_isBlockedAndStuck)
S6_TAG(BlockedTag, Bool, AIUpdateInterface, m_isBlocked)
S6_TAG(BlockedFramesTag, Int, AIUpdateInterface, m_blockedFrames)
S6_TAG(RequestedTag, Coord3D, AIUpdateInterface, m_requestedDestination)
S6_TAG(ObstacleTag, ObjectID, AIUpdateInterface, m_ignoreObstacleID)
S6_TAG(MachineTag, AIStateMachine*, AIUpdateInterface, m_stateMachine)

static std::string hex32(uint32_t v) {
    char b[16];
    std::snprintf(b, sizeof(b), "%08x", v);
    return b;
}
static std::string fx(float f) {
    uint32_t u;
    std::memcpy(&u, &f, 4);
    return hex32(u);
}
static std::string c3(const Coord3D& c) { return fx(c.x) + "," + fx(c.y) + "," + fx(c.z); }
static std::string num(long long v) { return std::to_string(v); }
static uint64_t fnv(const std::string& s) {
    uint64_t h = 1469598103934665603ull;
    for (unsigned char c : s) {
        h ^= c;
        h *= 1099511628211ull;
    }
    return h;
}
static std::string hex64(uint64_t v) {
    char b[24];
    std::snprintf(b, sizeof(b), "%016llx", static_cast<unsigned long long>(v));
    return b;
}

static uint32_t g_state = 0x53364154u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return n ? g_state % n : 0;
}
static bool chance(int percent) { return static_cast<int>(pick(100)) < percent; }
static float between(float lo, float hi) { return lo + (hi - lo) * (static_cast<float>(pick(100000)) / 100000.0f); }

// --- terrain ----------------------------------------------------------------

static const int kCells = 32;

class S6Terrain : public oracle_wp51::OracleTerrainLogic {
public:
    std::string cliff = std::string(kCells * kCells, '0');
    S6Terrain() {
        extent.lo = {0.0f, 0.0f, 0.0f};
        extent.hi = {kCells * 10.0f, kCells * 10.0f, 100.0f};
    }
    // The base answers "blocked"; flat ground hides nothing.
    Bool isClearLineOfSight(const Coord3D&, const Coord3D&) const override { return TRUE; }
    Bool isCliffCell(Real x, Real y) const override {
        int i = static_cast<int>(std::floor(x / 10.0f));
        int j = static_cast<int>(std::floor(y / 10.0f));
        i = i < 0 ? 0 : (i >= kCells ? kCells - 1 : i);
        j = j < 0 ? 0 : (j >= kCells ? kCells - 1 : j);
        return cliff[j * kCells + i] == '1';
    }
};

static S6Terrain* g_terrain = nullptr;

// --- fixture ----------------------------------------------------------------

static std::string locomotor(const char* name, const char* surfaces, const char* appearance, int speed) {
    std::ostringstream s;
    s << "Locomotor " << name << "\n  Surfaces = " << surfaces << "\n  Appearance = " << appearance
      << "\n  Speed = " << speed << "\n  TurnRate = 240\n  Acceleration = 90\n  Braking = 90\n  MinSpeed = 0\nEnd\n";
    return s.str();
}

static std::string locomotors_ini() {
    return locomotor("S6_Legs", "GROUND", "TWO_LEGS", 30) + locomotor("S6_Wheels", "GROUND", "FOUR_WHEELS", 60) +
           locomotor("S6_Treads", "GROUND", "TREADS", 45) + locomotor("S6_Air", "AIR", "FOUR_WHEELS", 90);
}

static std::string weapon(const char* name, int range, int minimum) {
    std::ostringstream s;
    s << "Weapon " << name << "\n  PrimaryDamage = 10\n  PrimaryDamageRadius = 0\n  AttackRange = " << range
      << "\n  MinimumAttackRange = " << minimum << "\n  DamageType = EXPLOSION\nEnd\n";
    return s.str();
}

static std::string weapons_ini() {
    return weapon("S6_Gun", 80, 0) + weapon("S6_Short", 25, 0) + weapon("S6_Knife", 5, 0) +
           weapon("S6_Mortar", 120, 40);
}

static std::string object_ini(const char* name, const char* kind_of, const char* locomotor, float radius,
                              const char* gun) {
    std::ostringstream s;
    s << "Object " << name << "\n  KindOf = " << kind_of << "\n  Geometry = CYLINDER\n"
      << "  GeometryMajorRadius = " << radius << "\n  GeometryHeight = 8\n  GeometryIsSmall = No\n"
      << "  VisionRange = 100\n";
    if (*gun) s << "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY " << gun << "\n  End\n";
    s << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n";
    if (*locomotor) {
        s << "  Behavior = PhysicsBehavior ModuleTag_Phys\n    Mass = 5\n  End\n"
          << "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
          << "  Locomotor = SET_NORMAL " << locomotor << "\n";
    }
    s << "End\n";
    return s.str();
}

static const char* const kTemplates[] = {"S6_Rifle", "S6_Scout",  "S6_Mortar", "S6_Stab",
                                         "S6_Flyer", "S6_Truck",  "S6_Block",  "S6_Fence"};
enum { K_RIFLE, K_SCOUT, K_MORTAR, K_STAB, K_FLYER, K_TRUCK, K_BLOCK, K_FENCE };

static std::string objects_ini() {
    return object_ini("S6_Rifle", "PRELOAD SELECTABLE INFANTRY ATTACK_NEEDS_LINE_OF_SIGHT", "S6_Legs", 4, "S6_Gun") +
           object_ini("S6_Scout", "PRELOAD SELECTABLE VEHICLE", "S6_Wheels", 5, "S6_Short") +
           object_ini("S6_Mortar", "PRELOAD SELECTABLE VEHICLE ATTACK_NEEDS_LINE_OF_SIGHT", "S6_Treads", 9,
                      "S6_Mortar") +
           object_ini("S6_Stab", "PRELOAD SELECTABLE INFANTRY ATTACK_NEEDS_LINE_OF_SIGHT", "S6_Legs", 4,
                      "S6_Knife") +
           object_ini("S6_Flyer", "PRELOAD SELECTABLE AIRCRAFT", "S6_Air", 6, "S6_Gun") +
           object_ini("S6_Truck", "PRELOAD SELECTABLE VEHICLE", "S6_Wheels", 8, "") +
           object_ini("S6_Block", "PRELOAD STRUCTURE IMMOBILE", "", 18, "") +
           object_ini("S6_Fence", "PRELOAD STRUCTURE IMMOBILE CAN_SEE_THROUGH_STRUCTURE", "", 12, "");
}

// --- observation ------------------------------------------------------------

static std::vector<ObjectID> g_ids;
static Object* unit(int i) { return TheGameLogic->findObjectByID(g_ids[i]); }

static std::string path_text(Path* path) {
    if (!path) return "-";
    std::vector<PathNode*> nodes;
    for (PathNode* n = path->getFirstNode(); n; n = n->getNext()) nodes.push_back(n);
    std::string s = num(static_cast<long long>(nodes.size()));
    for (PathNode* n : nodes) {
        int next = -1;
        for (size_t k = 0; k < nodes.size(); ++k)
            if (nodes[k] == n->getNextOptimized()) next = static_cast<int>(k);
        s += "/" + c3(*n->getPosition()) + "," + num(static_cast<int>(n->getLayer())) + "," + num(next);
    }
    return s;
}

// Fields are separated by '|' so a mismatch names the one that differs.
static std::string observe(int i) {
    Object* o = unit(i);
    AIUpdateInterface* ai = o->getAI();
    if (!ai) return c3(*o->getPosition());
    const ICoord2D goal = ai->*access_member(GoalCellTag());
    const ICoord2D cur = ai->*access_member(CurCellTag());
    std::string s = c3(*o->getPosition());
    s += "|" + num(ai->*access_member(AttackTag()) ? 1 : 0) + num(ai->*access_member(ApproachTag()) ? 1 : 0) +
         num(ai->*access_member(SafeTag()) ? 1 : 0) + num(ai->*access_member(WaitingTag()) ? 1 : 0) +
         num(ai->*access_member(StuckTag()) ? 1 : 0) + num(ai->*access_member(BlockedTag()) ? 1 : 0) + "," +
         num(ai->*access_member(BlockedFramesTag()));
    s += "|" + num(ai->*access_member(StampTag())) + "," + num(ai->*access_member(IgnoreTag()));
    s += "|" + c3(ai->*access_member(RequestedTag()));
    s += "|" + num(static_cast<int>(ai->*access_member(ObstacleTag())));
    s += "|" + num(static_cast<int>((ai->*access_member(MachineTag()))->getCurrentStateID()));
    s += "|" + num(goal.x) + "," + num(goal.y) + "," + num(cur.x) + "," + num(cur.y);
    s += "|" + path_text(ai->getPath());
    return s;
}

static std::string cells_digest() {
    std::string s;
    for (int y = 0; y < kCells; ++y)
        for (int x = 0; x < kCells; ++x) {
            PathfindCell* cell = TheAI->pathfinder()->getCell(LAYER_GROUND, x, y);
            if (!cell) continue;
            const int goal = static_cast<int>(cell->getGoalUnit());
            const int present = static_cast<int>(cell->getPosUnit());
            if (goal || present) s += num(x) + "," + num(y) + "," + num(goal) + "," + num(present) + ";";
        }
    return hex64(fnv(s));
}

// --- operations -------------------------------------------------------------

static Coord3D at(float x, float y) {
    Coord3D p;
    p.x = x;
    p.y = y;
    p.z = 0.0f;
    return p;
}

static void command(Object* o, AICommandType type, Object* target) {
    AICommandParms parms(type, CMD_FROM_AI);
    parms.m_obj = target;
    o->getAI()->aiDoCommand(&parms);
}

static std::string random_op(int movers, int units) {
    const int u = static_cast<int>(pick(movers));
    Object* o = unit(u);
    AIUpdateInterface* ai = o->getAI();
    int v = static_cast<int>(pick(units));
    if (v == u) v = (v + 1) % units;
    Object* other = unit(v);
    Coord3D to = at(between(15.0f, 305.0f), between(15.0f, 305.0f));
    if (chance(35)) {
        to = *other->getPosition();
        to.x += between(-15.0f, 15.0f);
        to.y += between(-15.0f, 15.0f);
    }
    const uint32_t r = pick(100);
    if (r < 12) {
        // Anywhere, often next to the other unit.
        const float angle = between(-3.14f, 3.14f);
        Coord3D p = to;
        p.z = o->isKindOf(KINDOF_AIRCRAFT) && chance(60) ? 40.0f : 0.0f;
        o->setOrientation(angle);
        o->setPosition(&p);
        return "pos " + num(u) + " " + c3(p) + " " + fx(angle);
    }
    if (r < 17 && other->getAI() && other->getAI()->getPath()) {
        command(o, AICMD_MOVE_AWAY_FROM_UNIT, other);
        return "away " + num(u) + " " + num(v);
    }
    command(o, AICMD_BUSY, nullptr);
    std::string op;
    if (r < 50) {
        ai->requestAttackPath(g_ids[v], other->getPosition());
        op = "attack " + num(u) + " " + num(v);
    } else if (r < 62) {
        ai->requestAttackPath(INVALID_ID, &to);
        op = "ground " + num(u) + " " + c3(to);
    } else if (r < 72) {
        ai->requestApproachPath(&to);
        op = "approach " + num(u) + " " + c3(to);
    } else if (r < 80) {
        ai->requestSafePath(g_ids[v]);
        op = "safe " + num(u) + " " + num(v);
    } else {
        const int stuck = chance(40) && ai->getPath() ? 1 : 0;
        ai->requestPath(&to, TRUE);
        if (stuck) ai->*access_member(StuckTag()) = TRUE;
        op = "move " + num(u) + " " + c3(to) + " " + num(stuck);
    }
    // A stuck unit asking again at once keeps the path it has.
    if ((r < 62) && chance(25) && ai->getPath()) {
        ai->*access_member(StuckTag()) = TRUE;
        ai->*access_member(BlockedTag()) = TRUE;
        ai->*access_member(BlockedFramesTag()) = 7;
        op += " stuck";
    }
    // The base is private; only a C-style cast reaches it.
    ai->doPathfind((PathfindServicesInterface*)TheAI->pathfinder());
    return op;
}

struct Spawn {
    int kind;
    int team;
    float x, y, z;
};

static void blob(std::string& table, int x, int y, int w, int h) {
    for (int j = y; j < y + h && j < kCells; ++j)
        for (int i = x; i < x + w && i < kCells; ++i) table[j * kCells + i] = '1';
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_s6_attack_path <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheFramePacer = new FramePacer;
    TheCommandList = NEW CommandList;
    TheCommandList->init();
    TheRecorder = new RecorderClass;
    TheRecorder->init();
    g_terrain = new S6Terrain;
    TheTerrainLogic = g_terrain;
    TheTerrainLogic->init();
    TheWritableGlobalData->m_gravity = -1.0f;
    TheWritableGlobalData->m_partitionCellSize = 20.0f;
    ThePartitionManager = NEW PartitionManager;
    ThePartitionManager->init();
    const_cast<TAiData*>(TheAI->getAiData())->m_repulsedDistance = 50.0f;
    const_cast<TAiData*>(TheAI->getAiData())->m_attackUsesLineOfSight = TRUE;
    w.load_ini(locomotors_ini());
    w.load_ini(weapons_ini());
    w.load_ini(objects_ini());
    TheWeaponStore->postProcessLoad();
    w.init_with_neutral_player();
    ThePlayerList->getNthPlayer(1)->setPlayerType(PLAYER_HUMAN, FALSE);
    ThePlayerList->getNthPlayer(2)->setPlayerType(PLAYER_COMPUTER, FALSE);

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* out = std::fopen(tmp.c_str(), "w");
    if (!out) return 1;
    auto escape = [](const std::string& text) {
        std::string escaped;
        for (char c : text) escaped += c == '\n' ? std::string("\\n") : std::string(1, c);
        return escaped;
    };

    const int kScenarios = 120;
    const int kRows = 24;
    std::fprintf(out,
                 "{\"vector_count\":%d,\"rows\":%d,\"cells\":%d,\"locomotors\":\"%s\",\"weapons\":\"%s\","
                 "\"objects\":\"%s\",\"vectors\":[\n",
                 kScenarios, kRows, kCells, escape(locomotors_ini()).c_str(), escape(weapons_ini()).c_str(),
                 escape(objects_ini()).c_str());

    for (int scenario = 0; scenario < kScenarios; ++scenario) {
        UnsignedInt frame = 100 + pick(3000);
        std::string& cliff = g_terrain->cliff;
        cliff.assign(kCells * kCells, '0');
        for (int n = static_cast<int>(pick(4)); n > 0; --n)
            blob(cliff, static_cast<int>(pick(28)), static_cast<int>(pick(28)), 1 + pick(8), 1 + pick(3));

        // Every third scene is crowded.
        const float lo = scenario % 3 == 0 ? 130.0f : 40.0f;
        const float hi = scenario % 3 == 0 ? 175.0f : 280.0f;
        std::vector<Spawn> spawns;
        const int movers = 2 + static_cast<int>(pick(3));
        for (int i = 0; i < movers; ++i) {
            const int kind = static_cast<int>(pick(5));
            spawns.push_back({kind, chance(60) ? 0 : 1, between(lo, hi), between(lo, hi),
                              kind == K_FLYER && chance(60) ? 40.0f : 0.0f});
        }
        for (int i = 1 + static_cast<int>(pick(3)); i > 0; --i)
            spawns.push_back({K_TRUCK, chance(50) ? 0 : 1, between(lo, hi), between(lo, hi), 0.0f});
        for (int i = static_cast<int>(pick(4)); i > 0; --i)
            spawns.push_back({chance(70) ? K_BLOCK : K_FENCE, chance(50) ? 0 : 1, between(60.0f, 260.0f),
                              between(60.0f, 260.0f), 0.0f});

        TheGameLogic->*access_member(Wp51GLFrameTag()) = frame;
        InitRandom(1 + scenario);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1000));
        Pathfinder* pf = TheAI->pathfinder();
        pf->reset();
        // The records outlive the map; start each scene from the same free list.
        PathfindCellInfo::allocateCellInfos();
        pf->newMap();

        g_ids.clear();
        std::ostringstream spawn_text;
        for (size_t i = 0; i < spawns.size(); ++i) {
            const Spawn& s = spawns[i];
            const ThingTemplate* tt = TheThingFactory->findTemplate(kTemplates[s.kind]);
            if (!tt) return 3;
            Object* o = TheThingFactory->newObject(tt, s.team ? w.team_b : w.team_a);
            Coord3D p = at(s.x, s.y);
            p.z = s.z;
            o->setPosition(&p);
            if (s.kind == K_BLOCK || s.kind == K_FENCE) pf->addObjectToPathfindMap(o);
            g_ids.push_back(o->getID());
            spawn_text << (i ? ";" : "") << s.kind << "," << s.team << "," << c3(p);
        }
        pf->processPathfindQueue();
        pf->processPathfindQueue();
        const int units = static_cast<int>(g_ids.size());
        std::fprintf(out, "%s{\"s\":%d,\"frame\":%u,\"spawns\":\"%s\",\"cliff\":\"%s\"}", scenario ? ",\n" : "",
                     scenario, static_cast<unsigned>(frame), spawn_text.str().c_str(), cliff.c_str());

        std::vector<std::string> last(units);
        for (int row = 0; row < kRows; ++row) {
            frame += chance(35) ? pick(3) : 3 + pick(60);
            TheGameLogic->*access_member(Wp51GLFrameTag()) = frame;
            const std::string op = random_op(movers, units);
            std::ostringstream line;
            line << "{\"s\":" << scenario << ",\"f\":" << frame << ",\"op\":\"" << op << "\",\"seed\":\""
                 << hex32(GetGameLogicRandomSeedCRC()) << "\",\"map\":\"" << cells_digest() << "\",\"u\":\"";
            for (int i = 0; i < units; ++i) {
                const std::string now = observe(i);
                line << (i ? "~" : "") << (now == last[i] ? std::string("=") : now);
                last[i] = now;
            }
            line << "\"}";
            std::fprintf(out, ",\n%s", line.str().c_str());
        }
        std::fflush(out);

        for (ObjectID id : g_ids)
            if (Object* o = TheGameLogic->findObjectByID(id)) TheGameLogic->destroyObject(o);
        w.flush_destroy_list();
        pf->processPathfindQueue();
        pf->processPathfindQueue();
        ThePartitionManager->update();
        if (TheGameLogic->getFirstObject()) return 4;
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return std::rename(tmp.c_str(), argv[1]) == 0 ? 0 : 1;
}
