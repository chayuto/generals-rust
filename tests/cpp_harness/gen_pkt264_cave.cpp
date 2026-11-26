// PKT-264 CaveContain, `CaveSystem` and the parts of `TunnelTracker` a cave reaches.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt264_cave.cpp --opt O0 -o <dir>/pkt264_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt264_cave.cpp --opt O2 -o <dir>/pkt264_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt264_cave.cpp --opt O2 --sanitize -o <dir>/pkt264_san
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/pkt264_o2 docs/golden_vectors/PKT264_Cave.json
//
// Each vector is a script of operations on up to five caves and their passengers. After every
// operation a row records one FNV-1a digest per section: cave system, cave modules, objects,
// contain queries, and misc (PRNG, AI orders). Vector 0 also stores the section text.
// Set PKT264_DUMP=<index> to print the section text of any other vector to stderr.
//
// Left out, each because the reference cannot be run there:
//   - any contain operation, query, death, damage or index switch of a cave whose index has no
//     tracker (no cave of it was ever built): the null tracker is dereferenced. A cave under
//     construction is still killed in that state, as `onDie` returns first;
//   - negative cave indices: `registerNewCave` appends slots until memory runs out;
//   - flushing the destroy list while a passenger list holds a destroyed object, or while an
//     object's container is destroyed (freed memory is read; only the flush is skipped);
//   - saving a `CaveSystem` with an unused index below a used one (it throws; recorded as such);
//   - damage that names a source when the source or the target is a cave left without a team:
//     `ActiveBody::attemptDamage` dereferences the null player. The hit is sent without a source.
// Also left out: `orderAllPassengersToExit` with `instantly`, where the unported AI leaves the
// container itself; and, after an order or a sale, the AI's "wants to exit" entry is withdrawn.

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/RandomValue.h"
#include "Common/Team.h"
#include "Common/ThingTemplate.h"
#include "Common/TunnelTracker.h"
#include "GameLogic/CaveSystem.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/CaveContain.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/CreateModule.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

struct Pkt264IndexTag { using type = Int CaveContain::*; };
template struct MemberAccess<Pkt264IndexTag, &CaveContain::m_caveIndex>;
Int CaveContain::* access_member(Pkt264IndexTag);
struct Pkt264VectorTag { using type = std::vector<TunnelTracker*> CaveSystem::*; };
template struct MemberAccess<Pkt264VectorTag, &CaveSystem::m_tunnelTrackerVector>;
std::vector<TunnelTracker*> CaveSystem::* access_member(Pkt264VectorTag);
struct Pkt264DetectTag { using type = UnsignedInt StealthUpdate::*; };
template struct MemberAccess<Pkt264DetectTag, &StealthUpdate::m_detectionExpiresFrame>;
UnsignedInt StealthUpdate::* access_member(Pkt264DetectTag);

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }

// A second team of team A's player: moving to it is a capture that keeps the owner.
static Team* g_team_a2 = nullptr;

static uint32_t g_state = 0x3A91C5E7u;
static uint32_t next_u32() {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state;
}
static uint32_t pick(uint32_t n) { return next_u32() % n; }

enum {
    OP_ENTER, OP_EXIT, OP_KILL_CAVE, OP_DESTROY_CAVE, OP_SELL, OP_CAPTURE_CAVE, OP_BUILD,
    OP_SWITCH, OP_DAMAGE, OP_DESTROY_UNIT, OP_KILL_UNIT, OP_REMOVE_ALL, OP_KILL_ALL, OP_HARM,
    OP_UPDATE, OP_ADVANCE, OP_FLUSH, OP_STATUS, OP_CAVE_STATUS, OP_RECALC, OP_ORDER_EXIT,
    OP_ORDER_IDLE, OP_CAPTURE_UNIT, OP_COUNT
};

// Cave template: its index and whether it dies only to suicide or is destroyed on death.
static const int kCaveTypes = 8;
static const int kCaveIndex[kCaveTypes] = {0, 0, 0, 1, 1, 2, 4, 0};
enum { U_INF, U_HERO, U_VEH, U_AIR, U_STEALTH, U_STEALTH_HERO, U_DD, U_AI, U_TRANS, U_COUNT };
static const char* kUnit[U_COUNT] = {"P264_Inf", "P264_Hero", "P264_Veh", "P264_Air",
                                     "P264_Stealth", "P264_StealthHero", "P264_DestroyDie",
                                     "P264_Ai", "P264_Trans"};

struct Op { uint32_t code, a, b, c; };
struct Vec {
    uint32_t cap = 0, frame = 0, seed = 0;
    std::vector<uint32_t> p;
    std::vector<Op> ops;
    std::vector<std::vector<std::string>> rows;
    std::string final_system, final_modules;
    uint32_t crc = 0, rng = 0;
};

static std::string body(const char* health) {
    return std::string("  Body = ActiveBody ModuleTag_Body\n    MaxHealth = ") + health +
           "\n    InitialHealth = " + health + "\n  End\n";
}

static std::string fixture_ini() {
    std::ostringstream ss;
    ss << "PlayerTemplate FactionP264\n  Side = America\n  PlayableSide = Yes\nEnd\n";
    const std::string destroy_die = "  Behavior = DestroyDie ModuleTag_Die\n  End\n";
    for (int i = 0; i < kCaveTypes; ++i) {
        std::string contain = std::string("  Behavior = CaveContain ModuleTag_Contain\n") +
                              "    CaveIndex = " + std::to_string(kCaveIndex[i]) + "\n";
        if (i == 2) contain += "    DeathTypes = NONE +SUICIDED\n";
        contain += "  End\n";
        ss << "Object P264_Cave" << i << "\n  KindOf = PRELOAD STRUCTURE\n" << body("500.0");
        if (i == 5) ss << destroy_die;
        ss << contain;
        if (i == 4 || i == 7) ss << destroy_die;
        ss << "End\n";
    }
    const std::string stealth = "  Behavior = StealthUpdate ModuleTag_Stealth\n  End\n";
    ss << "Object P264_Inf\n  KindOf = PRELOAD INFANTRY\n  TransportSlotCount = 1\n" << body("100.0") << "End\n";
    ss << "Object P264_Hero\n  KindOf = PRELOAD INFANTRY HERO\n" << body("200.0") << "End\n";
    ss << "Object P264_Veh\n  KindOf = PRELOAD VEHICLE\n" << body("300.0") << "End\n";
    ss << "Object P264_Air\n  KindOf = PRELOAD VEHICLE AIRCRAFT\n" << body("150.0") << "End\n";
    ss << "Object P264_Stealth\n  KindOf = PRELOAD INFANTRY STEALTH_GARRISON\n" << body("90.0") << stealth << "End\n";
    ss << "Object P264_StealthHero\n  KindOf = PRELOAD INFANTRY HERO STEALTH_GARRISON\n" << body("95.0") << stealth << "End\n";
    ss << "Object P264_DestroyDie\n  KindOf = PRELOAD INFANTRY\n" << body("60.0") << destroy_die << "End\n";
    ss << "Object P264_Ai\n  KindOf = PRELOAD INFANTRY\n" << body("110.0")
       << "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\nEnd\n";
    ss << "Object P264_Trans\n  KindOf = PRELOAD VEHICLE\n" << body("250.0")
       << "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 8\n"
       << "    AllowInsideKindOf = INFANTRY\n  End\nEnd\n";
    return ss.str();
}

struct Scene {
    oracle_wp51::World* w = nullptr;
    uint32_t caves = 0, units = 0;
    std::vector<Object*> obj;  // caves, then units, then transport riders; null once flushed
    std::vector<ObjectID> id;
    uint32_t orders = 0;

    Team* team(uint32_t i) const {
        if (i == 3) return g_team_a2;
        return i == 0 ? w->team_neutral : (i == 1 ? w->team_a : w->team_b);
    }
    // 4 is the null team.
    uint32_t team_index(const Object* o) const {
        for (uint32_t i = 0; i < 4; ++i) {
            if (o->getTeam() == team(i)) return i;
        }
        return 4u;
    }
};

static CaveContain* module_of(const Object* o) { return static_cast<CaveContain*>(o->getContain()); }
static Int index_of(const Object* o) { return module_of(o)->*access_member(Pkt264IndexTag()); }

static std::vector<TunnelTracker*>& trackers() {
    return TheCaveSystem->*access_member(Pkt264VectorTag());
}
static TunnelTracker* tracker_at(Int i) {
    return i >= 0 && static_cast<size_t>(i) < trackers().size() ? trackers()[i] : nullptr;
}
static bool has_tracker(const Object* o) { return o && tracker_at(index_of(o)) != nullptr; }
static bool under_construction(const Object* o) {
    return o->getStatusBits().test(OBJECT_STATUS_UNDER_CONSTRUCTION);
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

static bool system_has_gap() {
    return std::find(trackers().begin(), trackers().end(), nullptr) != trackers().end();
}

static std::string system_hex() {
    return system_has_gap() ? std::string("gap") : save_hex(static_cast<Snapshot*>(TheCaveSystem));
}

static std::vector<std::string> observe(Scene& s) {
    std::ostringstream sys, mod, obj, qry, misc;
    sys << trackers().size() << ':';
    for (TunnelTracker* t : trackers()) {
        if (!t) { sys << "-;"; continue; }
        sys << save_hex(t) << ' ' << t->getContainCount() << ' ' << t->getHeroUnitsContained()
            << ' ' << t->friend_getTunnelCount() << ' ' << t->getContainerList()->size();
        for (ObjectID id : *t->getContainerList()) sys << ' ' << static_cast<uint32_t>(id);
        sys << ' ' << t->getContainedItemsList()->size();
        for (const Object* o : *t->getContainedItemsList()) sys << ' ' << static_cast<uint32_t>(o->getID());
        sys << ';';
    }
    for (uint32_t i = 0; i < s.caves; ++i) {
        if (!s.obj[i]) { mod << "- "; continue; }
        mod << save_hex(module_of(s.obj[i])) << ' ';
    }
    for (Object* o : s.obj) {
        if (!o) { obj << "-;"; continue; }
        const Coord3D* pos = o->getPosition();
        const StealthUpdate* stealth = o->getStealth();
        const bool detected = stealth &&
            const_cast<StealthUpdate*>(stealth)->*access_member(Pkt264DetectTag()) != 0;
        obj << (o->isDestroyed() ? 1 : 0) << ' ' << (o->isEffectivelyDead() ? 1 : 0) << ' '
            << s.team_index(o) << ' '
            << (o->getContainedBy() ? static_cast<uint32_t>(o->getContainedBy()->getID()) : 0u) << ' '
            << o->getContainedByFrame() << ' '
            << (o->testStatus(OBJECT_STATUS_MASKED) ? 1 : 0) << ' '
            << (o->testStatus(OBJECT_STATUS_UNSELECTABLE) ? 1 : 0) << ' '
            << (o->isDisabledByType(DISABLED_HELD) ? 1 : 0) << ' '
            << (o->friend_getPartitionData() ? 1 : 0) << ' '
            << (o->getIsUndetectedDefector() ? 1 : 0) << ' ' << (detected ? 1 : 0) << ' '
            << bits(o->getBodyModule()->getHealth()) << ' ' << bits(pos->x) << ' ' << bits(pos->y)
            << ' ' << bits(pos->z) << ';';
    }
    for (uint32_t i = 0; i < s.caves; ++i) {
        if (!s.obj[i]) { qry << "-;"; continue; }
        CaveContain* m = module_of(s.obj[i]);
        qry << m->getHeroUnitsContained() << ' ' << m->getStealthUnitsContained() << ' '
            << m->getPlayerWhoEntered() << ' ' << (m->isKickOutOnCapture() ? 1 : 0)
            << (m->isTunnelContain() ? 1 : 0) << (m->isBustable() ? 1 : 0)
            << (m->isGarrisonable() ? 1 : 0) << (m->isHealContain() ? 1 : 0)
            << (m->isImmuneToClearBuildingAttacks() ? 1 : 0)
            << (m->isSpecialZeroSlotContainer() ? 1 : 0)
            << (m->shouldDoOnBuildComplete() ? 1 : 0);
        if (!has_tracker(s.obj[i])) { qry << " n;"; continue; }
        qry << ' ' << m->getContainCount() << ' ' << m->getContainMax();
        uint32_t contained = 0, valid = 0, valid_full = 0;
        for (uint32_t k = 0; k < s.units; ++k) {
            const Object* u = s.obj[s.caves + k];
            if (!u) continue;
            if (m->isContained(u)) contained |= 1u << k;
            if (m->isValidContainerFor(u, FALSE)) valid |= 1u << k;
            if (m->isValidContainerFor(u, TRUE)) valid_full |= 1u << k;
        }
        qry << ' ' << contained << ' ' << valid << ' ' << valid_full;
        for (const Object* o : *m->getContainedItemsList()) qry << ' ' << static_cast<uint32_t>(o->getID());
        qry << ';';
    }
    misc << GetGameLogicRandomSeedCRC() << ' ' << s.orders;
    return {sys.str(), mod.str(), obj.str(), qry.str(), misc.str()};
}

static bool flush_is_safe(Scene& s) {
    for (TunnelTracker* t : trackers()) {
        if (!t) continue;
        for (const Object* o : *t->getContainedItemsList()) {
            if (o->isDestroyed()) return false;
        }
    }
    for (const Object* o : s.obj) {
        if (o && o->getContainedBy() && o->getContainedBy()->isDestroyed()) return false;
    }
    return true;
}

static void flush(Scene& s) {
    std::vector<size_t> gone;
    for (size_t i = 0; i < s.obj.size(); ++i) {
        if (s.obj[i] && s.obj[i]->isDestroyed()) gone.push_back(i);
    }
    s.w->flush_destroy_list();
    for (size_t i : gone) s.obj[i] = nullptr;
}

static void build_cave(Object* o) {
    for (BehaviorModule** b = o->getBehaviorModules(); *b; ++b) {
        CreateModuleInterface* c = (*b)->getCreate();
        if (c) c->onBuildComplete();
    }
}

// The AI orders issued by `issue`, as a mask over `s.obj`.
template <class F>
static void record_orders(Scene& s, Object* cave, CommandSourceType source, uint32_t seed, F issue) {
    for (Object* o : s.obj) {
        if (o && o->getAI()) o->getAI()->friend_setLastCommandSource(CMD_FROM_PLAYER);
    }
    issue();
    for (size_t k = 0; k < s.obj.size(); ++k) {
        Object* o = s.obj[k];
        if (o && o->getAI() && o->getAI()->getLastCommandSource() == source) {
            s.orders |= 1u << k;
            // `aiExit` also registers the rider with the container; that is the unported AI's
            // doing, so withdraw it.
            cave->getContain()->onObjectWantsToEnterOrExit(o, WANTS_NEITHER);
        }
    }
    // The AI state machines are unported: both sides restart the stream here.
    InitRandom(seed);
}

static void apply(Scene& s, const Op& op) {
    const auto cave = [&](uint32_t i) { return s.obj[i]; };
    // A cave whose index has a tracker, or null.
    const auto live = [&](uint32_t i) -> Object* { return has_tracker(s.obj[i]) ? s.obj[i] : nullptr; };
    const auto unit = [&](uint32_t i) { return s.obj[s.caves + i]; };
    const auto reachable = [&](uint32_t i) { return i >= s.caves ? s.obj[i] : live(i); };
    s.orders = 0;
    switch (op.code) {
        case OP_ENTER:
            if (live(op.b) && unit(op.a)) live(op.b)->getContain()->addToContain(unit(op.a));
            break;
        case OP_EXIT:
            if (live(op.b) && unit(op.a))
                live(op.b)->getContain()->removeFromContain(unit(op.a), op.c ? TRUE : FALSE);
            break;
        case OP_KILL_CAVE:
            if (cave(op.a) && (live(op.a) || under_construction(cave(op.a))))
                cave(op.a)->kill(DAMAGE_UNRESISTABLE, op.b ? DEATH_SUICIDED : DEATH_NORMAL);
            break;
        case OP_DESTROY_CAVE:
            if (cave(op.a)) TheGameLogic->destroyObject(cave(op.a));
            break;
        case OP_SELL:
            if (live(op.a))
                record_orders(s, live(op.a), CMD_FROM_AI, op.c,
                              [&] { live(op.a)->getContain()->onSelling(); });
            break;
        case OP_CAPTURE_CAVE:
            if (cave(op.a)) cave(op.a)->setTeam(s.team(op.b));
            break;
        case OP_BUILD:
            if (cave(op.a)) build_cave(cave(op.a));
            break;
        case OP_SWITCH:
            if (live(op.a)) module_of(live(op.a))->tryToSetCaveIndex(static_cast<Int>(op.b));
            break;
        case OP_DAMAGE:
            if (reachable(op.a)) {
                DamageInfo d;
                d.in.m_damageType = DAMAGE_EXPLOSION;
                d.in.m_deathType = DEATH_NORMAL;
                d.in.m_amount = static_cast<Real>(op.b);
                d.in.m_sourceID = op.c ? s.id.at(op.c - 1) : INVALID_ID;
                // A hit from or on a cave left without a team names no source.
                const Object* from = op.c ? s.obj.at(op.c - 1) : nullptr;
                if (from && (!from->getTeam() || !reachable(op.a)->getTeam()))
                    d.in.m_sourceID = INVALID_ID;
                reachable(op.a)->attemptDamage(&d);
            }
            break;
        case OP_DESTROY_UNIT:
            if (unit(op.a)) TheGameLogic->destroyObject(unit(op.a));
            break;
        case OP_KILL_UNIT:
            if (unit(op.a)) unit(op.a)->kill();
            break;
        case OP_REMOVE_ALL:
            if (live(op.a)) live(op.a)->getContain()->removeAllContained(op.b ? TRUE : FALSE);
            break;
        case OP_KILL_ALL:
            if (live(op.a)) live(op.a)->getContain()->killAllContained();
            break;
        case OP_HARM:
            if (live(op.a)) {
                DamageInfo d;
                d.in.m_damageType = DAMAGE_EXPLOSION;
                d.in.m_deathType = DEATH_NORMAL;
                d.in.m_amount = static_cast<Real>(op.b);
                live(op.a)->getContain()->harmAndForceExitAllContained(&d);
            }
            break;
        case OP_UPDATE:
            if (live(op.a) && !live(op.a)->isDestroyed()) module_of(live(op.a))->update();
            break;
        case OP_ADVANCE:
            s.w->set_frame(TheGameLogic->getFrame() + op.a);
            break;
        case OP_FLUSH:
            if (flush_is_safe(s)) flush(s);
            break;
        case OP_STATUS:
            if (unit(op.a)) {
                const ObjectStatusTypes st = op.b == 0 ? OBJECT_STATUS_STEALTHED : OBJECT_STATUS_DETECTED;
                if (op.c) unit(op.a)->setStatus(MAKE_OBJECT_STATUS_MASK(st));
                else unit(op.a)->clearStatus(MAKE_OBJECT_STATUS_MASK(st));
            }
            break;
        case OP_CAVE_STATUS:
            if (cave(op.a)) {
                const ObjectStatusTypes st = op.b == 0 ? OBJECT_STATUS_UNDER_CONSTRUCTION : OBJECT_STATUS_SOLD;
                if (op.c) cave(op.a)->setStatus(MAKE_OBJECT_STATUS_MASK(st));
                else cave(op.a)->clearStatus(MAKE_OBJECT_STATUS_MASK(st));
            }
            break;
        case OP_RECALC:
            if (live(op.a)) live(op.a)->getContain()->recalcApparentControllingPlayer();
            break;
        case OP_ORDER_EXIT:
        case OP_ORDER_IDLE:
            if (live(op.a)) {
                record_orders(s, live(op.a), CMD_FROM_SCRIPT, op.c, [&] {
                    if (op.code == OP_ORDER_EXIT)
                        live(op.a)->getContain()->orderAllPassengersToExit(CMD_FROM_SCRIPT, FALSE);
                    else
                        live(op.a)->getContain()->orderAllPassengersToIdle(CMD_FROM_SCRIPT);
                });
            }
            break;
        case OP_CAPTURE_UNIT:
            if (unit(op.a)) unit(op.a)->setTeam(s.team(op.b));
            break;
    }
}

// How often the hard cases occur, printed after generation.
enum { COV_CAVE_IN, COV_MOVED_TO_FRONT, COV_FRONT_GONE, COV_SHARED, COV_SWITCHED, COV_SWITCH_REFUSED,
       COV_REREGISTERED, COV_STEALTH_FORCED_OUT, COV_STEALTH_BURIED, COV_TEAM_CHANGED, COV_TEAMLESS,
       COV_OVER_CAPACITY, COV_LISTED_TWICE, COV_LISTED_IN_TWO, COV_UNBUILT_ENTERED, COV_FLUSH_SKIPPED,
       COV_DIED_INSIDE, COV_CAVE_LISTED_TWICE, COV_COUNT };
static const char* kCov[COV_COUNT] = {"cave_in_with_riders", "riders_moved_to_front",
    "riders_moved_to_missing_front", "occupied_index_shared", "index_switched",
    "switch_refused_occupied", "emptied_index_reregistered", "stealthed_forced_out",
    "stealthed_killed_by_cave_in", "cave_changed_team", "cave_left_teamless", "entered_over_capacity",
    "rider_listed_twice", "rider_listed_in_two_trackers", "entered_through_unlisted_cave",
    "flush_skipped", "listed_rider_destroyed", "cave_listed_twice"};
static uint32_t g_cov[COV_COUNT];

static void cover(Scene& s, const Op& op) {
    const size_t n = trackers().size();
    std::vector<std::vector<const Object*>> list(n);
    std::vector<size_t> caves(n);
    std::vector<bool> exists(n);
    for (size_t i = 0; i < n; ++i) {
        TunnelTracker* t = trackers()[i];
        exists[i] = t != nullptr;
        if (!t) continue;
        caves[i] = t->getContainerList()->size();
        list[i].assign(t->getContainedItemsList()->begin(), t->getContainedItemsList()->end());
    }
    std::vector<const Object*> holder;
    std::vector<uint32_t> team;
    std::vector<Int> index;
    std::vector<bool> stealthed;
    for (size_t k = 0; k < s.obj.size(); ++k) {
        const Object* o = s.obj[k];
        holder.push_back(o ? o->getContainedBy() : nullptr);
        team.push_back(o ? s.team_index(o) : 0u);
        index.push_back(o && k < s.caves ? index_of(o) : -1);
        stealthed.push_back(o && o->testStatus(OBJECT_STATUS_STEALTHED));
    }
    if (op.code == OP_FLUSH) {
        bool pending = false;
        for (Object* o : s.obj) pending = pending || (o && o->isDestroyed());
        if (pending && !flush_is_safe(s)) ++g_cov[COV_FLUSH_SKIPPED];
    }
    if (op.code == OP_SWITCH && has_tracker(s.obj[op.a]) &&
        !TheCaveSystem->canSwitchIndexToIndex(index[op.a], static_cast<Int>(op.b)))
        ++g_cov[COV_SWITCH_REFUSED];
    apply(s, op);
    if (op.code == OP_SWITCH && s.obj[op.a] && index_of(s.obj[op.a]) != index[op.a]) ++g_cov[COV_SWITCHED];
    for (size_t i = 0; i < trackers().size(); ++i) {
        TunnelTracker* t = trackers()[i];
        if (!t) continue;
        const ContainedItemsList* now = t->getContainedItemsList();
        const bool known = i < n && exists[i];
        if (known && caves[i] && t->getContainerList()->empty() && !list[i].empty()) ++g_cov[COV_CAVE_IN];
        if (known && !caves[i] && !t->getContainerList()->empty()) ++g_cov[COV_REREGISTERED];
        if (t->getContainerList()->size() > 1 && !now->empty()) ++g_cov[COV_SHARED];
        if (op.code == OP_ENTER && known && now->size() > list[i].size() &&
            static_cast<Int>(now->size()) > TheGlobalData->m_maxTunnelCapacity)
            ++g_cov[COV_OVER_CAPACITY];
        if (op.code == OP_ENTER && known && now->size() > list[i].size()) {
            const ObjectID through = s.id[op.b];
            const std::list<ObjectID>& ids = *t->getContainerList();
            if (std::find(ids.begin(), ids.end(), through) == ids.end()) ++g_cov[COV_UNBUILT_ENTERED];
        }
        std::vector<ObjectID> ids(t->getContainerList()->begin(), t->getContainerList()->end());
        std::sort(ids.begin(), ids.end());
        if (std::adjacent_find(ids.begin(), ids.end()) != ids.end()) ++g_cov[COV_CAVE_LISTED_TWICE];
        for (const Object* o : *now) {
            if (std::count(now->begin(), now->end(), o) > 1) { ++g_cov[COV_LISTED_TWICE]; break; }
        }
        for (size_t j = i + 1; j < trackers().size(); ++j) {
            if (!trackers()[j]) continue;
            const ContainedItemsList* other = trackers()[j]->getContainedItemsList();
            for (const Object* o : *now) {
                if (std::find(other->begin(), other->end(), o) != other->end()) { ++g_cov[COV_LISTED_IN_TWO]; break; }
            }
        }
    }
    for (size_t i = 0; i < s.obj.size(); ++i) {
        Object* o = s.obj[i];
        if (!o) continue;
        if (holder[i] && o->getContainedBy() && o->getContainedBy() != holder[i] && op.code != OP_ENTER)
            ++g_cov[COV_MOVED_TO_FRONT];
        if (holder[i] && !o->getContainedBy() && !o->isDestroyed() &&
            (op.code == OP_KILL_CAVE || op.code == OP_SWITCH || op.code == OP_DAMAGE) &&
            o->testStatus(OBJECT_STATUS_UNSELECTABLE))
            ++g_cov[COV_FRONT_GONE];
        if (stealthed[i] && holder[i] && !o->getContainedBy()) {
            if (op.code == OP_REMOVE_ALL || op.code == OP_EXIT) ++g_cov[COV_STEALTH_FORCED_OUT];
            if (op.code == OP_KILL_CAVE && o->isDestroyed()) ++g_cov[COV_STEALTH_BURIED];
        }
        if (i < s.caves && team[i] != s.team_index(o) && op.code != OP_CAPTURE_CAVE) {
            ++g_cov[COV_TEAM_CHANGED];
            if (s.team_index(o) == 4) ++g_cov[COV_TEAMLESS];
        }
        if (i >= s.caves && holder[i] && o->isDestroyed() &&
            (op.code == OP_DAMAGE || op.code == OP_DESTROY_UNIT || op.code == OP_KILL_UNIT))
            ++g_cov[COV_DIED_INSIDE];
    }
}

static Op random_op(uint32_t caves, uint32_t units, uint32_t spawned) {
    static const uint32_t weight[OP_COUNT] = {
        30, 8, 5, 3, 2, 3, 6, 8, 6, 3, 3, 5, 1, 1, 1, 3, 4, 5, 3, 3, 1, 1, 3};
    uint32_t total = 0;
    for (uint32_t w : weight) total += w;
    uint32_t r = pick(total), code = 0;
    while (r >= weight[code]) r -= weight[code++];
    Op op{code, 0, 0, 0};
    switch (code) {
        case OP_ENTER: op.a = pick(units); op.b = pick(caves); break;
        case OP_EXIT: op.a = pick(units); op.b = pick(caves); op.c = pick(2); break;
        case OP_KILL_CAVE: op.a = pick(caves); op.b = pick(3) == 0; break;
        case OP_DESTROY_CAVE: case OP_BUILD: case OP_UPDATE: case OP_KILL_ALL: case OP_RECALC:
            op.a = pick(caves); break;
        case OP_SELL: op.a = pick(caves); op.c = 1u + pick(1000000); break;
        case OP_CAPTURE_CAVE: op.a = pick(caves); op.b = pick(4); break;
        case OP_SWITCH: op.a = pick(caves); op.b = pick(3) == 0 ? pick(6) : pick(3); break;
        case OP_DAMAGE:
            op.a = pick(2) ? pick(caves) : caves + pick(units);
            op.b = pick(5) == 0 ? 600u : 1u + pick(90);
            op.c = pick(3) == 0 ? 0u : 1u + pick(spawned);
            break;
        case OP_DESTROY_UNIT: case OP_KILL_UNIT: op.a = pick(units); break;
        case OP_REMOVE_ALL: op.a = pick(caves); op.b = pick(2); break;
        case OP_HARM: op.a = pick(caves); op.b = 1u + pick(60); break;
        case OP_ADVANCE: op.a = 1u + pick(100); break;
        case OP_STATUS: op.a = pick(units); op.b = pick(2); op.c = pick(4) != 0; break;
        case OP_CAVE_STATUS: op.a = pick(caves); op.b = pick(2); op.c = pick(3) == 0; break;
        case OP_ORDER_EXIT: case OP_ORDER_IDLE: op.a = pick(caves); op.c = 1u + pick(1000000); break;
        case OP_CAPTURE_UNIT: op.a = pick(units); op.b = pick(4); break;
        default: break;
    }
    return op;
}

// Cave rows are (type, team, x, y, built); unit rows are (type, team, x, y, damage).

// Vector 0 walks the shared-index cases in order.
static void scripted(Vec& v) {
    v.cap = 3;
    v.frame = 100;
    // Caves 0-2 share index 0 (cave 2 dies only to suicide), cave 3 has index 1 and cave 4,
    // also index 0, is not yet built. Units: infantry, hero, stealth, aircraft, DestroyDie, AI.
    v.p = {5, 0, 0, 100, 100, 1, 1, 0, 160, 100, 1, 2, 0, 220, 100, 1, 3, 0, 280, 100, 1,
              7, 0, 340, 100, 0,
           6, U_INF, 1, 100, 140, 40, U_HERO, 1, 120, 140, 90, U_STEALTH, 1, 140, 140, 0,
              U_AIR, 1, 160, 140, 0, U_DD, 1, 180, 140, 0, U_AI, 1, 200, 140, 0};
    const uint32_t c = 5;
    v.ops = {
        {OP_ENTER, 0, 0, 0}, {OP_ENTER, 1, 1, 0}, {OP_ENTER, 2, 0, 0},
        {OP_ENTER, 3, 1, 0},                          // an aircraft, over capacity
        {OP_SWITCH, 0, 1, 0},                         // refused: occupied
        {OP_SWITCH, 3, 0, 0},                         // refused: the target is occupied
        {OP_EXIT, 0, 1, 0},                           // out through a cave it did not enter
        {OP_BUILD, 4, 0, 0}, {OP_ENTER, 4, 4, 0}, {OP_ENTER, 5, 4, 0},
        {OP_ORDER_EXIT, 1, 0, 77}, {OP_ORDER_IDLE, 1, 0, 78}, {OP_SELL, 1, 0, 79},
        {OP_KILL_CAVE, 0, 0, 0},                      // passengers move to the front cave
        {OP_DESTROY_CAVE, 0, 0, 0}, {OP_FLUSH, 0, 0, 0},
        {OP_UPDATE, 1, 0, 0}, {OP_KILL_ALL, 1, 0, 0}, {OP_HARM, 1, 30, 0},
        {OP_DAMAGE, c + 4, 90, 0},                    // DestroyDie passenger dies inside
        {OP_FLUSH, 0, 0, 0},
        {OP_REMOVE_ALL, 1, 1, 0},                     // forces the stealthed unit out
        {OP_SWITCH, 3, 4, 0}, {OP_SWITCH, 3, 1, 0},
        {OP_ENTER, 2, 1, 0}, {OP_ENTER, 0, 2, 0},
        {OP_KILL_CAVE, 4, 0, 0}, {OP_KILL_CAVE, 1, 0, 0},
        {OP_KILL_CAVE, 2, 0, 0},                      // ignores a normal death
        {OP_KILL_CAVE, 2, 1, 0},                      // the last cave: a cave in
        {OP_FLUSH, 0, 0, 0},
        {OP_ENTER, 1, 3, 0}, {OP_RECALC, 3, 0, 0}, {OP_EXIT, 1, 3, 1}, {OP_RECALC, 3, 0, 0},
    };
}

// Vector 1 switches the only cave of an index, empties it and registers it again.
static void scripted_reregister(Vec& v) {
    v.cap = 10;
    v.frame = 200;
    v.p = {4, 0, 0, 100, 100, 1, 3, 0, 160, 100, 1, 1, 0, 220, 100, 0, 6, 2, 280, 100, 0,
           2, U_INF, 1, 100, 140, 0, U_STEALTH_HERO, 2, 120, 140, 0};
    v.ops = {
        {OP_SWITCH, 0, 0, 0}, {OP_SWITCH, 0, 1, 0}, {OP_SWITCH, 1, 0, 0}, {OP_SWITCH, 1, 5, 0},
        {OP_KILL_CAVE, 0, 0, 0},                      // empties index 1 with nobody inside
        {OP_BUILD, 2, 0, 0}, {OP_ENTER, 0, 2, 0}, {OP_BUILD, 3, 0, 0},
        {OP_ENTER, 1, 3, 0}, {OP_STATUS, 1, 0, 1}, {OP_KILL_CAVE, 2, 0, 0},
        {OP_FLUSH, 0, 0, 0}, {OP_SWITCH, 3, 0, 0}, {OP_SWITCH, 1, 0, 0},
        {OP_DESTROY_CAVE, 3, 0, 0}, {OP_FLUSH, 0, 0, 0},
    };
}

// Vector 2: a cave built into an occupied network has no original team. Emptying the network
// through it leaves every cave of the index without a team.
static void scripted_teamless(Vec& v) {
    v.cap = 10;
    v.frame = 300;
    v.p = {2, 0, 0, 100, 100, 1, 1, 0, 160, 100, 0,
           2, U_INF, 1, 100, 140, 0, U_INF, 2, 120, 140, 0};
    v.ops = {
        {OP_ENTER, 0, 0, 0}, {OP_BUILD, 1, 0, 0}, {OP_REMOVE_ALL, 1, 0, 0},
        {OP_ENTER, 1, 0, 0}, {OP_RECALC, 1, 0, 0}, {OP_CAPTURE_CAVE, 0, 2, 0},
        {OP_EXIT, 1, 0, 0}, {OP_ENTER, 0, 1, 0}, {OP_EXIT, 0, 1, 0},
        {OP_KILL_CAVE, 0, 0, 0}, {OP_KILL_CAVE, 1, 0, 0},
    };
}

// Vector 3: a cave deleted without dying stays listed; riders moved to it end up nowhere.
static void scripted_deleted_front(Vec& v) {
    v.cap = 10;
    v.frame = 400;
    v.p = {3, 0, 0, 100, 100, 1, 1, 0, 160, 100, 1, 3, 0, 220, 100, 1,
           2, U_INF, 1, 100, 140, 0, U_HERO, 1, 120, 140, 0};
    v.ops = {
        {OP_DESTROY_CAVE, 0, 0, 0}, {OP_FLUSH, 0, 0, 0},
        {OP_ENTER, 0, 1, 0}, {OP_ENTER, 1, 1, 0}, {OP_KILL_CAVE, 1, 0, 0},
        {OP_ENTER, 0, 2, 0}, {OP_ENTER, 1, 2, 0}, {OP_EXIT, 0, 2, 0}, {OP_ENTER, 0, 2, 0},
        {OP_KILL_UNIT, 0, 0, 0}, {OP_ADVANCE, 5, 0, 0}, {OP_DESTROY_UNIT, 1, 0, 0},
        {OP_FLUSH, 0, 0, 0},
    };
}

static void generate(Vec& v, int i) {
    static const uint32_t caps[8] = {0, 1, 2, 3, 10, 10, 10, 10};
    v.cap = caps[pick(8)];
    v.frame = pick(8) == 0 ? pick(30) : 100 + pick(5000);
    v.seed = next_u32() & 0x7fffffff;
    if (i == 0) { scripted(v); return; }
    if (i == 1) { scripted_reregister(v); return; }
    if (i == 2) { scripted_teamless(v); return; }
    if (i == 3) { scripted_deleted_front(v); return; }
    const uint32_t caves = 1u + pick(5);
    const uint32_t units = 1u + pick(7);
    v.p = {caves};
    static const uint32_t shared[4] = {0, 1, 2, 7};  // all index 0
    for (uint32_t k = 0; k < caves; ++k) {
        const uint32_t type = pick(3) ? shared[pick(4)] : pick(kCaveTypes);
        const uint32_t team = pick(3) == 0 ? pick(3) : 0u;
        v.p.insert(v.p.end(), {type, team, 80 + pick(300), 80 + pick(300), pick(5) != 0});
    }
    v.p.push_back(units);
    const uint32_t home = 1u + pick(2);
    for (uint32_t k = 0; k < units; ++k) {
        const uint32_t team = pick(4) == 0 ? pick(4) : home;
        const uint32_t type = pick(4) == 0 ? U_STEALTH + pick(2) : pick(U_COUNT);
        v.p.insert(v.p.end(), {type, team, 80 + pick(300), 80 + pick(300),
                               pick(2) ? 0u : 1u + pick(55)});
    }
    const uint32_t spawned = caves + units;
    const uint32_t count = 12u + pick(36);
    for (uint32_t k = 0; k < count; ++k) v.ops.push_back(random_op(caves, units, spawned));
}

static void run(oracle_wp51::World& w, Vec& v) {
    Scene s;
    s.w = &w;
    TheCaveSystem->reset();
    TheWritableGlobalData->m_maxTunnelCapacity = static_cast<Int>(v.cap);
    size_t at = 0;
    s.caves = v.p[at++];
    const auto place = [&](Object* o, uint32_t x, uint32_t y) {
        Coord3D pos = {static_cast<Real>(x), static_cast<Real>(y), 0.0f};
        o->setPosition(&pos);
        s.obj.push_back(o);
        s.id.push_back(o->getID());
    };
    for (uint32_t k = 0; k < s.caves; ++k, at += 5) {
        const std::string name = "P264_Cave" + std::to_string(v.p[at]);
        place(w.spawn_object(name.c_str(), s.team(v.p[at + 1])), v.p[at + 2], v.p[at + 3]);
        if (v.p[at + 4]) build_cave(s.obj.back());
    }
    s.units = v.p[at++];
    std::vector<size_t> transports;
    for (uint32_t k = 0; k < s.units; ++k, at += 5) {
        place(w.spawn_object(kUnit[v.p[at]], s.team(v.p[at + 1])), v.p[at + 2], v.p[at + 3]);
        if (v.p[at + 4]) {
            DamageInfo d;
            d.in.m_damageType = DAMAGE_EXPLOSION;
            d.in.m_deathType = DEATH_NORMAL;
            d.in.m_amount = static_cast<Real>(v.p[at + 4]);
            s.obj.back()->attemptDamage(&d);
        }
        if (v.p[at] == U_STEALTH || v.p[at] == U_STEALTH_HERO)
            s.obj.back()->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_STEALTHED));
        if (v.p[at] == U_TRANS) transports.push_back(s.obj.size() - 1);
    }
    // One passenger inside every transport, spawned after all units.
    for (size_t k : transports) {
        Object* carrier = s.obj[k];
        Object* rider = w.spawn_object("P264_Inf", carrier->getTeam());
        place(rider, 100, 100);
        carrier->getContain()->addToContain(rider);
    }
    ThePartitionManager->update();

    InitRandom(v.seed);
    w.set_frame(v.frame);
    v.rows.push_back(observe(s));
    for (const Op& op : v.ops) {
        cover(s, op);
        v.rows.push_back(observe(s));
    }
    v.rng = GetGameLogicRandomSeedCRC();
    v.final_system = system_hex();
    uint32_t crc = 0;
    for (uint32_t i = 0; i < s.caves; ++i) {
        if (!s.obj[i]) continue;
        v.final_modules += save_hex(module_of(s.obj[i])) + " ";
        crc = crc * 31u + oracle_wp51::World::snapshot_crc(module_of(s.obj[i]));
    }
    v.crc = crc;

    // Tear down: unhook every passenger first so no stale pointer survives.
    for (TunnelTracker* t : trackers()) {
        if (!t) continue;
        ContainedItemsList list;
        t->swapContainedItemsList(list);
    }
    for (size_t i = s.caves; i < s.obj.size(); ++i) {
        Object* o = s.obj[i];
        if (!o || !o->getContainedBy()) continue;
        for (uint32_t c = 0; c < s.caves; ++c) {
            if (o->getContainedBy() == s.obj[c]) { o->onRemovedFrom(s.obj[c]); break; }
        }
    }
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        if (!o->isDestroyed()) TheGameLogic->destroyObject(o);
    }
    w.flush_destroy_list();
    ThePartitionManager->update();
}

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_pkt264_cave <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    const int dump = std::getenv("PKT264_DUMP") ? std::atoi(std::getenv("PKT264_DUMP")) : -1;
    oracle_wp51::World w;
    w.load_ini(fixture_ini());
    w.init_with_neutral_player("FactionP264", nullptr);
    TheWritableGlobalData->m_unitDamagedThresh = 0.7f;
    TheWritableGlobalData->m_unitReallyDamagedThresh = 0.35f;
    TheCaveSystem = new CaveSystem;
    Dict dict_a2;
    TheTeamFactory->initTeam("TeamA2", AsciiString::TheEmptyString, TRUE, &dict_a2);
    g_team_a2 = TheTeamFactory->findTeam("TeamA2");
    g_team_a2->setControllingPlayer(w.team_a->getControllingPlayer());
    for (Team* t : {w.team_neutral, w.team_a, w.team_b}) {
        if (t->getControllingPlayer()->getDefaultTeam() != t) {
            std::fprintf(stderr, "not a default team\n");
            return 1;
        }
    }

    // Teams change under a looking cave. Reveal the map to everyone first, each player while
    // another one is local: a local player's shroud edge needs a display.
    Player* const players[3] = {w.team_neutral->getControllingPlayer(),
                                w.team_a->getControllingPlayer(), w.team_b->getControllingPlayer()};
    Player* const was_local = ThePlayerList->getLocalPlayer();
    for (int i = 0; i < 3; ++i) {
        ThePlayerList->setLocalPlayer(players[(i + 1) % 3]);
        ThePartitionManager->revealMapForPlayerPermanently(players[i]->getPlayerIndex());
    }
    ThePlayerList->setLocalPlayer(was_local);

    const int kTotal = 1000;
    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    const char* root = "vendor/GeneralsX/";
    std::fprintf(f, "{\n\"packet\": \"PKT-264\",\n\"vector_count\": %d,\n\"local_player\": %d,\n", kTotal, was_local->getPlayerIndex());
    std::fprintf(f, "\"team_ids\": [%u,%u,%u,%u],\n", w.team_neutral->getID(), w.team_a->getID(),
                 w.team_b->getID(), g_team_a2->getID());
    std::fprintf(f, "\"source_sha256\": {\"contain\":\"%s\",\"system\":\"%s\"},\n",
        oracle::sha256_file((std::string(root) + "GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Contain/CaveContain.cpp").c_str()).c_str(),
        oracle::sha256_file((std::string(root) + "Core/GameEngine/Source/GameLogic/System/CaveSystem.cpp").c_str()).c_str());
    std::vector<Vec> vecs;
    for (int i = 0; i < kTotal; ++i) {
        Vec v;
        generate(v, i);
        if (std::getenv("PKT264_TRACE")) std::fprintf(stderr, "vector %d\n", i);
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
    std::fprintf(f, "\"seed0_diagnostics\": [\n");
    for (size_t r = 0; r < vecs[0].rows.size(); ++r) {
        std::fprintf(f, "{\"row\":%zu", r);
        const char* names[5] = {"system", "modules", "objects", "queries", "misc"};
        for (int k = 0; k < 5; ++k) std::fprintf(f, ",\"%s\":\"%s\"", names[k], vecs[0].rows[r][k].c_str());
        std::fprintf(f, "}%s\n", r + 1 < vecs[0].rows.size() ? "," : "");
    }
    std::fprintf(f, "],\n\"vectors\": [\n");
    for (int i = 0; i < kTotal; ++i) {
        const Vec& v = vecs[i];
        std::fprintf(f, "{\"id\":%d,\"cap\":%u,\"frame\":%u,\"seed\":%u,\"p\":[", i, v.cap, v.frame, v.seed);
        for (size_t k = 0; k < v.p.size(); ++k) std::fprintf(f, "%s%u", k ? "," : "", v.p[k]);
        std::fprintf(f, "],\"ops\":[");
        for (size_t k = 0; k < v.ops.size(); ++k) {
            const Op& o = v.ops[k];
            std::fprintf(f, "%s%u,%u,%u,%u", k ? "," : "", o.code, o.a, o.b, o.c);
        }
        std::fprintf(f, "],\"rows\":\"");
        for (size_t r = 0; r < v.rows.size(); ++r) {
            for (int k = 0; k < 5; ++k) std::fprintf(f, "%08x", fnv32(v.rows[r][k]));
        }
        std::fprintf(f, "\",\"final_system\":\"%s\",\"final_modules\":%u,\"crc\":%u,\"rng\":%u}%s\n",
                     v.final_system.c_str(), fnv32(v.final_modules), v.crc, v.rng,
                     i + 1 < kTotal ? "," : "");
    }
    std::fprintf(f, "]\n}\n");
    std::fclose(f);
    if (std::rename(tmp.c_str(), out_path.c_str()) != 0) { std::perror("rename"); return 1; }

    std::fprintf(stderr, "wrote %d vectors\n", kTotal);
    for (int c = 0; c < COV_COUNT; ++c) std::fprintf(stderr, "%s=%u ", kCov[c], g_cov[c]);
    std::fprintf(stderr, "\n");
    return 0;
}
