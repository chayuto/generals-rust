// SPDX-License-Identifier: GPL-3.0-or-later
// Trace oracle for the frame step (docs/AI_PORT_PLAN.md, slice S0): the unmodified
// reference `GameLogic::update` (GameLogic.cpp) and `AI::update` (AI.cpp) run a
// small real world, frame after frame.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s0_frame_driver.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/S0_FrameDriver.json
//
// The harness adds only virtual overrides: the subsystems `GameLogic::update`
// calls record the state when their turn comes (ScriptEngine, TerrainLogic,
// Recorder, one AIPlayer per player, BuildAssistant, PartitionManager,
// WeaponStore, VictoryConditions), so the order of the call sites is part of
// every vector. The AI is a subclass with no behaviour: its update records the
// frame and does what the vector's plan says (order itself or another unit, ask
// about idleness, destroy, damage, disable, wake, spawn, ask for a path). The
// other objects run the real, already ported modules.
//
// One vector is one logic frame: the operations applied before it, then what was
// observed during and after it. Observations carry only what changed:
//   <hook>{G frame rng queueHead queueTail sabotage0 sabotage1}{O id x y z angle
//   health disabled destroyed [waiting idle]}{-id}{L world list}
// and the last one (E) adds `name:nextCallFrame:crc:saveHash` per module.
//
// Not covered: scripts, commands, water, replay CRC messages, the sell list,
// delayed weapon damage and victory conditions run in this world with nothing to
// do (their hooks only show where they are called); the path queue itself and the
// searches (only which units had their path computed, and when); the brown-out
// that ends with a sabotage (no object is KINDOF POWERED); the order inside
// Player::update beyond the computer player's turn; physics against units; the
// pathfind map's own CRC and the game-wide CRC; saving and loading a world.

#include "oracle_wp51_world.h"

#include "Common/BuildAssistant.h"
#include "Common/Energy.h"
#include "Common/FramePacer.h"
#include "Common/MessageStream.h"
#include "Common/Module.h"
#include "Common/ModuleFactory.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/Recorder.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/AIPlayer.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/VictoryConditions.h"
#include "GameLogic/Weapon.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <set>
#include <sstream>
#include <string>
#include <vector>

// --- member access ----------------------------------------------------------

struct QueueHeadTag { using type = Int Pathfinder::*; };
template struct MemberAccess<QueueHeadTag, &Pathfinder::m_queuePRHead>;
QueueHeadTag::type access_member(QueueHeadTag);

struct QueueTailTag { using type = Int Pathfinder::*; };
template struct MemberAccess<QueueTailTag, &Pathfinder::m_queuePRTail>;
QueueTailTag::type access_member(QueueTailTag);

struct WaitingTag { using type = Bool AIUpdateInterface::*; };
template struct MemberAccess<WaitingTag, &AIUpdateInterface::m_waitingForPath>;
WaitingTag::type access_member(WaitingTag);

struct DestinationTag { using type = Coord3D AIUpdateInterface::*; };
template struct MemberAccess<DestinationTag, &AIUpdateInterface::m_requestedDestination>;
DestinationTag::type access_member(DestinationTag);

struct PlayerAiTag { using type = AIPlayer* Player::*; };
template struct MemberAccess<PlayerAiTag, &Player::m_ai>;
PlayerAiTag::type access_member(PlayerAiTag);

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

// --- random for the scenario (never the engine's) ----------------------------

static uint32_t g_state = 0x50f4a3e1u;
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
    "Locomotor S0_Loco\n  Surfaces = GROUND\n  Speed = 30\n  TurnRate = 180\n"
    "  Acceleration = 30\n  Braking = 30\n  MinSpeed = 0\n  Appearance = TWO_LEGS\nEnd\n";

static std::string body(int health) {
    std::ostringstream s;
    s << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = " << health
      << "\n    InitialHealth = " << health << "\n  End\n";
    return s.str();
}

static std::string head(const char* name, const char* kind_of, int radius, int height) {
    std::ostringstream s;
    s << "Object " << name << "\n  KindOf = " << kind_of << "\n  Geometry = CYLINDER\n"
      << "  GeometryMajorRadius = " << radius << "\n  GeometryHeight = " << height
      << "\n  GeometryIsSmall = No\n";
    return s.str();
}

static const char* const kTemplates[] = {
    "S0_Ai", "S0_Crate", "S0_Fader", "S0_Healer", "S0_Dier", "S0_Spy",
};
static const int kKinds = 6;
static const int kAi = 0;
static const int kCrate = 1;

static std::string fixture_ini() {
    std::ostringstream s;
    s << head("S0_Ai", "PRELOAD SELECTABLE INFANTRY", 4, 8) << body(200)
      << "  Behavior = OracleAI ModuleTag_AI\n  End\n"
      << "  Locomotor = SET_NORMAL S0_Loco\nEnd\n";
    s << head("S0_Crate", "PRELOAD", 6, 8) << body(100)
      << "  Behavior = PhysicsBehavior ModuleTag_Phys\n    Mass = 5\n    AllowBouncing = Yes\n"
      << "    AllowCollideForce = Yes\n    KillWhenRestingOnGround = No\n  End\n"
      << "  Behavior = LifetimeUpdate ModuleTag_Life\n    MinLifetime = 400\n"
      << "    MaxLifetime = 4000\n  End\n"
      << "  Behavior = DestroyDie ModuleTag_Die\n  End\nEnd\n";
    s << head("S0_Fader", "PRELOAD", 3, 4) << body(50)
      << "  Behavior = DeletionUpdate ModuleTag_Delete\n    MinLifetime = 33\n"
      << "    MaxLifetime = 700\n  End\nEnd\n";
    s << head("S0_Healer", "PRELOAD STRUCTURE", 10, 10) << body(300)
      << "  Behavior = AutoHealBehavior ModuleTag_Heal\n    HealingAmount = 7\n"
      << "    HealingDelay = 100\n    StartsActive = Yes\n    StartHealingDelay = 133\n  End\nEnd\n";
    s << head("S0_Dier", "PRELOAD", 5, 6) << body(60)
      << "  Behavior = SlowDeathBehavior ModuleTag_SlowDeath\n    SinkDelay = 100\n"
      << "    SinkDelayVariance = 66\n    SinkRate = 3\n    DestructionDelay = 400\n"
      << "    DestructionDelayVariance = 200\n  End\n"
      << "  Behavior = LifetimeUpdate ModuleTag_Life\n    MinLifetime = 200\n"
      << "    MaxLifetime = 900\n  End\nEnd\n";
    // Hears of disabled edges.
    s << head("S0_Spy", "PRELOAD STRUCTURE", 8, 8) << body(120)
      << "  Behavior = SpyVisionUpdate ModuleTag_Spy\n    NeedsUpgrade = No\n"
      << "    SelfPowered = Yes\n    SelfPoweredDuration = 200\n"
      << "    SelfPoweredInterval = 300\n  End\nEnd\n";
    return s.str();
}

static const char* const kModules[] = {
    "ActiveBody",       "PhysicsBehavior",   "LifetimeUpdate", "DeletionUpdate",
    "AutoHealBehavior", "SlowDeathBehavior", "DestroyDie",     "AIUpdateInterface",
    "SpyVisionUpdate",
};

// --- scenario ---------------------------------------------------------------

struct Action { int code = 0; uint32_t a = 0, b = 0; };
struct Plan { uint32_t sleep = 1; std::vector<Action> actions; };

static std::vector<ObjectID> g_ids;          // objects of the scenario, in spawn order
static std::vector<int> g_kinds;
static std::map<ObjectID, Plan> g_plans;
static std::set<ObjectID> g_not_idle;
static std::ostringstream g_out;
static Team* g_teams[2];
static std::map<std::string, std::string> g_seen;  // delta state
static std::string g_seen_global, g_seen_list;
static int g_action_counts[16];

static Object* by_index(uint32_t i) {
    if (g_ids.empty()) return nullptr;
    return TheGameLogic->findObjectByID(g_ids[i % g_ids.size()]);
}

static Coord3D at(float x, float y, float z) {
    Coord3D c;
    c.set(x, y, z);
    return c;
}

static void spawn(int kind, float x, float y, float z, float angle, int team) {
    const ThingTemplate* tt = TheThingFactory->findTemplate(kTemplates[kind]);
    if (!tt) die("missing template");
    Object* o = TheThingFactory->newObject(tt, g_teams[team]);
    if (!o) die("no object");
    Coord3D p = at(x, y, z);
    o->setPosition(&p);
    o->setOrientation(angle);
    g_ids.push_back(o->getID());
    g_kinds.push_back(kind);
}

static void damage(Object* o, float amount, int type) {
    DamageInfo d;
    d.in.m_amount = amount;
    d.in.m_damageType = type ? DAMAGE_UNRESISTABLE : DAMAGE_EXPLOSION;
    d.in.m_deathType = DEATH_NORMAL;
    d.in.m_sourceID = INVALID_ID;
    o->attemptDamage(&d);
}

static void queue_path(Object* o) {
    AIUpdateInterface* ai = o ? o->getAIUpdateInterface() : nullptr;
    if (!o) return;
    if (ai) {
        Coord3D dest = *o->getPosition();
        dest.x += 30.0f;
        dest.y += 20.0f;
        ai->*access_member(DestinationTag()) = dest;
        ai->*access_member(WaitingTag()) = TRUE;
    }
    if (!TheAI->pathfinder()->queueForPath(o->getID())) die("queue full");
}

static const DisabledType kDisabled[] = {
    DISABLED_PARALYZED, DISABLED_EMP,          DISABLED_HELD,
    DISABLED_HACKED,    DISABLED_UNDERPOWERED, DISABLED_DEFAULT,
};
static const int kDisabledCount = 6;

// --- recording AI -----------------------------------------------------------

class RecAI : public AIUpdateInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(RecAI, "OracleFrameRecAI", 64, 64)
public:
    RecAI(Thing* thing, const ModuleData* data) : AIUpdateInterface(thing, data) {}
    Bool isIdle() const override { return g_not_idle.count(getObject()->getID()) == 0; }
    Bool processCollision(PhysicsBehavior*, Object*) override { return FALSE; }

    void aiDoCommand(const AICommandParms* p) override {
        if (p->m_cmd != AICMD_ATTACK_POSITION) die("unexpected AI command");
        g_not_idle.insert(getObject()->getID());
        g_out << "[o " << getObject()->getID() << ' ' << bits(p->m_pos.x) << ' ' << p->m_intValue
              << ']';
    }

    UpdateSleepTime update() override {
        const ObjectID me = getObject()->getID();
        g_out << "[u " << me << ' ' << TheGameLogic->getFrame() << ']';
        Plan plan;
        std::map<ObjectID, Plan>::iterator it = g_plans.find(me);
        if (it != g_plans.end()) {
            plan = it->second;
            g_plans.erase(it);
        }
        for (const Action& act : plan.actions) {
            ++g_action_counts[act.code];
            Object* self = getObject();
            Object* other = by_index(act.a);
            switch (act.code) {
            case 1:    // an order to itself, from inside its own update
            case 3: {  // an order to another unit
                Object* target = act.code == 1 ? self : other;
                AIUpdateInterface* ai = target ? target->getAIUpdateInterface() : nullptr;
                if (ai) {
                    Coord3D p = at(real_of(act.b), 2.0f, 3.0f);
                    ai->aiAttackPosition(&p, static_cast<Int>(act.a), CMD_FROM_AI);
                }
                break;
            }
            case 2:     // what does it answer about itself while it runs
            case 11: {  // and what does another unit answer
                Object* target = act.code == 2 ? self : other;
                AIUpdateInterface* ai = target ? target->getAIUpdateInterface() : nullptr;
                if (ai)
                    g_out << "[q " << target->getID() << ' ' << (ai->isIdle() ? 1 : 0) << ']';
                break;
            }
            case 4:
                TheGameLogic->destroyObject(self);
                break;
            case 5:
                if (other) TheGameLogic->destroyObject(other);
                break;
            case 6:
                if (other)
                    other->setDisabledUntil(kDisabled[act.b % kDisabledCount],
                                            TheGameLogic->getFrame() + act.b / kDisabledCount);
                break;
            case 7:
                if (other) damage(other, static_cast<float>(act.b), 1);
                break;
            case 8:
                queue_path(self);
                break;
            case 9:
                g_not_idle.erase(me);
                break;
            case 10:
                if (other && other != self && other->getAIUpdateInterface())
                    TheGameLogic->friend_awakenUpdateModule(
                        other, other->getAIUpdateInterface(), TheGameLogic->getFrame() + act.b);
                break;
            case 12: {
                const int kind = static_cast<int>(act.a % kKinds);
                const float x = (kind == kAi ? 600.0f : 150.0f) + static_cast<float>(act.b % 97);
                const float y = 200.0f + static_cast<float>(act.b % 89);
                spawn(kind, x, y, 0.0f, 0.0f, 0);
                g_out << "[s " << g_ids.back() << ']';
                break;
            }
            default:
                die("unknown action");
            }
        }
        g_out << "[x " << me << ']';
        return plan.sleep >= UPDATE_SLEEP_FOREVER ? UPDATE_SLEEP_FOREVER
                                                  : UPDATE_SLEEP(static_cast<Int>(plan.sleep));
    }
};
RecAI::~RecAI() {}

static Module* make_rec_ai(Thing* t, const ModuleData* d) { return newInstance(RecAI)(t, d); }

// --- observation ------------------------------------------------------------

static uint32_t fnv(const std::string& text) {
    uint32_t h = 2166136261u;
    for (char c : text) h = (h ^ static_cast<unsigned char>(c)) * 16777619u;
    return h;
}

static std::string object_state(Object* o, bool full) {
    std::ostringstream s;
    s << bits(o->getPosition()->x) << ' ' << bits(o->getPosition()->y) << ' '
      << bits(o->getPosition()->z) << ' ' << bits(o->getOrientation()) << ' '
      << bits(o->getBodyModule()->getHealth()) << ' ';
    uint32_t mask = 0;
    for (int i = 0; i < DISABLED_COUNT; ++i)
        if (o->isDisabledByType(static_cast<DisabledType>(i))) mask |= 1u << i;
    s << mask << ' ' << (o->isDestroyed() ? 1 : 0);
    if (AIUpdateInterface* ai = o->getAIUpdateInterface())
        s << ' ' << (ai->*access_member(WaitingTag()) ? 1 : 0) << ' ' << (ai->isIdle() ? 1 : 0);
    if (!full) return s.str();
    for (const char* name : kModules) {
        const NameKeyType key = NAMEKEY(name);
        for (BehaviorModule** it = o->getBehaviorModules(); *it; ++it) {
            BehaviorModule* b = *it;
            if (b->getModuleNameKey() != key) continue;
            s << ' ' << name << ':';
            if (b->getUpdate()) s << static_cast<UpdateModule*>(b)->friend_getNextCallFrame();
            s << ':';
            // The AI has no state of its own here: only its schedule is compared.
            if (std::strcmp(name, "AIUpdateInterface") != 0)
                s << oracle_wp51::World::snapshot_crc(b) << ':'
                  << fnv(oracle_wp51::World::snapshot_save_hex(b));
        }
    }
    return s.str();
}

static void observe(const char* hook, bool full = false) {
    g_out << hook;
    std::ostringstream global;
    Pathfinder* pf = TheAI->pathfinder();
    global << TheGameLogic->getFrame() << ' ' << GetGameLogicRandomSeedCRC() << ' '
           << pf->*access_member(QueueHeadTag()) << ' ' << pf->*access_member(QueueTailTag());
    for (int p = 0; p < 2; ++p)
        global << ' ' << ThePlayerList->getNthPlayer(p)->getEnergy()->getPowerSabotagedTillFrame();
    if (global.str() != g_seen_global) {
        g_seen_global = global.str();
        g_out << "{G " << g_seen_global << '}';
    }
    std::set<std::string> present;
    std::ostringstream list;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        const std::string id = std::to_string(static_cast<int>(o->getID()));
        present.insert(id);
        list << ' ' << id;
        const std::string state = object_state(o, false);
        std::string& seen = g_seen[id];
        if (seen != state) {
            seen = state;
            g_out << "{O " << id << ' ' << state << '}';
        }
        if (full) {
            const std::string all = object_state(o, true);
            std::string& was = g_seen["F" + id];
            if (was != all) {
                was = all;
                g_out << "{F " << id << all.substr(state.size()) << '}';
            }
        }
    }
    std::vector<std::string> gone;
    for (std::map<std::string, std::string>::iterator it = g_seen.begin(); it != g_seen.end(); ++it)
        if (it->first[0] != 'F' && !present.count(it->first)) gone.push_back(it->first);
    for (const std::string& id : gone) {
        g_out << "{-" << id << '}';
        g_seen.erase(id);
        g_seen.erase("F" + id);
    }
    if (list.str() != g_seen_list) {
        g_seen_list = list.str();
        g_out << "{L" << g_seen_list << '}';
    }
}

// --- hooks: the subsystems GameLogic::update calls ----------------------------

class RecScript : public ScriptEngine {
public:
    void update() override { observe("S"); ScriptEngine::update(); }
};
class RecTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    void update() override { observe("T"); OracleTerrainLogic::update(); }
};
class RecRecorder : public RecorderClass {
public:
    void update() override { observe("R"); RecorderClass::update(); }
};
class RecAIPlayer : public AIPlayer {
public:
    int index;
    static void* operator new(size_t n) { return std::malloc(n); }
    static void operator delete(void* p) { std::free(p); }
    RecAIPlayer(Player* p, int i) : AIPlayer(p), index(i) {}
    MemoryPool* getObjectMemoryPool() override { return nullptr; }
    void update() override { observe(index == 0 ? "A0" : "A1"); }
};
class RecBuild : public BuildAssistant {
public:
    void update() override { observe("B"); BuildAssistant::update(); }
};
class RecPartition : public PartitionManager {
public:
    void update() override { observe("P"); PartitionManager::update(); }
};
class RecWeapons : public WeaponStore {
public:
    void update() override { observe("W"); WeaponStore::update(); }
};
class RecVictory : public VictoryConditionsInterface {
public:
    VictoryConditionsInterface* real;
    explicit RecVictory(VictoryConditionsInterface* r) : real(r) {}
    void init() override { real->init(); }
    void reset() override { real->reset(); }
    void update() override { observe("V"); real->update(); }
    Bool hasAchievedVictory(Player* p) override { return real->hasAchievedVictory(p); }
    Bool hasBeenDefeated(Player* p) override { return real->hasBeenDefeated(p); }
    Bool hasSinglePlayerBeenDefeated(Player* p) override {
        return real->hasSinglePlayerBeenDefeated(p);
    }
    void cachePlayerPtrs() override { real->cachePlayerPtrs(); }
    Bool isLocalAlliedVictory() override { return real->isLocalAlliedVictory(); }
    Bool isLocalAlliedDefeat() override { return real->isLocalAlliedDefeat(); }
    Bool isLocalDefeat() override { return real->isLocalDefeat(); }
    Bool amIObserver() override { return real->amIObserver(); }
    UnsignedInt getEndFrame() override { return real->getEndFrame(); }
};

// --- operations between frames ------------------------------------------------

static bool g_frozen = false;
// Clears for the next frame: one state is lifted while another has just run out.
static std::vector<std::pair<uint32_t, uint32_t> > g_clears;
static int g_op_counts[16];

static std::string op(int code, const std::vector<uint32_t>& v) {
    ++g_op_counts[code];
    std::ostringstream s;
    s << code;
    for (uint32_t x : v) s << ' ' << x;
    switch (code) {
    case 0:  // spawn kind x y z angle team
        spawn(static_cast<int>(v[0]), real_of(v[1]), real_of(v[2]), real_of(v[3]), real_of(v[4]),
              static_cast<int>(v[5]));
        break;
    case 1: {  // plan: object sleep {code a b}*
        Object* o = by_index(v[0]);
        if (o && o->getAIUpdateInterface()) {
            Plan p;
            p.sleep = v[1];
            for (size_t i = 2; i + 2 < v.size(); i += 3) {
                Action a;
                a.code = static_cast<int>(v[i]);
                a.a = v[i + 1];
                a.b = v[i + 2];
                p.actions.push_back(a);
            }
            g_plans[o->getID()] = p;
        }
        break;
    }
    case 2:    // physics force
    case 3: {  // physics shock
        Object* o = by_index(v[0]);
        if (o && o->getPhysics()) {
            Coord3D f = at(real_of(v[1]), real_of(v[2]), real_of(v[3]));
            if (code == 2) o->getPhysics()->applyForce(&f);
            else o->getPhysics()->applyShock(&f);
        }
        break;
    }
    case 4:  // damage object amount unresistable
        if (Object* o = by_index(v[0])) damage(o, real_of(v[1]), static_cast<int>(v[2]));
        break;
    case 5:
        if (Object* o = by_index(v[0])) TheGameLogic->destroyObject(o);
        break;
    case 6:  // disable object type frames
        if (Object* o = by_index(v[0]))
            o->setDisabledUntil(kDisabled[v[1]], TheGameLogic->getFrame() + v[2]);
        break;
    case 7:
        if (Object* o = by_index(v[0])) o->clearDisabled(kDisabled[v[1]]);
        break;
    case 8:  // sabotage player frames
        ThePlayerList->getNthPlayer(static_cast<Int>(v[0]))->getEnergy()->setPowerSabotagedTillFrame(
            TheGameLogic->getFrame() + v[1]);
        break;
    case 9:
        queue_path(by_index(v[0]));
        break;
    case 10:
        g_frozen = v[0] != 0;
        if (g_frozen) TheScriptEngine->doFreezeTime();
        else TheScriptEngine->doUnfreezeTime();
        break;
    default:
        die("unknown op");
    }
    return s.str();
}

static uint32_t random_sleep() {
    const uint32_t r = pick(10);
    if (r < 5) return 1;
    if (r < 9) return 2 + pick(4);
    return UPDATE_SLEEP_FOREVER;
}

static void random_action(std::vector<uint32_t>& v) {
    static const int kCodes[] = {1, 1, 1, 2, 2, 3, 3, 4, 5, 6, 6, 7, 8, 8, 9, 10, 10, 11, 12};
    const int code = kCodes[pick(sizeof(kCodes) / sizeof(kCodes[0]))];
    uint32_t a = pick(64), b = pick(1000);
    if (code == 1 || code == 3) b = bits(real_in(-50.0f, 50.0f));
    if (code == 6) b = pick(kDisabledCount) + kDisabledCount * pick(6);
    if (code == 7) b = pick(3) == 0 ? 1000 : 1 + pick(40);
    if (code == 10) b = 1 + pick(4);
    v.push_back(static_cast<uint32_t>(code));
    v.push_back(a);
    v.push_back(b);
}

static std::vector<uint32_t> random_spawn(int kind) {
    // Units stand apart from the crates; crates start close enough to collide.
    const float x = kind == kAi ? real_in(600.0f, 700.0f)
                    : kind == kCrate ? real_in(200.0f, 240.0f) : real_in(300.0f, 500.0f);
    const float y = kind == kCrate ? real_in(200.0f, 240.0f) : real_in(200.0f, 600.0f);
    const float z = kind == kCrate && pick(2) ? real_in(0.0f, 40.0f) : 0.0f;
    return {static_cast<uint32_t>(kind), bits(x), bits(y), bits(z),
            bits(pick(3) ? real_in(-3.1f, 3.1f) : 0.0f), pick(2)};
}

static std::string random_op() {
    const uint32_t r = pick(100);
    const uint32_t n = static_cast<uint32_t>(g_ids.size());
    const uint32_t obj = n ? pick(n) : 0;
    if (r < 10 && n < 24) return op(0, random_spawn(static_cast<int>(pick(kKinds))));
    if (r < 45) {
        // Plans go to units; pick one if there is any.
        std::vector<uint32_t> units;
        for (uint32_t i = 0; i < n; ++i)
            if (g_kinds[i] == kAi && by_index(i)) units.push_back(i);
        if (units.empty()) return op(0, random_spawn(kAi));
        std::vector<uint32_t> v = {units[pick(static_cast<uint32_t>(units.size()))], random_sleep()};
        const uint32_t count = pick(4);
        for (uint32_t i = 0; i < count; ++i) random_action(v);
        return op(1, v);
    }
    if (r < 58)
        return op(2 + pick(2), {obj, bits(real_in(-30.0f, 30.0f)), bits(real_in(-30.0f, 30.0f)),
                                bits(real_in(-5.0f, 30.0f))});
    if (r < 66)
        return op(4, {obj, bits(pick(3) == 0 ? 1000.0f : real_in(1.0f, 80.0f)), pick(2)});
    if (r < 70) return op(5, {obj});
    if (r < 79) return op(6, {obj, pick(kDisabledCount), pick(7)});
    if (r < 80) return op(7, {obj, pick(kDisabledCount)});
    if (r < 83) {
        const uint32_t a = pick(kDisabledCount);
        const uint32_t b = (a + 1 + pick(kDisabledCount - 1)) % kDisabledCount;
        g_clears.push_back(std::make_pair(obj, a));
        const std::string first = op(6, {obj, a, 5});
        return first + ';' + op(6, {obj, b, 1});
    }
    if (r < 89) return op(8, {pick(2), pick(6)});
    if (r < 97) return op(9, {obj});
    return op(10, {g_frozen ? 0u : 1u});
}

// --- main -------------------------------------------------------------------

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_s0_frame_driver <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;

    TheFramePacer = new FramePacer;
    TheCommandList = NEW CommandList;
    TheCommandList->init();
    TheRecorder = new RecRecorder;
    TheRecorder->init();
    TheTerrainLogic = new RecTerrain;
    TheTerrainLogic->init();
    TheScriptEngine = new RecScript;
    TheScriptEngine->init();
    TheBuildAssistant = new RecBuild;
    TheBuildAssistant->init();
    ThePartitionManager = new RecPartition;
    ThePartitionManager->init();
    TheWeaponStore = new RecWeapons;
    TheWeaponStore->init();
    TheVictoryConditions = new RecVictory(TheVictoryConditions);

    (TheModuleFactory->*(&ExposedFactory::addModuleInternal))(
        make_rec_ai, AIUpdateInterface::friend_newModuleData, MODULETYPE_BEHAVIOR, "OracleAI",
        AIUpdateInterface::getInterfaceMask());
    w.load_ini(kLocomotors);
    const std::string fixture = fixture_ini();
    w.load_ini(fixture);
    w.init_with_neutral_player();
    // init_with_neutral_player: player 0 is neutral, 1 controls team A, 2 team B.
    g_teams[0] = w.team_a;
    g_teams[1] = w.team_b;
    for (int p = 0; p < 2; ++p)
        ThePlayerList->getNthPlayer(p)->*access_member(PlayerAiTag()) =
            new RecAIPlayer(ThePlayerList->getNthPlayer(p), p);

    FILE* out = std::fopen(argv[1], "w");
    if (!out) return 1;
    std::string escaped = fixture, locos = kLocomotors;
    for (char& c : escaped)
        if (c == '\n') c = '~';
    for (char& c : locos)
        if (c == '\n') c = '~';

    const int kScenarios = 80;
    const int kFrames = 40;
    std::fprintf(out,
                 "{\"vector_count\":%d,\"frames_per_scenario\":%d,\"locomotors\":\"%s\","
                 "\"objects\":\"%s\",\"vectors\":[",
                 kScenarios * kFrames, kFrames, locos.c_str(), escaped.c_str());

    int frozen_frames = 0;
    for (int scenario = 0; scenario < kScenarios; ++scenario) {
        const UnsignedInt first = 1 + pick(3000);
        const UnsignedInt seed = 1 + pick(1000000000);
        w.set_frame(first);
        InitRandom(seed);
        g_ids.clear();
        g_kinds.clear();
        g_plans.clear();
        g_not_idle.clear();
        g_seen.clear();
        g_seen_global.clear();
        g_seen_list.clear();
        g_frozen = false;
        g_clears.clear();
        TheScriptEngine->doUnfreezeTime();
        Pathfinder* pf = TheAI->pathfinder();
        pf->*access_member(QueueHeadTag()) = 0;
        pf->*access_member(QueueTailTag()) = 0;
        for (int p = 0; p < 2; ++p)
            ThePlayerList->getNthPlayer(p)->getEnergy()->setPowerSabotagedTillFrame(0);

        UnsignedInt base = 0;
        for (int f = 0; f < kFrames; ++f) {
            std::vector<std::string> ops;
            if (f == 0) {
                const int count = 6 + static_cast<int>(pick(7));
                for (int i = 0; i < count; ++i) {
                    const int kind = i < 2 ? kAi : i < 5 ? kCrate : static_cast<int>(pick(kKinds));
                    ops.push_back(op(0, random_spawn(kind)));
                }
                base = static_cast<UnsignedInt>(g_ids.front());
            }
            for (const std::pair<uint32_t, uint32_t>& c : g_clears)
                ops.push_back(op(7, {c.first, c.second}));
            g_clears.clear();
            const int count = static_cast<int>(pick(4));
            for (int i = 0; i < count; ++i) ops.push_back(random_op());
            // A freeze ends after a few frames.
            if (g_frozen && pick(3) == 0) ops.push_back(op(10, {0u}));
            frozen_frames += g_frozen ? 1 : 0;

            g_out.str("");
            TheGameLogic->update();
            observe("E", true);

            std::fprintf(out, "%s\n{\"scenario\":%d,\"frame\":%d,", scenario || f ? "," : "",
                         scenario, f);
            if (f == 0)
                std::fprintf(out, "\"first\":%u,\"seed\":%u,\"id\":%u,", first, seed, base);
            std::fprintf(out, "\"ops\":\"");
            for (size_t k = 0; k < ops.size(); ++k)
                std::fprintf(out, "%s%s", k ? ";" : "", ops[k].c_str());
            std::fprintf(out, "\",\"out\":\"%s\"}", g_out.str().c_str());
        }

        g_plans.clear();
        if (g_frozen) TheScriptEngine->doUnfreezeTime();
        for (ObjectID id : g_ids)
            if (Object* o = TheGameLogic->findObjectByID(id)) TheGameLogic->destroyObject(o);
        w.flush_destroy_list();
        g_seen.clear();
        // Drain the path queue and the partition of this scenario's objects.
        TheAI->pathfinder()->processPathfindQueue();
        TheAI->pathfinder()->processPathfindQueue();
        ThePartitionManager->PartitionManager::update();
        if (TheGameLogic->getFirstObject()) die("objects left over");
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    std::printf("frozen frames %d\n", frozen_frames);
    for (int i = 0; i <= 10; ++i) std::printf("op %d: %d\n", i, g_op_counts[i]);
    for (int i = 1; i <= 12; ++i) std::printf("action %d: %d\n", i, g_action_counts[i]);
    return 0;
}
