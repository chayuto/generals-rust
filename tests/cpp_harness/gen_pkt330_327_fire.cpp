// PKT-330 FlammableUpdate and PKT-327 FireSpreadUpdate.
//
// Canonical 3-mode build & run (all three outputs are byte-identical):
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt330_327_fire.cpp --opt O0 -o <dir>/fire_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt330_327_fire.cpp --opt O2 -o <dir>/fire_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt330_327_fire.cpp --opt O2 --sanitize -o <dir>/fire_san
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/fire_o2 docs/golden_vectors/PKT330_327_Fire.json
//
// Each vector spawns up to eight objects from the fixture templates and runs a script of damage,
// direct ignition, spread starts, moves, deaths, status changes and frames. A frame step is the
// engine's sleepy-update loop (restated here), the partition update and the destroy-list flush.
// After every operation a row records one FNV-1a digest per section: modules (save bytes of both
// modules, the unsaved-in-version-1 flame source, wouldIgnite), objects, OCL calls and the logic
// PRNG. Vector 0 also stores the section text. Set FIRE_DUMP=<index> to print the section text
// of any other vector to stderr. The fixture INI is stored in the JSON and loaded by the test.
//
// The reference saves FlammableUpdate version 1 (`RETAIL_COMPATIBLE_XFER_SAVE=1`).
//
// Left out, and why:
//   - objects without a body: `tryToIgnite` calls through a null body module;
//   - frames from about 0x3fffffff up: a module sleeping forever is then always due and the
//     sleepy loop never ends. Frames here stay below 0x20100000;
//   - a spread delay pair with min = INT_MIN and max = INT_MAX: `% 0` in the PRNG helper;
//   - a NaN or infinite SpreadTryRange: the partition search converts it to an integer cell
//     radius, which is undefined;
//   - durations above 4294967295 ms: the reference's Apple INI branch saturates them and other
//     platforms reject them. So no delay passes 0x7fffffff frames, and the signed reading of
//     `UpdateSleepTime` and the wrap of `now + delay` are never reached;
//   - NaN FlameDamageLimit in INI (not a token retail data can hold), NaN healing;
//   - not varied, because neither module reads them: relationships, capture, damage types other
//     than flame, particle beam, explosion and unresistable, disabled types other than HELD,
//     contained objects, loading a save (the Rust test reloads the recorded bytes);
//   - embers: `OCLEmbers` lists hold a recording nugget only, so the call, its creator and its
//     position are checked but nothing is created (the port records OCL calls and does not run
//     them). Burning sounds are client audio and are not observed;
//   - `ActiveBody::setAflame` only rebuilds client particle systems and is not observed.

#include "oracle_wp51_world.h"

#include "Common/GameAudio.h"
#include "Common/RandomValue.h"
#include "Common/Team.h"
#include "Common/ThingTemplate.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/FireSpreadUpdate.h"
#include "GameLogic/Module/FlammableUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/PartitionManager.h"

#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <limits>
#include <sstream>
#include <string>
#include <vector>

struct FireSleepyTag { using type = std::vector<UpdateModulePtr> GameLogic::*; };
template struct MemberAccess<FireSleepyTag, &GameLogic::m_sleepyUpdates>;
std::vector<UpdateModulePtr> GameLogic::* access_member(FireSleepyTag);
struct FireCurTag { using type = UpdateModulePtr GameLogic::*; };
template struct MemberAccess<FireCurTag, &GameLogic::m_curUpdateModule>;
UpdateModulePtr GameLogic::* access_member(FireCurTag);
struct FireSourceTag { using type = ObjectID FlammableUpdate::*; };
template struct MemberAccess<FireSourceTag, &FlammableUpdate::m_flameSource>;
ObjectID FlammableUpdate::* access_member(FireSourceTag);

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }
static Real real_of(uint32_t b) { Real v; std::memcpy(&v, &b, 4); return v; }

static uint32_t g_state = 0x5F1E3A27u;
static uint32_t next_u32() {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state;
}
static uint32_t pick(uint32_t n) { return next_u32() % n; }

template <size_t N>
static const char* choose(const char* const (&pool)[N]) { return pool[pick(N)]; }

static void die(const char* what) {
    std::fprintf(stderr, "gen_pkt330_327_fire: %s\n", what);
    std::exit(1);
}

// ---------------------------------------------------------------------------------------------
// OCL recorder
// ---------------------------------------------------------------------------------------------
static std::ostringstream g_ocl;
static uint32_t g_ocl_calls = 0;

class RecordingNugget : public ObjectCreationNugget {
public:
    int list = 0;
    Object* create(const Object* primaryObj, const Coord3D*, const Coord3D*, Real, UnsignedInt) const override {
        die("unexpected positional OCL call");
        return nullptr;
    }
    Object* create(const Object* primary, const Object* secondary, UnsignedInt lifetime) const override {
        const Coord3D* p = primary->getPosition();
        g_ocl << list << ' ' << static_cast<uint32_t>(primary->getID()) << ' ' << bits(p->x) << ' '
              << bits(p->y) << ' ' << bits(p->z) << ';';
        ++g_ocl_calls;
        if (secondary || lifetime) die("unexpected OCL arguments");
        return nullptr;
    }
protected:
    MemoryPool* getObjectMemoryPool() override { return nullptr; }
};
static RecordingNugget g_nugget[2];
static const char* const kOclNames[2] = {"P330_Embers", "P330_Embers2"};

// ---------------------------------------------------------------------------------------------
// Fixture
// ---------------------------------------------------------------------------------------------
static const char* kArmorIni =
    "Armor P330_Half\n  Armor = Default 100%\n  Armor = FLAME 50%\n  Armor = PARTICLE_BEAM 50%\nEnd\n"
    "Armor P330_Double\n  Armor = Default 100%\n  Armor = FLAME 200%\n  Armor = PARTICLE_BEAM 300%\nEnd\n"
    "Armor P330_Immune\n  Armor = Default 100%\n  Armor = FLAME 0%\nEnd\n"
    "Armor P330_Odd\n  Armor = Default 100%\n  Armor = FLAME 37%\n  Armor = PARTICLE_BEAM -100%\nEnd\n";

static const char* const kBurned[] = {nullptr, "0", "1", "100", "500", "1000", "2000", "5000", "4294967295"};
static const char* const kAflame[] = {nullptr, "0", "33", "100", "500", "1000", "1000", "3000", "3000", "10000", "4294967295"};
static const char* const kDelay[] = {nullptr, "0", "1", "100", "500", "500", "1000", "4294967295"};
static const char* const kAmount[] = {nullptr, "0", "1", "5", "5", "30", "1000", "-5"};
static const char* const kLimit[] = {nullptr, nullptr, "20.0", "0.0", "-0.0", "-5", "0.5", "1.0", "37.3", "100", "1000000000"};
static const char* const kExpire[] = {nullptr, nullptr, "0", "1", "100", "500", "2000", "4294967295"};
static const char* const kMinDelay[] = {nullptr, "0", "1", "33", "100", "100", "500", "1000", "4294967295"};
static const char* const kMaxDelay[] = {nullptr, "0", "33", "100", "500", "500", "1000", "2000", "4294967295"};
static const char* const kRange[] = {nullptr, "0", "-0.0", "-1", "-50", "-100", "0.5", "5", "10", "10",
                                     "25", "25", "40", "100", "1000000000",
                                     "1000000000000000000000000000000"};
static const char* const kOcl[] = {nullptr, "None", "P330_Embers", "P330_Embers", "P330_Embers2"};
static const char* const kHealth[] = {"5.0", "30.0", "100.0", "100.0", "250.0", "1000.0"};
static const char* const kBody[] = {"ActiveBody", "ActiveBody", "ActiveBody", "ActiveBody", "ActiveBody",
                                    "ImmortalBody", "HighlanderBody", "StructureBody", "InactiveBody"};
static const char* const kArmor[] = {nullptr, nullptr, nullptr, "P330_Half", "P330_Double", "P330_Immune", "P330_Odd"};

static void field(std::ostringstream& ss, const char* name, const char* v) {
    if (v) ss << "    " << name << " = " << v << "\n";
}

static void flammable(std::ostringstream& ss, int tag, const char* burned, const char* aflame,
                      const char* delay, const char* amount, const char* limit, const char* expire,
                      const char* sound) {
    ss << "  Behavior = FlammableUpdate ModuleTag_Flame" << tag << "\n";
    field(ss, "BurnedDelay", burned);
    field(ss, "AflameDuration", aflame);
    field(ss, "AflameDamageDelay", delay);
    field(ss, "AflameDamageAmount", amount);
    field(ss, "FlameDamageLimit", limit);
    field(ss, "FlameDamageExpiration", expire);
    field(ss, "BurningSoundName", sound);
    ss << "  End\n";
}

static void random_flammable(std::ostringstream& ss, int tag) {
    const char* b = choose(kBurned);
    const char* a = choose(kAflame);
    const char* d = choose(kDelay);
    const char* m = choose(kAmount);
    const char* l = choose(kLimit);
    const char* e = choose(kExpire);
    flammable(ss, tag, b, a, d, m, l, e, pick(4) == 0 ? "P330_Burning" : nullptr);
}

static void spread(std::ostringstream& ss, int tag, const char* lo, const char* hi, const char* range,
                   const char* ocl) {
    ss << "  Behavior = FireSpreadUpdate ModuleTag_Spread" << tag << "\n";
    field(ss, "MinSpreadDelay", lo);
    field(ss, "MaxSpreadDelay", hi);
    field(ss, "SpreadTryRange", range);
    field(ss, "OCLEmbers", ocl);
    ss << "  End\n";
}

static void random_spread(std::ostringstream& ss, int tag) {
    const char* lo = choose(kMinDelay);
    const char* hi = choose(kMaxDelay);
    const char* r = choose(kRange);
    const char* o = choose(kOcl);
    spread(ss, tag, lo, hi, r, o);
}

static void head(std::ostringstream& ss, int i, const char* body, const char* health, const char* armor) {
    ss << "Object P330_" << i << "\n  KindOf = PRELOAD\n";
    if (armor) ss << "  ArmorSet\n    Conditions = None\n    Armor = " << armor << "\n    DamageFX = None\n  End\n";
    ss << "  Body = " << body << " ModuleTag_Body\n";
    if (std::strcmp(body, "InactiveBody") != 0)
        ss << "    MaxHealth = " << health << "\n    InitialHealth = " << health << "\n";
    ss << "  End\n";
}

static const int kFixed = 6;
static const int kTemplates = 160;

static std::string fixture_ini() {
    std::ostringstream ss;
    const std::string destroy_die = "  Behavior = DestroyDie ModuleTag_Die\n  End\n";
    // 0: a spreader that burns down; 1: a plain flammable; 2: dies of its own fire and is removed;
    // 3: spread only; 4: no fire module; 5: spread listed before two flammables.
    head(ss, 0, "ActiveBody", "500.0", nullptr);
    flammable(ss, 0, "500", "1000", "100", "5", "20.0", "100", "P330_Burning");
    spread(ss, 0, "100", "200", "25", "P330_Embers");
    ss << "End\n";
    head(ss, 1, "ActiveBody", "100.0", nullptr);
    flammable(ss, 0, nullptr, "500", "100", "5", nullptr, nullptr, nullptr);
    ss << "End\n";
    head(ss, 2, "ActiveBody", "12.0", nullptr);
    ss << destroy_die;
    flammable(ss, 0, "100", "1000", "100", "30", "1.0", "100", nullptr);
    spread(ss, 0, "33", "33", "40", "P330_Embers2");
    ss << "End\n";
    head(ss, 3, "ActiveBody", "100.0", nullptr);
    spread(ss, 0, "100", "100", "100", "P330_Embers");
    ss << "End\n";
    head(ss, 4, "ActiveBody", "100.0", nullptr);
    ss << "End\n";
    head(ss, 5, "ActiveBody", "300.0", "P330_Half");
    spread(ss, 0, "100", "500", "40", "None");
    flammable(ss, 0, "1000", "500", "33", "1", "10", "500", nullptr);
    flammable(ss, 1, "100", "2000", "500", "2", "30", nullptr, nullptr);
    ss << "End\n";
    for (int i = kFixed; i < kTemplates; ++i) {
        const char* b = choose(kBody);
        const char* h = choose(kHealth);
        const char* a = choose(kArmor);
        head(ss, i, b, h, a);
        if (pick(5) == 0) ss << destroy_die;
        const uint32_t layout = pick(20);
        if (layout < 5) {
            random_flammable(ss, 0);
        } else if (layout < 11) {
            random_flammable(ss, 0);
            random_spread(ss, 0);
        } else if (layout < 15) {
            random_spread(ss, 0);
            random_flammable(ss, 0);
        } else if (layout < 17) {
            random_spread(ss, 0);
        } else if (layout < 19) {
            random_flammable(ss, 0);
            random_spread(ss, 0);
            random_flammable(ss, 1);
            if (layout == 18) random_spread(ss, 1);
        }
        ss << "End\n";
    }
    return ss.str();
}

// ---------------------------------------------------------------------------------------------
// Script
// ---------------------------------------------------------------------------------------------
enum {
    OP_DAMAGE, OP_HEAL, OP_STEP, OP_IGNITE, OP_SPREAD, OP_MOVE, OP_DESTROY, OP_KILL, OP_STATUS,
    OP_DISABLE, OP_JUMP, OP_COUNT
};
static const DamageType kDamage[] = {DAMAGE_FLAME, DAMAGE_PARTICLE_BEAM, DAMAGE_EXPLOSION, DAMAGE_UNRESISTABLE};

struct Op { uint32_t code, a, b, c, d; };
struct Spawn { uint32_t type, team, x, y, z; };
struct Vec {
    uint32_t frame = 0, seed = 0;
    std::vector<Spawn> spawns;
    std::vector<Op> ops;
    std::vector<std::vector<std::string>> rows;
    uint32_t crc = 0, rng = 0;
};

struct Scene {
    oracle_wp51::World* w = nullptr;
    std::vector<Object*> obj;  // null once flushed
    std::vector<ObjectID> id;
};

static NameKeyType key_flammable() { return TheNameKeyGenerator->nameToKey("FlammableUpdate"); }
static NameKeyType key_spread() { return TheNameKeyGenerator->nameToKey("FireSpreadUpdate"); }
static FlammableUpdate* flammable_of(Object* o) {
    return static_cast<FlammableUpdate*>(o->findUpdateModule(key_flammable()));
}
static FireSpreadUpdate* spread_of(Object* o) {
    return static_cast<FireSpreadUpdate*>(o->findUpdateModule(key_spread()));
}

static std::string save_hex(Snapshot* s) {
    oracle::MemoryXfer xs(XFER_SAVE);
    xs.open("save");
    oracle::call_snapshot_xfer(s, &xs);
    xs.close();
    return xs.hex();
}

static uint32_t fnv32(const std::string& s) {
    uint32_t h = 0x811c9dc5u;
    for (unsigned char c : s) {
        h ^= c;
        h *= 0x01000193u;
    }
    return h;
}

// Coverage.
enum { COV_IGNITE_DAMAGE, COV_IGNITE_DIRECT, COV_IGNITE_STEP, COV_OUT_BURNED, COV_OUT_NORMAL,
       COV_REIGNITE, COV_LIMIT_RESET, COV_LIMIT_DRAINED, COV_FIRE_TICK, COV_FIRE_DEATH,
       COV_FIRE_DESTROY, COV_OCL, COV_BURNED_STAYS_NORMAL,
       COV_DUPLICATE_MODULE, COV_COUNT };
static const char* kCov[COV_COUNT] = {"ignited_by_damage", "ignited_directly", "ignited_in_step",
    "ended_burned", "ended_normal", "reignited", "limit_reset", "limit_drained", "fire_tick_damage",
    "died_aflame", "removed_aflame_in_step", "ocl_calls", "burned_bit_while_would_ignite", "second_flammable_ignited"};
static uint32_t g_cov[COV_COUNT];

struct Probe {
    bool alive = false, would = false, aflame = false, burned = false, dead = false;
    Real health = 0, limit = 0;
};
static Probe probe(Object* o) {
    Probe p;
    if (!o) return p;
    p.alive = true;
    p.aflame = o->getStatusBits().test(OBJECT_STATUS_AFLAME);
    p.burned = o->getStatusBits().test(OBJECT_STATUS_BURNED);
    p.dead = o->isEffectivelyDead();
    p.health = o->getBodyModule()->getHealth();
    if (FlammableUpdate* f = flammable_of(o)) {
        p.would = f->wouldIgnite();
        const std::string h = save_hex(f);
        // version, update header (version, behavior, object, module versions, next frame), status,
        // three frames, then the limit.
        const size_t at = h.size() - 16;  // limit and last-damage frame are the last 8 bytes
        uint32_t v = 0;
        for (int i = 0; i < 4; ++i)
            v |= static_cast<uint32_t>(std::stoul(h.substr(at + 2 * i, 2), nullptr, 16)) << (8 * i);
        p.limit = real_of(v);
    }
    return p;
}

static std::vector<std::string> observe(Scene& s) {
    std::ostringstream mod, obj, rng;
    for (Object* o : s.obj) {
        if (!o) { mod << "-;"; obj << "-;"; continue; }
        for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m) {
            if ((*m)->getModuleNameKey() == key_flammable()) {
                FlammableUpdate* f = static_cast<FlammableUpdate*>(*m);
                mod << 'F' << save_hex(f) << ' ' << static_cast<uint32_t>(f->*access_member(FireSourceTag()))
                    << ' ' << (f->wouldIgnite() ? 1 : 0) << ' ';
            } else if ((*m)->getModuleNameKey() == key_spread()) {
                mod << 'S' << save_hex(static_cast<FireSpreadUpdate*>(*m)) << ' ';
            }
        }
        mod << ';';
        const Coord3D* pos = o->getPosition();
        const Drawable* draw = o->getDrawable();
        if (!draw) die("object without a drawable");
        obj << (o->isDestroyed() ? 1 : 0) << ' ' << (o->isEffectivelyDead() ? 1 : 0) << ' '
            << (o->getStatusBits().test(OBJECT_STATUS_AFLAME) ? 1 : 0) << ' '
            << (o->getStatusBits().test(OBJECT_STATUS_BURNED) ? 1 : 0) << ' '
            << (draw->getModelConditionFlags().test(MODELCONDITION_AFLAME) ? 1 : 0) << ' '
            << (draw->getModelConditionFlags().test(MODELCONDITION_SMOLDERING) ? 1 : 0) << ' '
            << bits(o->getBodyModule()->getHealth()) << ' ' << bits(pos->x) << ' ' << bits(pos->y)
            << ' ' << bits(pos->z) << ';';
    }
    rng << GetGameLogicRandomSeedCRC();
    std::string ocl = g_ocl.str();
    g_ocl.str(std::string());
    return {mod.str(), obj.str(), ocl, rng.str()};
}

// `GameLogic::update`'s sleepy loop. `rebalanceSleepyUpdate(0)` only ever moves the root down.
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

static void step_sleepy_updates() {
    auto& heap = TheGameLogic->*access_member(FireSleepyTag());
    auto& cur = TheGameLogic->*access_member(FireCurTag());
    const UnsignedInt now = TheGameLogic->getFrame();
    int guard = 0;
    while (!heap.empty()) {
        UpdateModulePtr u = heap.front();
        if (u->friend_getNextCallFrame() > now) break;
        if (++guard > 4096) die("sleepy update guard exceeded");
        UpdateSleepTime sleepLen = UPDATE_SLEEP_NONE;
        const DisabledMaskType dis = u->friend_getObject()->getDisabledFlags();
        if (u->getDisabledTypesToProcess().testForAll(dis)) {
            cur = u;
            sleepLen = u->update();
            if (sleepLen < 1) sleepLen = UPDATE_SLEEP_NONE;
            cur = nullptr;
        }
        u->friend_setNextCallFrame(now + sleepLen);
        sift_down_root(heap);
    }
}

static void flush(Scene& s) {
    std::vector<size_t> gone;
    for (size_t i = 0; i < s.obj.size(); ++i) {
        if (s.obj[i] && s.obj[i]->isDestroyed()) gone.push_back(i);
    }
    s.w->flush_destroy_list();
    for (size_t i : gone) s.obj[i] = nullptr;
}

static void apply(Scene& s, const Op& op) {
    Object* o = (op.code == OP_STEP || op.code == OP_JUMP) ? nullptr : s.obj.at(op.a);
    switch (op.code) {
        case OP_DAMAGE:
            if (o) {
                DamageInfo d;
                d.in.m_damageType = kDamage[op.b];
                d.in.m_deathType = DEATH_NORMAL;
                d.in.m_amount = real_of(op.c);
                d.in.m_sourceID = op.d ? s.id.at(op.d - 1) : INVALID_ID;
                o->attemptDamage(&d);
            }
            break;
        case OP_HEAL:
            if (o) o->attemptHealing(real_of(op.b), nullptr);
            break;
        case OP_STEP:
            for (uint32_t k = 0; k < op.a; ++k) {
                s.w->set_frame(TheGameLogic->getFrame() + 1);
                step_sleepy_updates();
                ThePartitionManager->update();
                flush(s);
            }
            break;
        case OP_IGNITE:
            if (o && flammable_of(o)) flammable_of(o)->tryToIgnite();
            break;
        case OP_SPREAD:
            if (o && spread_of(o)) spread_of(o)->startFireSpreading();
            break;
        case OP_MOVE:
            if (o) {
                Coord3D pos = {real_of(op.b), real_of(op.c), real_of(op.d)};
                o->setPosition(&pos);
                ThePartitionManager->update();
            }
            break;
        case OP_DESTROY:
            if (o) TheGameLogic->destroyObject(o);
            break;
        case OP_KILL:
            if (o) o->kill();
            break;
        case OP_STATUS:
            if (o) {
                const ObjectStatusTypes st = op.b == 0 ? OBJECT_STATUS_AFLAME : OBJECT_STATUS_BURNED;
                if (op.c) o->setStatus(MAKE_OBJECT_STATUS_MASK(st));
                else o->clearStatus(MAKE_OBJECT_STATUS_MASK(st));
            }
            break;
        case OP_DISABLE:
            if (o) {
                if (op.b) o->setDisabled(DISABLED_HELD);
                else o->clearDisabled(DISABLED_HELD);
            }
            break;
        case OP_JUMP:
            s.w->set_frame(TheGameLogic->getFrame() + op.a);
            break;
    }
}

static void cover(Scene& s, const Op& op) {
    std::vector<Probe> before;
    for (Object* o : s.obj) before.push_back(probe(o));
    const uint32_t ocl = g_ocl_calls;
    apply(s, op);
    g_cov[COV_OCL] += g_ocl_calls - ocl;
    for (size_t i = 0; i < s.obj.size(); ++i) {
        const Probe& a = before[i];
        const Probe b = probe(s.obj[i]);
        if (a.alive && !b.alive && a.aflame && op.code == OP_STEP) ++g_cov[COV_FIRE_DESTROY];
        if (!a.alive || !b.alive) continue;
        if (!a.aflame && b.aflame && op.code != OP_STATUS) {
            if (op.code == OP_DAMAGE) ++g_cov[COV_IGNITE_DAMAGE];
            else if (op.code == OP_IGNITE) ++g_cov[COV_IGNITE_DIRECT];
            else if (op.code == OP_STEP) ++g_cov[COV_IGNITE_STEP];
            if (a.burned) ++g_cov[COV_REIGNITE];
        }
        if (op.code == OP_STEP && a.aflame && !b.aflame) {
            if (b.burned) ++g_cov[COV_OUT_BURNED];
            else ++g_cov[COV_OUT_NORMAL];
        }
        if (op.code == OP_DAMAGE && op.a == i) {
            if (b.limit > a.limit) ++g_cov[COV_LIMIT_RESET];
            if (b.limit < a.limit) ++g_cov[COV_LIMIT_DRAINED];
        }
        if (op.code == OP_STEP && a.aflame && b.health < a.health) ++g_cov[COV_FIRE_TICK];
        if (op.code == OP_STEP && a.aflame && !a.dead && b.dead) ++g_cov[COV_FIRE_DEATH];
        if (b.burned && b.would) ++g_cov[COV_BURNED_STAYS_NORMAL];
    }
}

static const uint32_t kInf = 0x7f800000u;
static const uint32_t kNan = 0x7fc00000u;

static uint32_t random_amount() {
    static const Real pool[] = {0.0f, -0.0f, -10.0f, 0.25f, 1.0f, 5.0f, 19.999998f, 20.0f, 25.0f,
                                40.0f, 100.0f, 1000.0f, 1.0e9f};
    const uint32_t r = pick(40);
    if (r == 0) return kInf;
    if (r == 1) return kNan;
    if (r < 20) return bits(pool[pick(13)]);
    return bits(static_cast<Real>(pick(4000)) / 97.0f);
}

static uint32_t random_xy() {
    static const Real edge[] = {-30.0f, -1.0f, 0.0f, 19.999998f, 20.0f, 999.0f, 1000.0f, 1030.0f};
    const uint32_t r = pick(24);
    if (r == 0) return bits(edge[pick(8)]);
    if (r < 5) return bits(100.0f + static_cast<Real>(pick(6000)) / 101.0f);
    return bits(100.0f + 5.0f * static_cast<Real>(pick(13)));
}

static uint32_t random_z() {
    static const Real pool[] = {0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 5.0f, 10.0f, 30.0f, -10.0f, 200.0f};
    return bits(pool[pick(10)]);
}

static Op random_op(uint32_t n) {
    static const uint32_t weight[OP_COUNT] = {34, 3, 30, 9, 4, 6, 2, 2, 5, 3, 2};
    uint32_t total = 0;
    for (uint32_t w : weight) total += w;
    uint32_t r = pick(total), code = 0;
    while (r >= weight[code]) r -= weight[code++];
    Op op{code, 0, 0, 0, 0};
    if (code != OP_STEP && code != OP_JUMP) op.a = pick(n);
    switch (code) {
        case OP_DAMAGE:
            op.b = pick(3) ? pick(2) : pick(4);
            op.c = random_amount();
            op.d = pick(3) == 0 ? 0u : 1u + pick(n);
            break;
        case OP_HEAL: op.b = bits(static_cast<Real>(1u + pick(100))); break;
        case OP_STEP: {
            const uint32_t k = pick(10);
            op.a = k < 4 ? 1u + pick(3) : (k < 8 ? 3u + pick(20) : 20u + pick(80));
            break;
        }
        case OP_MOVE: op.b = random_xy(); op.c = random_xy(); op.d = random_z(); break;
        case OP_STATUS: op.b = pick(2); op.c = pick(2); break;
        case OP_DISABLE: op.b = pick(2); break;
        case OP_JUMP: op.a = 1u + (pick(4) == 0 ? pick(200000) : pick(200)); break;
        default: break;
    }
    return op;
}

// Vector 0: drain the limit, let it expire, ignite, spread along a row, burn down, kill by fire.
static void scripted(Vec& v) {
    v.frame = 10;
    v.seed = 1234;
    const auto at = [](uint32_t type, uint32_t team, float x, float y, float z) {
        return Spawn{type, team, bits(x), bits(y), bits(z)};
    };
    v.spawns = {at(0, 1, 100, 100, 0), at(1, 1, 120, 100, 0), at(2, 2, 140, 100, 0),
                at(3, 1, 100, 140, 0), at(4, 1, 105, 100, 0), at(5, 2, 160, 100, 0)};
    const auto flame = [](uint32_t i, float amount, uint32_t source) {
        return Op{OP_DAMAGE, i, 0, bits(amount), source};
    };
    v.ops = {
        flame(0, 8.0f, 2), {OP_DAMAGE, 0, 2, bits(8.0f), 2},  // explosion does not count
        {OP_STEP, 2, 0, 0, 0}, flame(0, 8.0f, 3),
        {OP_STEP, 4, 0, 0, 0}, flame(0, 8.0f, 4),             // expired: the limit starts over
        {OP_DAMAGE, 0, 1, bits(8.0f), 0}, flame(0, 8.0f, 5),  // ignites
        flame(0, 1.0f, 1),                                    // already aflame
        {OP_STEP, 3, 0, 0, 0}, {OP_STEP, 3, 0, 0, 0}, {OP_STEP, 6, 0, 0, 0},
        {OP_STEP, 5, 0, 0, 0}, {OP_STEP, 12, 0, 0, 0}, {OP_STEP, 20, 0, 0, 0},
        {OP_IGNITE, 0, 0, 0, 0},                              // burned: never again
        flame(1, 500.0f, 1), {OP_STEP, 30, 0, 0, 0},
        {OP_STATUS, 3, 0, 1, 0}, {OP_SPREAD, 3, 0, 0, 0}, {OP_STEP, 4, 0, 0, 0},
        {OP_MOVE, 5, bits(100.0f), bits(150.0f), bits(0.0f)}, {OP_STEP, 8, 0, 0, 0},
        {OP_STATUS, 3, 0, 0, 0}, {OP_STEP, 8, 0, 0, 0},
        {OP_IGNITE, 5, 0, 0, 0}, {OP_DISABLE, 5, 1, 0, 0}, {OP_STEP, 20, 0, 0, 0},
        {OP_DISABLE, 5, 0, 0, 0}, {OP_STEP, 40, 0, 0, 0}, {OP_STEP, 60, 0, 0, 0},
        {OP_IGNITE, 2, 0, 0, 0}, {OP_STEP, 5, 0, 0, 0},        // its own fire removes it
        {OP_HEAL, 0, bits(10.0f), 0, 0}, {OP_KILL, 1, 0, 0, 0}, {OP_IGNITE, 1, 0, 0, 0},
        {OP_STEP, 20, 0, 0, 0}, {OP_DESTROY, 5, 0, 0, 0}, {OP_STEP, 2, 0, 0, 0},
    };
}

static void generate(Vec& v, int i) {
    if (i == 0) { scripted(v); return; }
    static const uint32_t starts[] = {0, 1, 2, 30, 59, 60, 61, 100, 100000, 0x20000000u};
    v.frame = pick(3) == 0 ? starts[pick(10)] : 1u + pick(5000);
    v.seed = next_u32() & 0x7fffffff;
    const uint32_t n = 1u + (pick(4) == 0 ? pick(3) : 2u + pick(6));
    // Half the vectors repeat one or two templates, so that chains of spreaders occur.
    const bool uniform = pick(2) == 0;
    const uint32_t t0 = pick(kTemplates), t1 = pick(kTemplates);
    for (uint32_t k = 0; k < n; ++k) {
        uint32_t type = uniform ? (pick(2) ? t0 : t1) : pick(kTemplates);
        if (pick(10) == 0) type = pick(kFixed);
        v.spawns.push_back({type, 1u + pick(2), random_xy(), random_xy(), random_z()});
    }
    const uint32_t count = 12u + pick(30);
    if (pick(2)) v.ops.push_back({OP_IGNITE, pick(n), 0, 0, 0});
    for (uint32_t k = 0; k < count; ++k) v.ops.push_back(random_op(n));
}

static void run(oracle_wp51::World& w, Vec& v) {
    Scene s;
    s.w = &w;
    for (const Spawn& sp : v.spawns) {
        const std::string name = "P330_" + std::to_string(sp.type);
        Object* o = w.spawn_object(name.c_str(), sp.team == 1 ? w.team_a : w.team_b);
        if (!o || !o->getBodyModule()) die("spawn failed");
        Coord3D pos = {real_of(sp.x), real_of(sp.y), real_of(sp.z)};
        o->setPosition(&pos);
        s.obj.push_back(o);
        s.id.push_back(o->getID());
    }
    ThePartitionManager->update();
    InitRandom(v.seed);
    w.set_frame(v.frame);
    g_ocl.str(std::string());
    v.rows.push_back(observe(s));
    for (const Op& op : v.ops) {
        cover(s, op);
        v.rows.push_back(observe(s));
    }
    v.rng = GetGameLogicRandomSeedCRC();
    uint32_t crc = 0;
    for (Object* o : s.obj) {
        if (!o) continue;
        for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m) {
            const NameKeyType k = (*m)->getModuleNameKey();
            if (k == key_flammable())
                crc = crc * 31u + oracle_wp51::World::snapshot_crc(static_cast<FlammableUpdate*>(*m));
            else if (k == key_spread())
                crc = crc * 31u + oracle_wp51::World::snapshot_crc(static_cast<FireSpreadUpdate*>(*m));
        }
    }
    v.crc = crc;
    // Coverage: a second FlammableUpdate is reachable only through its own damage callback.
    for (Object* o : s.obj) {
        if (!o) continue;
        int seen = 0;
        for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m) {
            if ((*m)->getModuleNameKey() != key_flammable()) continue;
            if (seen++ && !static_cast<FlammableUpdate*>(*m)->wouldIgnite())
                ++g_cov[COV_DUPLICATE_MODULE];
        }
    }
    for (Object* o : s.obj) {
        if (o && !o->isDestroyed()) TheGameLogic->destroyObject(o);
    }
    w.flush_destroy_list();
    ThePartitionManager->update();
    if (!(TheGameLogic->*access_member(FireSleepyTag())).empty()) die("updates left over");
}

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_pkt330_327_fire <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    const int dump = std::getenv("FIRE_DUMP") ? std::atoi(std::getenv("FIRE_DUMP")) : -1;
    oracle_wp51::World w;
    w.load_ini(kArmorIni);
    w.load_ini("ObjectCreationList P330_Embers\nEnd\nObjectCreationList P330_Embers2\nEnd\n");
    w.init_with_neutral_player(nullptr, nullptr);
    for (int i = 0; i < 2; ++i) {
        const ObjectCreationList* list = TheObjectCreationListStore->findObjectCreationList(kOclNames[i]);
        if (!list) die("missing OCL");
        g_nugget[i].list = i;
        const_cast<ObjectCreationList*>(list)->addObjectCreationNugget(&g_nugget[i]);
    }
    const std::string ini = fixture_ini();
    w.load_ini(ini);
    TheWritableGlobalData->m_unitDamagedThresh = 0.7f;
    TheWritableGlobalData->m_unitReallyDamagedThresh = 0.35f;

    const int kTotal = 1000;
    std::vector<Vec> vecs;
    for (int i = 0; i < kTotal; ++i) {
        Vec v;
        generate(v, i);
        if (std::getenv("FIRE_TRACE")) std::fprintf(stderr, "vector %d\n", i);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(v.frame);
        InitRandom(v.seed);
        run(w, v);
        if (i == dump) {
            for (size_t r = 0; r < v.rows.size(); ++r)
                for (const std::string& x : v.rows[r]) std::fprintf(stderr, "row %zu: %s\n", r, x.c_str());
        }
        vecs.push_back(std::move(v));
    }

    std::ostringstream f;
    const std::string root = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/";
    f << "{\n\"packet\": \"PKT-330+PKT-327\",\n\"vector_count\": " << kTotal << ",\n";
    f << "\"team_ids\": [" << w.team_a->getID() << ',' << w.team_b->getID() << "],\n";
    f << "\"source_sha256\": {\"flammable\":\"" << oracle::sha256_file((root + "FlammableUpdate.cpp").c_str())
      << "\",\"spread\":\"" << oracle::sha256_file((root + "FireSpreadUpdate.cpp").c_str()) << "\"},\n";
    f << "\"armor_ini\": \"" << oracle::json_escape(kArmorIni) << "\",\n";
    f << "\"object_ini\": \"" << oracle::json_escape(ini) << "\",\n";
    f << "\"seed0_diagnostics\": [\n";
    for (size_t r = 0; r < vecs[0].rows.size(); ++r) {
        f << "{\"row\":" << r;
        const char* names[4] = {"modules", "objects", "ocl", "rng"};
        for (int k = 0; k < 4; ++k) f << ",\"" << names[k] << "\":\"" << vecs[0].rows[r][k] << '"';
        f << (r + 1 < vecs[0].rows.size() ? "},\n" : "}\n");
    }
    f << "],\n\"vectors\": [\n";
    char hex[16];
    for (int i = 0; i < kTotal; ++i) {
        const Vec& v = vecs[i];
        f << "{\"id\":" << i << ",\"frame\":" << v.frame << ",\"seed\":" << v.seed << ",\"spawns\":[";
        for (size_t k = 0; k < v.spawns.size(); ++k) {
            const Spawn& s = v.spawns[k];
            f << (k ? "," : "") << s.type << ',' << s.team << ',' << s.x << ',' << s.y << ',' << s.z;
        }
        f << "],\"ops\":[";
        for (size_t k = 0; k < v.ops.size(); ++k) {
            const Op& o = v.ops[k];
            f << (k ? "," : "") << o.code << ',' << o.a << ',' << o.b << ',' << o.c << ',' << o.d;
        }
        f << "],\"rows\":\"";
        for (size_t r = 0; r < v.rows.size(); ++r) {
            for (int k = 0; k < 4; ++k) {
                std::snprintf(hex, sizeof(hex), "%08x", fnv32(v.rows[r][k]));
                f << hex;
            }
        }
        f << "\",\"crc\":" << v.crc << ",\"rng\":" << v.rng << (i + 1 < kTotal ? "},\n" : "}\n");
    }
    f << "]\n}\n";
    oracle::write_file_atomic(out_path.c_str(), f.str());

    std::fprintf(stderr, "wrote %d vectors\n", kTotal);
    for (int c = 0; c < COV_COUNT; ++c) std::fprintf(stderr, "%s=%u ", kCov[c], g_cov[c]);
    std::fprintf(stderr, "\n");
    return 0;
}
