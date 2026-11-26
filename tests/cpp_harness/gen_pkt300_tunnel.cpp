// PKT-300 TunnelContain and the parts of `TunnelTracker` it reaches.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt300_tunnel.cpp --opt O0 -o <dir>/pkt300_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt300_tunnel.cpp --opt O2 -o <dir>/pkt300_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt300_tunnel.cpp --opt O2 --sanitize -o <dir>/pkt300_san
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/pkt300_o2 docs/golden_vectors/PKT300_Tunnel.json
//
// Each vector is a script of operations on up to four tunnels and their passengers. After every
// operation a row records one FNV-1a digest per section: trackers, tunnel modules, objects,
// contain queries, and misc (PRNG, AI orders, nemesis). Vector 0 also stores the section text.
// Set PKT300_DUMP=<index> to print the section text of any other vector to stderr.
//
// States left out because the reference reads freed memory in them (the flush is skipped, the
// operations themselves are not restricted):
//   - flushing the destroy list while a passenger list holds a destroyed object, which happens
//     after a unit is exited through another player's tunnel and then destroyed;
//   - flushing while an object's container is destroyed, which happens after a unit enters a
//     sold or killed tunnel that is later destroyed.
// Also left out: `orderAllPassengersToExit` with `instantly`, where the unported AI leaves the
// container itself; and, after an order, the AI's "wants to exit" entry is withdrawn.

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/RandomValue.h"
#include "Common/Team.h"
#include "Common/ThingTemplate.h"
#include "Common/TunnelTracker.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/TunnelContain.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

struct Pkt300NemesisTag { using type = ObjectID TunnelTracker::*; };
template struct MemberAccess<Pkt300NemesisTag, &TunnelTracker::m_curNemesisID>;
ObjectID TunnelTracker::* access_member(Pkt300NemesisTag);
struct Pkt300NemesisTimeTag { using type = UnsignedInt TunnelTracker::*; };
template struct MemberAccess<Pkt300NemesisTimeTag, &TunnelTracker::m_nemesisTimestamp>;
UnsignedInt TunnelTracker::* access_member(Pkt300NemesisTimeTag);

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
    OP_ENTER, OP_EXIT, OP_KILL_TUNNEL, OP_DESTROY_TUNNEL, OP_SELL, OP_CAPTURE_TUNNEL, OP_UPDATE,
    OP_DAMAGE, OP_DESTROY_UNIT, OP_KILL_UNIT, OP_REMOVE_ALL, OP_KILL_ALL, OP_HARM, OP_ADVANCE,
    OP_FLUSH, OP_UPDATE_NEMESIS, OP_GET_NEMESIS, OP_STATUS, OP_ORDER_EXIT, OP_ORDER_IDLE,
    OP_CAPTURE_UNIT, OP_COUNT
};

static const int kTunnelTypes = 8;
enum { U_INF, U_HERO, U_VEH, U_AIR, U_DRONE, U_STEALTH, U_DD, U_AI, U_TRANS, U_COUNT };
static const char* kUnit[U_COUNT] = {"P300_Inf", "P300_Hero", "P300_Veh", "P300_Air",
                                     "P300_Drone", "P300_Stealth", "P300_DestroyDie",
                                     "P300_Ai", "P300_Trans"};

struct Op { uint32_t code, a, b, c; };
struct Vec {
    uint32_t cap = 0, frame = 0, seed = 0;
    std::vector<uint32_t> p;
    std::vector<Op> ops;
    std::vector<std::vector<std::string>> rows;
    std::string final_trackers, final_modules;
    uint32_t crc = 0, rng = 0;
};

static std::string body(const char* health) {
    return std::string("  Body = ActiveBody ModuleTag_Body\n    MaxHealth = ") + health +
           "\n    InitialHealth = " + health + "\n  End\n";
}

static std::string fixture_ini() {
    std::ostringstream ss;
    ss << "PlayerTemplate FactionP300\n  Side = America\n  PlayableSide = Yes\nEnd\n";
    const char* heal[kTunnelTypes] = {"0", "100", "1000", "5000", "333", "1000", "2000", "10"};
    const std::string destroy_die = "  Behavior = DestroyDie ModuleTag_Die\n  End\n";
    for (int i = 0; i < kTunnelTypes; ++i) {
        std::string contain = std::string("  Behavior = TunnelContain ModuleTag_Contain\n") +
                              "    TimeForFullHeal = " + heal[i] + "\n";
        if (i == 5) contain += "    DeathTypes = NONE +SUICIDED\n";
        contain += "  End\n";
        ss << "Object P300_Tunnel" << i << "\n  KindOf = PRELOAD STRUCTURE\n" << body("500.0");
        if (i == 7) ss << destroy_die;
        ss << contain;
        if (i == 6) ss << destroy_die;
        ss << "End\n";
    }
    ss << "Object P300_Inf\n  KindOf = PRELOAD INFANTRY\n  TransportSlotCount = 1\n" << body("100.0") << "End\n";
    ss << "Object P300_Hero\n  KindOf = PRELOAD INFANTRY HERO\n" << body("200.0") << "End\n";
    ss << "Object P300_Veh\n  KindOf = PRELOAD VEHICLE\n" << body("300.0") << "End\n";
    ss << "Object P300_Air\n  KindOf = PRELOAD VEHICLE AIRCRAFT\n" << body("150.0") << "End\n";
    ss << "Object P300_Drone\n  KindOf = PRELOAD DRONE\n" << body("70.0") << "End\n";
    ss << "Object P300_Stealth\n  KindOf = PRELOAD INFANTRY STEALTH_GARRISON\n" << body("90.0") << "End\n";
    ss << "Object P300_DestroyDie\n  KindOf = PRELOAD INFANTRY\n" << body("60.0") << destroy_die << "End\n";
    ss << "Object P300_Ai\n  KindOf = PRELOAD INFANTRY\n" << body("110.0")
       << "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\nEnd\n";
    ss << "Object P300_Trans\n  KindOf = PRELOAD VEHICLE\n" << body("250.0")
       << "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 8\n"
       << "    AllowInsideKindOf = INFANTRY\n  End\nEnd\n";
    return ss.str();
}

struct Scene {
    oracle_wp51::World* w = nullptr;
    uint32_t tunnels = 0, units = 0;
    std::vector<Object*> obj;  // tunnels, then units, then transport riders; null once flushed
    std::vector<ObjectID> id;
    uint32_t orders = 0;
    uint32_t nemesis = 0;

    Team* team(uint32_t i) const {
        if (i == 3) return g_team_a2;
        return i == 0 ? w->team_neutral : (i == 1 ? w->team_a : w->team_b);
    }
    uint32_t team_index(const Object* o) const {
        if (o->getTeam() == g_team_a2) return 3u;
        return o->getTeam() == w->team_neutral ? 0u : (o->getTeam() == w->team_a ? 1u : 2u);
    }
    TunnelTracker* tracker(uint32_t i) const {
        return team(i)->getControllingPlayer()->getTunnelSystem();
    }
};

static TunnelContain* module_of(Object* o) { return static_cast<TunnelContain*>(o->getContain()); }

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

static std::vector<std::string> observe(Scene& s) {
    std::ostringstream trk, mod, obj, qry, misc;
    for (uint32_t p = 0; p < 3; ++p) {
        TunnelTracker* t = s.tracker(p);
        trk << save_hex(t) << ' ' << t->getContainCount() << ' ' << t->getHeroUnitsContained()
            << ' ' << t->friend_getTunnelCount() << ' ' << t->getContainerList()->size();
        for (ObjectID id : *t->getContainerList()) trk << ' ' << static_cast<uint32_t>(id);
        trk << ' ' << t->getContainedItemsList()->size();
        for (const Object* o : *t->getContainedItemsList()) trk << ' ' << static_cast<uint32_t>(o->getID());
        trk << ' ' << static_cast<uint32_t>(t->*access_member(Pkt300NemesisTag())) << ' '
            << t->*access_member(Pkt300NemesisTimeTag()) << ';';
    }
    for (uint32_t i = 0; i < s.tunnels; ++i) {
        if (!s.obj[i]) { mod << "- "; continue; }
        mod << save_hex(module_of(s.obj[i])) << ' ';
    }
    for (Object* o : s.obj) {
        if (!o) { obj << "-;"; continue; }
        const Coord3D* pos = o->getPosition();
        obj << (o->isDestroyed() ? 1 : 0) << ' ' << (o->isEffectivelyDead() ? 1 : 0) << ' '
            << s.team_index(o) << ' '
            << (o->getContainedBy() ? static_cast<uint32_t>(o->getContainedBy()->getID()) : 0u) << ' '
            << o->getContainedByFrame() << ' '
            << (o->testStatus(OBJECT_STATUS_MASKED) ? 1 : 0) << ' '
            << (o->testStatus(OBJECT_STATUS_UNSELECTABLE) ? 1 : 0) << ' '
            << (o->isDisabledByType(DISABLED_HELD) ? 1 : 0) << ' '
            << (o->friend_getPartitionData() ? 1 : 0) << ' '
            << bits(o->getBodyModule()->getHealth()) << ' ' << bits(pos->x) << ' ' << bits(pos->y)
            << ' ' << bits(pos->z) << ';';
    }
    for (uint32_t i = 0; i < s.tunnels; ++i) {
        if (!s.obj[i]) { qry << "-;"; continue; }
        TunnelContain* m = module_of(s.obj[i]);
        qry << m->getContainCount() << ' ' << m->getContainMax() << ' ' << m->getHeroUnitsContained()
            << ' ' << m->getStealthUnitsContained() << ' ' << m->getPlayerWhoEntered() << ' '
            << (m->isKickOutOnCapture() ? 1 : 0) << (m->isTunnelContain() ? 1 : 0)
            << (m->isBustable() ? 1 : 0) << (m->isGarrisonable() ? 1 : 0)
            << (m->isHealContain() ? 1 : 0) << (m->isImmuneToClearBuildingAttacks() ? 1 : 0)
            << (m->isSpecialZeroSlotContainer() ? 1 : 0) << ' ';
        uint32_t contained = 0, valid = 0, valid_full = 0;
        for (uint32_t k = 0; k < s.units; ++k) {
            const Object* u = s.obj[s.tunnels + k];
            if (!u) continue;
            if (m->isContained(u)) contained |= 1u << k;
            if (m->isValidContainerFor(u, FALSE)) valid |= 1u << k;
            if (m->isValidContainerFor(u, TRUE)) valid_full |= 1u << k;
        }
        qry << contained << ' ' << valid << ' ' << valid_full;
        const ContainedItemsList* items = m->getContainedItemsList();
        for (const Object* o : *items) qry << ' ' << static_cast<uint32_t>(o->getID());
        qry << ';';
    }
    misc << GetGameLogicRandomSeedCRC() << ' ' << s.orders << ' ' << s.nemesis;
    return {trk.str(), mod.str(), obj.str(), qry.str(), misc.str()};
}

static void apply(Scene& s, const Op& op);
static bool flush_is_safe(Scene& s);

// How often the hard cases occur, printed after generation.
enum { COV_LAST_WITH_RIDERS, COV_MOVED_TO_FRONT, COV_OVER_CAPACITY, COV_HEALED, COV_NEMESIS,
       COV_DIED_INSIDE, COV_FLUSH_SKIPPED, COV_LISTED_TWICE, COV_SELL_EMPTIES, COV_CAPTURE_KILLS,
       COV_COUNT };
static const char* kCov[COV_COUNT] = {"last_tunnel_with_riders", "riders_moved_to_front",
    "entered_over_capacity", "updates_that_healed", "nemesis_found", "listed_rider_destroyed",
    "flush_skipped", "rider_listed_twice", "sell_emptied", "capture_destroyed_riders"};
static uint32_t g_cov[COV_COUNT];

static void cover(Scene& s, const Op& op) {
    std::vector<size_t> tunnels(3), size(3);
    std::vector<std::vector<const Object*>> list(3);
    std::vector<const Object*> holder;
    std::vector<Real> health;
    for (uint32_t p = 0; p < 3; ++p) {
        tunnels[p] = s.tracker(p)->getContainerList()->size();
        list[p].assign(s.tracker(p)->getContainedItemsList()->begin(),
                       s.tracker(p)->getContainedItemsList()->end());
    }
    for (Object* o : s.obj) {
        holder.push_back(o ? o->getContainedBy() : nullptr);
        health.push_back(o ? o->getBodyModule()->getHealth() : 0.0f);
    }
    if (op.code == OP_FLUSH) {
        bool pending = false;
        for (Object* o : s.obj) pending = pending || (o && o->isDestroyed());
        if (pending && !flush_is_safe(s)) ++g_cov[COV_FLUSH_SKIPPED];
    }
    apply(s, op);
    for (uint32_t p = 0; p < 3; ++p) {
        const ContainedItemsList* now = s.tracker(p)->getContainedItemsList();
        if (tunnels[p] && s.tracker(p)->getContainerList()->empty() && !list[p].empty()) {
            ++g_cov[COV_LAST_WITH_RIDERS];
            if (op.code == OP_CAPTURE_TUNNEL) ++g_cov[COV_CAPTURE_KILLS];
        }
        if (op.code == OP_SELL && !list[p].empty() && now->empty()) ++g_cov[COV_SELL_EMPTIES];
        if (now->size() > list[p].size() &&
            static_cast<Int>(now->size()) > TheGlobalData->m_maxTunnelCapacity)
            ++g_cov[COV_OVER_CAPACITY];
        for (const Object* o : *now) {
            if (std::count(now->begin(), now->end(), o) > 1) { ++g_cov[COV_LISTED_TWICE]; break; }
        }
    }
    for (size_t i = 0; i < s.obj.size(); ++i) {
        Object* o = s.obj[i];
        if (!o) continue;
        if (holder[i] && o->getContainedBy() && o->getContainedBy() != holder[i] &&
            op.code != OP_ENTER)
            ++g_cov[COV_MOVED_TO_FRONT];
        if (op.code == OP_UPDATE && o->getBodyModule()->getHealth() > health[i]) ++g_cov[COV_HEALED];
        if (holder[i] && o->isDestroyed() &&
            (op.code == OP_DAMAGE || op.code == OP_DESTROY_UNIT || op.code == OP_KILL_UNIT) &&
            i >= s.tunnels)
            ++g_cov[COV_DIED_INSIDE];
    }
    if (op.code == OP_GET_NEMESIS && s.nemesis) ++g_cov[COV_NEMESIS];
}

static bool flush_is_safe(Scene& s) {
    for (uint32_t p = 0; p < 3; ++p) {
        for (const Object* o : *s.tracker(p)->getContainedItemsList()) {
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

static void apply(Scene& s, const Op& op) {
    const auto tun = [&](uint32_t i) { return s.obj[i]; };
    const auto unit = [&](uint32_t i) { return s.obj[s.tunnels + i]; };
    s.orders = 0;
    switch (op.code) {
        case OP_ENTER:
            if (tun(op.b) && unit(op.a)) tun(op.b)->getContain()->addToContain(unit(op.a));
            break;
        case OP_EXIT:
            if (tun(op.b) && unit(op.a))
                tun(op.b)->getContain()->removeFromContain(unit(op.a), op.c ? TRUE : FALSE);
            break;
        case OP_KILL_TUNNEL:
            if (tun(op.a)) tun(op.a)->kill(DAMAGE_UNRESISTABLE, op.b ? DEATH_SUICIDED : DEATH_NORMAL);
            break;
        case OP_DESTROY_TUNNEL:
            if (tun(op.a)) TheGameLogic->destroyObject(tun(op.a));
            break;
        case OP_SELL:
            if (tun(op.a)) tun(op.a)->getContain()->onSelling();
            break;
        case OP_CAPTURE_TUNNEL:
            if (tun(op.a)) tun(op.a)->setTeam(s.team(op.b));
            break;
        case OP_UPDATE:
            if (tun(op.a) && !tun(op.a)->isDestroyed()) module_of(tun(op.a))->update();
            break;
        case OP_DAMAGE:
            if (s.obj[op.a]) {
                DamageInfo d;
                d.in.m_damageType = DAMAGE_EXPLOSION;
                d.in.m_deathType = DEATH_NORMAL;
                d.in.m_amount = static_cast<Real>(op.b);
                d.in.m_sourceID = op.c ? s.id.at(op.c - 1) : INVALID_ID;
                s.obj[op.a]->attemptDamage(&d);
            }
            break;
        case OP_DESTROY_UNIT:
            if (unit(op.a)) TheGameLogic->destroyObject(unit(op.a));
            break;
        case OP_KILL_UNIT:
            if (unit(op.a)) unit(op.a)->kill();
            break;
        case OP_REMOVE_ALL:
            if (tun(op.a)) tun(op.a)->getContain()->removeAllContained(op.b ? TRUE : FALSE);
            break;
        case OP_KILL_ALL:
            if (tun(op.a)) tun(op.a)->getContain()->killAllContained();
            break;
        case OP_HARM:
            if (tun(op.a)) {
                DamageInfo d;
                d.in.m_damageType = DAMAGE_EXPLOSION;
                d.in.m_deathType = DEATH_NORMAL;
                d.in.m_amount = static_cast<Real>(op.b);
                d.in.m_sourceID = op.c ? s.id.at(op.c - 1) : INVALID_ID;
                tun(op.a)->getContain()->harmAndForceExitAllContained(&d);
            }
            break;
        case OP_ADVANCE:
            s.w->set_frame(TheGameLogic->getFrame() + op.a);
            break;
        case OP_FLUSH:
            if (flush_is_safe(s)) flush(s);
            break;
        case OP_UPDATE_NEMESIS:
            s.tracker(op.a)->updateNemesis(op.b ? s.obj[op.b - 1] : nullptr);
            break;
        case OP_GET_NEMESIS: {
            const Object* n = s.tracker(op.a)->getCurNemesis();
            s.nemesis = n ? static_cast<uint32_t>(n->getID()) : 0u;
            break;
        }
        case OP_STATUS:
            if (unit(op.a)) {
                const ObjectStatusTypes st = op.b == 0 ? OBJECT_STATUS_STEALTHED
                    : (op.b == 1 ? OBJECT_STATUS_DETECTED : OBJECT_STATUS_DISGUISED);
                if (op.c) unit(op.a)->setStatus(MAKE_OBJECT_STATUS_MASK(st));
                else unit(op.a)->clearStatus(MAKE_OBJECT_STATUS_MASK(st));
            }
            break;
        case OP_ORDER_EXIT:
        case OP_ORDER_IDLE:
            if (tun(op.a)) {
                for (Object* o : s.obj) {
                    if (o && o->getAI()) o->getAI()->friend_setLastCommandSource(CMD_FROM_PLAYER);
                }
                if (op.code == OP_ORDER_EXIT)
                    tun(op.a)->getContain()->orderAllPassengersToExit(CMD_FROM_SCRIPT, FALSE);
                else
                    tun(op.a)->getContain()->orderAllPassengersToIdle(CMD_FROM_SCRIPT);
                for (size_t k = 0; k < s.obj.size(); ++k) {
                    Object* o = s.obj[k];
                    if (o && o->getAI() && o->getAI()->getLastCommandSource() == CMD_FROM_SCRIPT) {
                        s.orders |= 1u << k;
                        // `aiExit` also registers the rider with the container; that is the
                        // unported AI's doing, so withdraw it.
                        tun(op.a)->getContain()->onObjectWantsToEnterOrExit(o, WANTS_NEITHER);
                    }
                }
                // The AI state machines are unported: both sides restart the stream here.
                InitRandom(op.c);
            }
            break;
        case OP_CAPTURE_UNIT:
            if (unit(op.a)) unit(op.a)->setTeam(s.team(op.b));
            break;
    }
}

static Op random_op(uint32_t tunnels, uint32_t units, uint32_t spawned) {
    static const uint32_t weight[OP_COUNT] = {
        22, 7, 4, 4, 4, 4, 14, 9, 3, 3, 3, 2, 3, 8, 5, 3, 4, 3, 2, 2, 2};
    uint32_t total = 0;
    for (uint32_t w : weight) total += w;
    uint32_t r = pick(total), code = 0;
    while (r >= weight[code]) r -= weight[code++];
    Op op{code, 0, 0, 0};
    switch (code) {
        case OP_ENTER: op.a = pick(units); op.b = pick(tunnels); break;
        case OP_EXIT: op.a = pick(units); op.b = pick(tunnels); op.c = pick(2); break;
        case OP_KILL_TUNNEL: op.a = pick(tunnels); op.b = pick(3) == 0; break;
        case OP_DESTROY_TUNNEL: case OP_SELL: case OP_UPDATE: case OP_KILL_ALL: op.a = pick(tunnels); break;
        case OP_CAPTURE_TUNNEL: op.a = pick(tunnels); op.b = pick(4); break;
        case OP_DAMAGE:
            op.a = pick(2) ? pick(tunnels) : tunnels + pick(units);
            op.b = 1u + pick(90);
            op.c = pick(5) == 0 ? 0u : 1u + pick(spawned);
            break;
        case OP_DESTROY_UNIT: case OP_KILL_UNIT: op.a = pick(units); break;
        case OP_REMOVE_ALL: op.a = pick(tunnels); op.b = pick(2); break;
        case OP_HARM:
            op.a = pick(tunnels);
            op.b = pick(3) == 0 ? 500u : 1u + pick(60);
            op.c = pick(3) == 0 ? 0u : 1u + pick(spawned);
            break;
        case OP_ADVANCE: {
            static const uint32_t step[8] = {1, 2, 3, 9, 29, 30, 91, 200};
            op.a = step[pick(8)];
            break;
        }
        case OP_UPDATE_NEMESIS: op.a = pick(3); op.b = pick(spawned + 1); break;
        case OP_GET_NEMESIS: op.a = pick(3); break;
        case OP_STATUS: op.a = pick(units); op.b = pick(3); op.c = pick(4) != 0; break;
        case OP_ORDER_EXIT: op.a = pick(tunnels); op.c = 1u + pick(1000000); break;
        case OP_ORDER_IDLE: op.a = pick(tunnels); op.c = 1u + pick(1000000); break;
        case OP_CAPTURE_UNIT: op.a = pick(units); op.b = pick(4); break;
        default: break;
    }
    return op;
}

// Vector 0 walks the hard cases in order.
static void scripted(Vec& v) {
    v.cap = 3;
    v.frame = 100;
    // Three tunnels of team A; units: infantry, hero, vehicle, aircraft, DestroyDie, AI.
    v.p = {3, 2, 1, 100, 100, 3, 1, 160, 100, 1, 1, 220, 100,
           6, U_INF, 1, 100, 140, 40, U_HERO, 1, 120, 140, 90, U_VEH, 1, 140, 140, 150,
           U_AIR, 1, 160, 140, 20, U_DD, 1, 180, 140, 10, U_AI, 1, 200, 140, 30};
    const uint32_t t = 3;
    v.ops = {
        {OP_ENTER, 0, 0, 0}, {OP_ENTER, 1, 1, 0}, {OP_ENTER, 2, 0, 0},
        {OP_ENTER, 3, 0, 0},                          // a fourth, an aircraft, over capacity
        {OP_ADVANCE, 3, 0, 0}, {OP_UPDATE, 0, 0, 0}, {OP_UPDATE, 1, 0, 0},
        {OP_KILL_TUNNEL, 0, 0, 0},                    // passengers move to the front tunnel
        {OP_ADVANCE, 9, 0, 0}, {OP_UPDATE, 2, 0, 0}, {OP_DESTROY_TUNNEL, 0, 0, 0}, {OP_FLUSH, 0, 0, 0},
        {OP_SELL, 1, 0, 0}, {OP_UPDATE, 2, 0, 0},
        {OP_ENTER, 4, 2, 0}, {OP_ENTER, 5, 2, 0}, {OP_ORDER_EXIT, 2, 0, 77}, {OP_ORDER_IDLE, 2, 0, 78},
        {OP_DAMAGE, 3 + 4, 90, 0},                    // DestroyDie passenger dies inside
        {OP_FLUSH, 0, 0, 0},
        {OP_DAMAGE, 2, 10, t + 5 + 1}, {OP_CAPTURE_UNIT, 5, 2, 0}, {OP_DAMAGE, 2, 10, t + 5 + 1},
        {OP_UPDATE, 2, 0, 0}, {OP_GET_NEMESIS, 1, 0, 0},
        {OP_HARM, 2, 30, 0}, {OP_ENTER, 0, 2, 0}, {OP_ENTER, 1, 2, 0},
        {OP_CAPTURE_TUNNEL, 2, 2, 0},                 // the last tunnel changes owner
        {OP_FLUSH, 0, 0, 0}, {OP_UPDATE, 2, 0, 0}, {OP_ENTER, 2, 2, 0}, {OP_KILL_ALL, 2, 0, 0},
    };
}

// Vector 1 sits on the time boundaries of healing and of the nemesis.
static void scripted_boundaries(Vec& v) {
    v.cap = 10;
    v.frame = 200;
    // One tunnel of team A (30 frames to full health); an enemy infantry and drone; two hurt
    // passengers. Objects: tunnel 0, infantry 1, drone 2, hero 3, infantry 4.
    v.p = {1, 2, 1, 100, 100,
           4, U_INF, 2, 100, 140, 0, U_DRONE, 2, 120, 140, 0, U_HERO, 1, 140, 140, 150,
           U_INF, 1, 160, 140, 80};
    v.ops = {
        {OP_ENTER, 2, 0, 0}, {OP_ENTER, 3, 0, 0},
        {OP_ADVANCE, 29, 0, 0}, {OP_UPDATE, 0, 0, 0}, {OP_ADVANCE, 1, 0, 0}, {OP_UPDATE, 0, 0, 0},
        {OP_DAMAGE, 0, 10, 3}, {OP_UPDATE, 0, 0, 0}, {OP_GET_NEMESIS, 1, 0, 0},
        {OP_DAMAGE, 0, 10, 2}, {OP_ADVANCE, 30, 0, 0}, {OP_UPDATE, 0, 0, 0}, {OP_GET_NEMESIS, 1, 0, 0},
        {OP_DAMAGE, 0, 10, 2}, {OP_ADVANCE, 29, 0, 0}, {OP_UPDATE, 0, 0, 0}, {OP_GET_NEMESIS, 1, 0, 0},
        {OP_ADVANCE, 120, 0, 0}, {OP_GET_NEMESIS, 1, 0, 0},
        {OP_ADVANCE, 1, 0, 0}, {OP_GET_NEMESIS, 1, 0, 0},
        {OP_UPDATE_NEMESIS, 1, 2, 0}, {OP_ADVANCE, 100, 0, 0}, {OP_UPDATE_NEMESIS, 1, 2, 0},
        {OP_ADVANCE, 100, 0, 0}, {OP_GET_NEMESIS, 1, 0, 0},
        {OP_STATUS, 0, 0, 1}, {OP_GET_NEMESIS, 1, 0, 0},                    // stealthed
        {OP_STATUS, 0, 2, 1}, {OP_UPDATE_NEMESIS, 1, 2, 0}, {OP_GET_NEMESIS, 1, 0, 0},  // disguised
        {OP_STATUS, 0, 2, 0}, {OP_STATUS, 0, 1, 1}, {OP_GET_NEMESIS, 1, 0, 0},          // detected
        {OP_STATUS, 0, 1, 0}, {OP_GET_NEMESIS, 1, 0, 0},
        {OP_STATUS, 0, 0, 0}, {OP_UPDATE_NEMESIS, 1, 2, 0}, {OP_UPDATE_NEMESIS, 1, 3, 0},
        {OP_UPDATE_NEMESIS, 1, 0, 0}, {OP_GET_NEMESIS, 1, 0, 0},
        {OP_KILL_UNIT, 0, 0, 0}, {OP_GET_NEMESIS, 1, 0, 0},
    };
}

static void generate(Vec& v, int i) {
    static const uint32_t caps[8] = {0, 1, 2, 3, 10, 10, 10, 10};
    v.cap = caps[pick(8)];
    v.frame = pick(8) == 0 ? pick(30) : 100 + pick(5000);
    v.seed = next_u32() & 0x7fffffff;
    if (i == 0) { scripted(v); return; }
    if (i == 1) { scripted_boundaries(v); return; }
    const uint32_t tunnels = 1u + pick(4);
    const uint32_t units = 1u + pick(7);
    v.p = {tunnels};
    const uint32_t home = 1u + pick(2);
    for (uint32_t k = 0; k < tunnels; ++k) {
        const uint32_t team = pick(5) == 0 ? pick(3) : home;
        v.p.insert(v.p.end(), {pick(kTunnelTypes), team, 80 + pick(300), 80 + pick(300)});
    }
    v.p.push_back(units);
    for (uint32_t k = 0; k < units; ++k) {
        const uint32_t team = pick(4) == 0 ? pick(3) : home;
        v.p.insert(v.p.end(), {pick(U_COUNT), team, 80 + pick(300), 80 + pick(300),
                               pick(2) ? 0u : 1u + pick(55)});
    }
    uint32_t spawned = tunnels + units;
    const uint32_t count = 12u + pick(36);
    for (uint32_t k = 0; k < count; ++k) v.ops.push_back(random_op(tunnels, units, spawned));
}

static void reset_trackers(oracle_wp51::World& w, Scene& s) {
    for (uint32_t i = 0; i < 3; ++i) {
        Player* p = s.team(i)->getControllingPlayer();
        deleteInstance(p->*access_member(Wp51PlayerTunnelTag()));
        p->*access_member(Wp51PlayerTunnelTag()) = newInstance(TunnelTracker);
    }
}

static void run(oracle_wp51::World& w, Vec& v) {
    Scene s;
    s.w = &w;
    reset_trackers(w, s);
    TheWritableGlobalData->m_maxTunnelCapacity = static_cast<Int>(v.cap);
    size_t at = 0;
    s.tunnels = v.p[at++];
    const auto place = [&](Object* o, uint32_t x, uint32_t y) {
        Coord3D pos = {static_cast<Real>(x), static_cast<Real>(y), 0.0f};
        o->setPosition(&pos);
        s.obj.push_back(o);
        s.id.push_back(o->getID());
    };
    for (uint32_t k = 0; k < s.tunnels; ++k, at += 4) {
        const std::string name = "P300_Tunnel" + std::to_string(v.p[at]);
        place(w.spawn_object(name.c_str(), s.team(v.p[at + 1])), v.p[at + 2], v.p[at + 3]);
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
        if (v.p[at] == U_TRANS) transports.push_back(s.obj.size() - 1);
    }
    // One passenger inside every transport, spawned after all units.
    for (size_t k : transports) {
        Object* carrier = s.obj[k];
        Object* rider = w.spawn_object("P300_Inf", carrier->getTeam());
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
    for (uint32_t p = 0; p < 3; ++p) v.final_trackers += save_hex(s.tracker(p)) + " ";
    uint32_t crc = 0;
    for (uint32_t i = 0; i < s.tunnels; ++i) {
        if (!s.obj[i]) continue;
        v.final_modules += save_hex(module_of(s.obj[i])) + " ";
        crc = crc * 31u + oracle_wp51::World::snapshot_crc(module_of(s.obj[i]));
    }
    v.crc = crc;

    // Tear down: unhook every passenger first so no stale pointer survives.
    for (uint32_t p = 0; p < 3; ++p) {
        ContainedItemsList list;
        s.tracker(p)->swapContainedItemsList(list);
    }
    for (Object* o : s.obj) {
        if (o && o->getContainedBy() && o->getContainedBy()->getContain()->isTunnelContain())
            o->onRemovedFrom(o->getContainedBy());
    }
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        if (!o->isDestroyed()) TheGameLogic->destroyObject(o);
    }
    w.flush_destroy_list();
    ThePartitionManager->update();
}

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_pkt300_tunnel <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    const int dump = std::getenv("PKT300_DUMP") ? std::atoi(std::getenv("PKT300_DUMP")) : -1;
    oracle_wp51::World w;
    w.load_ini(fixture_ini());
    w.init_with_neutral_player("FactionP300", nullptr);
    TheWritableGlobalData->m_unitDamagedThresh = 0.7f;
    TheWritableGlobalData->m_unitReallyDamagedThresh = 0.35f;
    const Int local = w.team_a->getControllingPlayer()->getPlayerIndex();
    Dict dict_a2;
    TheTeamFactory->initTeam("TeamA2", AsciiString::TheEmptyString, TRUE, &dict_a2);
    g_team_a2 = TheTeamFactory->findTeam("TeamA2");
    g_team_a2->setControllingPlayer(w.team_a->getControllingPlayer());

    const int kTotal = 1000;
    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    const char* root = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/";
    std::fprintf(f, "{\n\"packet\": \"PKT-300\",\n\"vector_count\": %d,\n\"local_player\": %d,\n", kTotal, local);
    std::fprintf(f, "\"source_sha256\": {\"contain\":\"%s\",\"tracker\":\"%s\"},\n",
                 oracle::sha256_file((std::string(root) + "GameLogic/Object/Contain/TunnelContain.cpp").c_str()).c_str(),
                 oracle::sha256_file((std::string(root) + "Common/RTS/TunnelTracker.cpp").c_str()).c_str());
    std::vector<Vec> vecs;
    for (int i = 0; i < kTotal; ++i) {
        Vec v;
        generate(v, i);
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
        const char* names[5] = {"trackers", "modules", "objects", "queries", "misc"};
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
        std::fprintf(f, "\",\"final_trackers\":\"%s\",\"final_modules\":%u,\"crc\":%u,\"rng\":%u}%s\n",
                     v.final_trackers.c_str(), fnv32(v.final_modules), v.crc, v.rng,
                     i + 1 < kTotal ? "," : "");
    }
    std::fprintf(f, "]\n}\n");
    std::fclose(f);
    if (std::rename(tmp.c_str(), out_path.c_str()) != 0) { std::perror("rename"); return 1; }

    uint32_t opcount[OP_COUNT] = {0};
    for (const Vec& v : vecs)
        for (const Op& o : v.ops) ++opcount[o.code];
    std::fprintf(stderr, "wrote %d vectors; ops:", kTotal);
    for (uint32_t c : opcount) std::fprintf(stderr, " %u", c);
    std::fprintf(stderr, "\n");
    for (int c = 0; c < COV_COUNT; ++c) std::fprintf(stderr, "%s=%u ", kCov[c], g_cov[c]);
    std::fprintf(stderr, "\n");
    return 0;
}
