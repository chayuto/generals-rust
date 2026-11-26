// Partition shroud (object side): drives real Objects created through
// ThingFactory::newObject, moved, contained, upgraded, captured, defected,
// spied on, status-toggled, killed, destroyed, and boundary-reset against
// PartitionManager::update and PartitionManager::crc / xfer.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_object_shroud.cpp --opt O0 -o /tmp/gen_pos_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_object_shroud.cpp --opt O2 -o /tmp/gen_pos_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_object_shroud.cpp --opt O2 --sanitize -o /tmp/gen_pos_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_pos_o2 docs/golden_vectors/PARTITION_OBJECT_SHROUD.json
//
// The local player is replaced by one with index 99 so that no cell reaches the
// display or radar.

#include "oracle_wp51_world.h"

#include "Common/GameUtility.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "Common/Upgrade.h"
#include "Common/XferCRC.h"
#include "GameLogic/Module/CollideModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

#include <algorithm>
#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

struct PosLocalTag { using type = Player* PlayerList::*; };
template struct MemberAccess<PosLocalTag, &PlayerList::m_local>;
inline Player* PlayerList::* access_member(PosLocalTag);

struct ObjLastLookTag { using type = SightingInfo* Object::*; };
template struct MemberAccess<ObjLastLookTag, &Object::m_partitionLastLook>;
inline SightingInfo* Object::* access_member(ObjLastLookTag);

struct ObjRevealAllLastLookTag { using type = SightingInfo* Object::*; };
template struct MemberAccess<ObjRevealAllLastLookTag, &Object::m_partitionRevealAllLastLook>;
inline SightingInfo* Object::* access_member(ObjRevealAllLastLookTag);

struct ObjLastShroudTag { using type = SightingInfo* Object::*; };
template struct MemberAccess<ObjLastShroudTag, &Object::m_partitionLastShroud>;
inline SightingInfo* Object::* access_member(ObjLastShroudTag);

struct ObjVisionSpiedByTag { using type = Int (Object::*)[MAX_PLAYER_COUNT]; };
template struct MemberAccess<ObjVisionSpiedByTag, &Object::m_visionSpiedBy>;
inline Int (Object::* access_member(ObjVisionSpiedByTag))[MAX_PLAYER_COUNT];

struct ObjVisionSpiedMaskTag { using type = PlayerMaskType Object::*; };
template struct MemberAccess<ObjVisionSpiedMaskTag, &Object::m_visionSpiedMask>;
inline PlayerMaskType Object::* access_member(ObjVisionSpiedMaskTag);

struct ObjVisionRangeTag { using type = Real Object::*; };
template struct MemberAccess<ObjVisionRangeTag, &Object::m_visionRange>;
inline Real Object::* access_member(ObjVisionRangeTag);

struct ObjShroudClearingRangeTag { using type = Real Object::*; };
template struct MemberAccess<ObjShroudClearingRangeTag, &Object::m_shroudClearingRange>;
inline Real Object::* access_member(ObjShroudClearingRangeTag);

struct ObjShroudRangeTag { using type = Real Object::*; };
template struct MemberAccess<ObjShroudRangeTag, &Object::m_shroudRange>;
inline Real Object::* access_member(ObjShroudRangeTag);

static const int kVectors = 500;
static const int kReadable = 10;

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }

static uint32_t g_state = 0x4B1D0C55u;
static uint32_t next_u32() {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state;
}
static uint32_t pick(uint32_t n) { return next_u32() % n; }
static Real spread(Real lo, Real hi) {
    return lo + (static_cast<Real>(next_u32() % 100000u) / 100000.0f) * (hi - lo);
}

static void die(const char* what) {
    std::fprintf(stderr, "harness: %s\n", what);
    std::exit(1);
}

static std::string digest_bytes(const std::vector<uint8_t>& bytes) {
    uint64_t h = 0xcbf29ce484222325ull;
    for (uint8_t c : bytes) {
        h ^= c;
        h *= 0x100000001b3ull;
    }
    char out[40];
    std::snprintf(out, sizeof(out), "%zu:%016llx", bytes.size(), static_cast<unsigned long long>(h));
    return out;
}

static std::vector<uint8_t> object_vision_slice(Object* obj) {
    oracle::MemoryXfer xs(XFER_SAVE);
    xs.open("obj_vision");
    xs.xferSnapshot(obj->*access_member(ObjLastLookTag()));
    xs.xferSnapshot(obj->*access_member(ObjRevealAllLastLookTag()));
    xs.xferSnapshot(obj->*access_member(ObjLastShroudTag()));
    xs.xferUser(obj->*access_member(ObjVisionSpiedByTag()), sizeof(Int) * MAX_PLAYER_COUNT);
    xs.xferUser(&(obj->*access_member(ObjVisionSpiedMaskTag())), sizeof(PlayerMaskType));
    xs.xferReal(&(obj->*access_member(ObjVisionRangeTag())));
    xs.xferReal(&(obj->*access_member(ObjShroudClearingRangeTag())));
    xs.xferReal(&(obj->*access_member(ObjShroudRangeTag())));
    xs.close();
    return xs.bytes;
}

static void verify_slice_in_object_xfer(Object* obj) {
    const std::vector<uint8_t> slice = object_vision_slice(obj);
    oracle::MemoryXfer full(XFER_SAVE);
    full.open("obj_full");
    oracle::call_snapshot_xfer(obj, &full);
    full.close();
    auto it = std::search(full.bytes.begin(), full.bytes.end(), slice.begin(), slice.end());
    if (it == full.bytes.end()) {
        die("vision/shroud slice missing from Object::xfer");
    }
}

struct Grid { Real cs, lox, loy, hix, hiy; };

static Grid make_grid(bool tiny) {
    const Real sizes[5] = {40.0f, 20.0f, 10.0f, 25.0f, 33.333f};
    const Real lows[4] = {0.0f, -100.0f, -25.0f, 50.0f};
    Grid g;
    g.cs = sizes[pick(5)];
    g.lox = lows[pick(4)];
    g.loy = lows[pick(4)];
    const int nx = tiny ? 4 + pick(4) : 6 + pick(10);
    const int ny = tiny ? 4 + pick(4) : 6 + pick(10);
    g.hix = g.lox + nx * g.cs;
    g.hiy = g.loy + ny * g.cs;
    return g;
}

static std::string grid_json(const Grid& g) {
    std::ostringstream ss;
    ss << bits(g.cs) << ',' << bits(g.lox) << ',' << bits(g.loy) << ',' << bits(g.hix) << ','
       << bits(g.hiy);
    return ss.str();
}

static void set_grid(oracle_wp51::OracleTerrainLogic* terrain, const Grid& g) {
    TheWritableGlobalData->m_partitionCellSize = g.cs;
    terrain->extent.lo = {g.lox, g.loy, 0.0f};
    terrain->extent.hi = {g.hix, g.hiy, 100.0f};
    ThePartitionManager->reset();
    ThePartitionManager->init();
}

static const char* kTemplates[9] = {
    "PosUnitInherit",
    "PosUnitExplicit",
    "PosUnitZero",
    "PosRevealAll",
    "PosRevealEnemy",
    "PosGarrison",
    "PosTransport",
    "PosShroudCrate",
    "PosSpLauncher",
};

struct TrackedObj {
    Object* ptr = nullptr;
    int tmpl_kind = 0;
    bool deleted = false;
};

static std::string state_json(const std::vector<TrackedObj>& objs, bool verify_full_xfer) {
    XferCRC crc;
    crc.open("crc");
    ThePartitionManager->crc(&crc);
    crc.close();

    oracle::MemoryXfer save(XFER_SAVE);
    save.open("save");
    ThePartitionManager->xfer(&save);
    save.close();

    std::ostringstream ss;
    ss << crc.getCRC() << ",\"" << digest_bytes(save.bytes) << "\",[";
    for (size_t i = 0; i < objs.size(); ++i) {
        if (i) ss << ',';
        if (objs[i].deleted || !objs[i].ptr) {
            ss << "\"\"";
        } else {
            if (verify_full_xfer) {
                verify_slice_in_object_xfer(objs[i].ptr);
            }
            ss << "\"" << digest_bytes(object_vision_slice(objs[i].ptr)) << "\"";
        }
    }
    ss << ']';
    return ss.str();
}

static Real make_coord(Real lo, Real hi, Real cs) {
    switch (pick(6)) {
    case 0: return 0.0f;
    case 1: return lo + static_cast<Real>(pick(4)) * cs;
    case 2: return hi - static_cast<Real>(pick(3)) * cs;
    case 3: return pick(2) ? lo - cs : hi + cs;
    default: return spread(lo + 1.0f, hi - 1.0f);
    }
}

static Real make_range(Real cs) {
    const Real vals[7] = {0.0f, -10.0f, 0.5f * cs, 1.5f * cs, 2.5f * cs, 4.0f * cs, 6.0f * cs};
    return pick(3) ? vals[pick(7)] : spread(0.0f, 5.0f * cs);
}

static const UpgradeTemplate* g_upg_shroud = nullptr;
static Team* g_def_teams[MAX_PLAYER_COUNT] = {};
static Team* g_alt_teams[MAX_PLAYER_COUNT] = {};

static uint32_t g_cov_ops[24] = {};

static std::string run_vector(
    oracle_wp51::OracleTerrainLogic* terrain,
    oracle_wp51::World& w,
    bool readable
) {
    Grid grid = make_grid(readable);
    set_grid(terrain, grid);

    const int players = (pick(10) == 0) ? (7 + pick(10)) : (3 + pick(4));
    ThePlayerList->*access_member(Wp51PlayerListCountTag()) = players;
    for (int p = 0; p < players; ++p) {
        Player* pl = ThePlayerList->getNthPlayer(p);
        pl->init(ThePlayerTemplateStore->findPlayerTemplate(NAMEKEY("FactionPOS")));
        if (!pl->getResourceGatheringManager()) {
            pl->*access_member(Wp51PlayerRgmTag()) = newInstance(ResourceGatheringManager);
        }
        if (!pl->getTunnelSystem()) {
            pl->*access_member(Wp51PlayerTunnelTag()) = newInstance(TunnelTracker);
        }
        g_def_teams[p]->setControllingPlayer(pl);
        g_alt_teams[p]->setControllingPlayer(pl);
        pl->*access_member(Wp51PlayerDefaultTeamTag()) = g_def_teams[p];
    }

    std::vector<int> rels(players * players, 2);
    for (int i = 0; i < players; ++i) {
        Player* pi = ThePlayerList->getNthPlayer(i);
        pi->setPlayerRelationship(pi, ALLIES);
        for (int j = 0; j < players; ++j) {
            if (i == j) continue;
            Player* pj = ThePlayerList->getNthPlayer(j);
            const uint32_t r = pick(3);
            Relationship rel = (r == 0) ? ENEMIES : ((r == 1) ? NEUTRAL : ALLIES);
            pi->setPlayerRelationship(pj, rel);
            rels[i * players + j] = static_cast<int>(rel);
        }
    }

    const UnsignedInt persists[5] = {0u, 1u, 5u, 30u, 0xFFFFFFF0u};
    const UnsignedInt persist = persists[pick(5)];
    TheWritableGlobalData->m_unlookPersistDuration = persist;
    UnsignedInt frame = (pick(8) == 0) ? (0xFFFFFFC0u + pick(40)) : (1u + pick(80));
    w.set_frame(frame);

    std::ostringstream ss;
    ss << "{\"g\":[" << grid_json(grid) << "],\"players\":" << players << ",\"persist\":"
       << persist << ",\"frame\":" << frame << ",\"rels\":[";
    for (size_t i = 0; i < rels.size(); ++i) {
        if (i) ss << ',';
        ss << rels[i];
    }
    ss << "],\"s\":[";

    std::vector<TrackedObj> objs;
    bool first_row = true;
    auto emit_row = [&](const std::string& args_str) {
        ss << (first_row ? "" : ",") << "[" << args_str << ","
           << state_json(objs, readable) << "]";
        first_row = false;
    };

    auto pick_team = [&]() -> int { return static_cast<int>(pick(2 * players)); };
    auto team_ptr = [&](int tidx) -> Team* {
        return (tidx < players) ? g_def_teams[tidx] : g_alt_teams[tidx - players];
    };

    auto pick_live = [&](int want_tmpl = -1) -> int {
        std::vector<int> cand;
        for (size_t i = 0; i < objs.size(); ++i) {
            if (objs[i].deleted || !objs[i].ptr) continue;
            if (want_tmpl >= 0 && objs[i].tmpl_kind != want_tmpl) continue;
            cand.push_back(static_cast<int>(i));
        }
        if (cand.empty()) return -1;
        return cand[pick(static_cast<uint32_t>(cand.size()))];
    };

    auto spawn_one = [&](int tmpl_idx, int team_idx, Coord3D pos, bool under_const) {
        const ThingTemplate* tt = TheThingFactory->findTemplate(kTemplates[tmpl_idx]);
        if (!tt) die("missing template");
        ObjectStatusMaskType st;
        if (under_const) st.set(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
        Object* o = TheThingFactory->newObject(tt, team_ptr(team_idx), st);
        o->setPosition(&pos);
        objs.push_back(TrackedObj{o, tmpl_idx, false});
        ++g_cov_ops[0];
        std::ostringstream a;
        a << "0," << tmpl_idx << ',' << team_idx << ',' << bits(pos.x) << ',' << bits(pos.y)
          << ',' << bits(pos.z) << ',' << (under_const ? 1 : 0);
        emit_row(a.str());
    };

    // Seed initial objects so most operations have live targets.
    for (int k = 0; k < 4; ++k) {
        const int tmpl = static_cast<int>(pick(9));
        const int tidx = pick_team();
        const Coord3D pos = {
            make_coord(grid.lox, grid.hix, grid.cs),
            make_coord(grid.loy, grid.hiy, grid.cs),
            0.0f
        };
        spawn_one(tmpl, tidx, pos, pick(5) == 0);
    }

    const int ops = 18 + pick(24);
    for (int step = 0; step < ops; ++step) {
        const uint32_t choice = pick(40);
        if (choice < 4 && objs.size() < 16) {
            const int tmpl = static_cast<int>(pick(9));
            const int tidx = pick_team();
            Coord3D pos = {0.0f, 0.0f, 0.0f};
            if (pick(5) != 0) {
                pos.x = make_coord(grid.lox, grid.hix, grid.cs);
                pos.y = make_coord(grid.loy, grid.hiy, grid.cs);
            }
            spawn_one(tmpl, tidx, pos, pick(4) == 0);
        } else if (choice < 8) {
            const int idx = pick_live();
            if (idx < 0) continue;
            const Coord3D pos = {
                make_coord(grid.lox, grid.hix, grid.cs),
                make_coord(grid.loy, grid.hiy, grid.cs),
                0.0f
            };
            objs[idx].ptr->setPosition(&pos);
            ++g_cov_ops[1];
            std::ostringstream a;
            a << "1," << idx << ',' << bits(pos.x) << ',' << bits(pos.y) << ',' << bits(pos.z);
            emit_row(a.str());
        } else if (choice == 8) {
            const int idx = pick_live();
            if (idx < 0) continue;
            const Real r = make_range(grid.cs);
            objs[idx].ptr->setVisionRange(r);
            ++g_cov_ops[2];
            std::ostringstream a;
            a << "2," << idx << ',' << bits(r);
            emit_row(a.str());
        } else if (choice < 11) {
            const int idx = pick_live();
            if (idx < 0) continue;
            const Real r = pick(5) == 0
                ? objs[idx].ptr->getShroudClearingRange()
                : make_range(grid.cs);
            objs[idx].ptr->setShroudClearingRange(r);
            ++g_cov_ops[3];
            std::ostringstream a;
            a << "3," << idx << ',' << bits(r);
            emit_row(a.str());
        } else if (choice == 11) {
            const int idx = pick_live();
            if (idx < 0) continue;
            const Real r = make_range(grid.cs);
            objs[idx].ptr->setShroudRange(r);
            ++g_cov_ops[4];
            std::ostringstream a;
            a << "4," << idx << ',' << bits(r);
            emit_row(a.str());
        } else if (choice == 12) {
            const int idx = pick_live();
            if (idx < 0) continue;
            objs[idx].ptr->handlePartitionCellMaintenance();
            ++g_cov_ops[5];
            emit_row("5," + std::to_string(idx));
        } else if (choice == 13) {
            const int idx = pick_live(1);
            if (idx < 0) continue;
            objs[idx].ptr->giveUpgrade(g_upg_shroud);
            ++g_cov_ops[6];
            emit_row("6," + std::to_string(idx));
        } else if (choice < 16) {
            const int idx = pick_live();
            if (idx < 0) continue;
            const int spy = static_cast<int>(pick(players));
            const int* spied_by = objs[idx].ptr->*access_member(ObjVisionSpiedByTag());
            const Bool set = (spied_by[spy] == 0) ? TRUE : (pick(2) ? TRUE : FALSE);
            objs[idx].ptr->setVisionSpied(set, spy);
            ++g_cov_ops[7];
            std::ostringstream a;
            a << "7," << idx << ',' << (set ? 1 : 0) << ',' << spy;
            emit_row(a.str());
        } else if (choice < 19) {
            const int idx = pick_live();
            if (idx < 0) continue;
            const int sk = static_cast<int>(pick(4));
            const ObjectStatusTypes st = (sk == 0) ? OBJECT_STATUS_STEALTHED
                : (sk == 1) ? OBJECT_STATUS_DETECTED
                : (sk == 2) ? OBJECT_STATUS_DISGUISED
                            : OBJECT_STATUS_UNDER_CONSTRUCTION;
            const Bool on = (pick(2) != 0);
            objs[idx].ptr->setStatus(MAKE_OBJECT_STATUS_MASK(st), on);
            ++g_cov_ops[8];
            std::ostringstream a;
            a << "8," << idx << ',' << sk << ',' << (on ? 1 : 0);
            emit_row(a.str());
        } else if (choice < 21) {
            const int cidx = pick(2) ? pick_live(5) : pick_live(6);
            if (cidx < 0) continue;
            ContainModuleInterface* c = objs[cidx].ptr->getContain();
            if (!c || c->getContainCount() >= c->getContainMax()) continue;
            std::vector<int> riders;
            for (size_t i = 0; i < objs.size(); ++i) {
                if (static_cast<int>(i) == cidx || objs[i].deleted || !objs[i].ptr) continue;
                if (objs[i].ptr->isDestroyed() || objs[i].ptr->isEffectivelyDead()) continue;
                if (objs[i].ptr->getContainedBy() != nullptr) continue;
                if (objs[i].tmpl_kind == 5 || objs[i].tmpl_kind == 6 || objs[i].tmpl_kind == 8) continue;
                if (objs[i].tmpl_kind == 9) continue; // skip ViewObj
                riders.push_back(static_cast<int>(i));
            }
            if (riders.empty()) continue;
            const int ridx = riders[pick(static_cast<uint32_t>(riders.size()))];
            c->addToContain(objs[ridx].ptr);
            ++g_cov_ops[9];
            emit_row("9," + std::to_string(cidx) + "," + std::to_string(ridx));
        } else if (choice == 21) {
            std::vector<std::pair<int, int>> pairs;
            for (size_t i = 0; i < objs.size(); ++i) {
                if (objs[i].deleted || !objs[i].ptr) continue;
                Object* cb = objs[i].ptr->getContainedBy();
                if (!cb) continue;
                for (size_t j = 0; j < objs.size(); ++j) {
                    if (!objs[j].deleted && objs[j].ptr == cb) {
                        pairs.push_back({static_cast<int>(j), static_cast<int>(i)});
                        break;
                    }
                }
            }
            if (pairs.empty()) continue;
            const auto pr = pairs[pick(static_cast<uint32_t>(pairs.size()))];
            if (ContainModuleInterface* c = objs[pr.first].ptr->getContain()) {
                c->removeFromContain(objs[pr.second].ptr, FALSE);
                ++g_cov_ops[10];
                emit_row("10," + std::to_string(pr.first) + "," + std::to_string(pr.second));
            }
        } else if (choice == 22) {
            const int idx = pick_live();
            if (idx < 0 || objs[idx].ptr->getContainedBy()) continue;
            const int tidx = pick_team();
            objs[idx].ptr->setTeam(team_ptr(tidx));
            ++g_cov_ops[11];
            emit_row("11," + std::to_string(idx) + "," + std::to_string(tidx));
        } else if (choice == 23) {
            const int idx = pick_live();
            if (idx < 0) continue;
            const int tidx = pick_team();
            const UnsignedInt det = pick(2) ? 0u : (10u + pick(40));
            objs[idx].ptr->defect(team_ptr(tidx), det);
            ++g_cov_ops[12];
            emit_row("12," + std::to_string(idx) + "," + std::to_string(tidx) + "," + std::to_string(det));
        } else if (choice == 24) {
            const int tidx = pick_team();
            const int np = static_cast<int>(pick(players));
            team_ptr(tidx)->setControllingPlayer(ThePlayerList->getNthPlayer(np));
            ++g_cov_ops[13];
            emit_row("13," + std::to_string(tidx) + "," + std::to_string(np));
        } else if (choice == 25) {
            const int idx = pick_live();
            if (idx < 0 || objs[idx].ptr->isEffectivelyDead()) continue;
            objs[idx].ptr->kill(DAMAGE_UNRESISTABLE, DEATH_NORMAL);
            ++g_cov_ops[14];
            emit_row("14," + std::to_string(idx));
        } else if (choice == 26) {
            const int idx = pick_live();
            if (idx < 0 || objs[idx].ptr->isDestroyed()) continue;
            TheGameLogic->destroyObject(objs[idx].ptr);
            ++g_cov_ops[15];
            emit_row("15," + std::to_string(idx));
        } else if (choice == 27) {
            bool gone[32] = {};
            for (size_t i = 0; i < objs.size() && i < 32; ++i) {
                gone[i] = !objs[i].deleted && objs[i].ptr && objs[i].ptr->isDestroyed();
            }
            TheGameLogic->processDestroyList();
            for (size_t i = 0; i < objs.size() && i < 32; ++i) {
                if (gone[i]) {
                    objs[i].deleted = true;
                    objs[i].ptr = nullptr;
                }
            }
            ++g_cov_ops[16];
            emit_row("16");
        } else if (choice < 30) {
            const UnsignedInt deltas[5] = {1u, 2u, 6u, 31u, 45u};
            frame += deltas[pick(5)];
            w.set_frame(frame);
            ++g_cov_ops[17];
            emit_row("17," + std::to_string(frame));
        } else if (choice < 35) {
            ThePartitionManager->update();
            ++g_cov_ops[18];
            emit_row("18");
        } else if (choice == 35) {
            const int crate_idx = pick_live(7);
            const int picker_idx = pick_live(0);
            if (crate_idx < 0 || picker_idx < 0) continue;
            if (objs[crate_idx].ptr->isDestroyed() || objs[crate_idx].ptr->isEffectivelyDead()) continue;
            if (objs[picker_idx].ptr->isDestroyed() || objs[picker_idx].ptr->isEffectivelyDead()) continue;
            CollideModuleInterface* col = nullptr;
            for (BehaviorModule** b = objs[crate_idx].ptr->getBehaviorModules(); *b; ++b) {
                if ((col = (*b)->getCollide()) != nullptr) break;
            }
            if (!col) continue;
            const Coord3D p = *objs[crate_idx].ptr->getPosition();
            const Coord3D n = {0.0f, 0.0f, 1.0f};
            col->onCollide(objs[picker_idx].ptr, &p, &n);
            ++g_cov_ops[19];
            emit_row("19," + std::to_string(crate_idx) + "," + std::to_string(picker_idx));
        } else if (choice == 36 && objs.size() < 16) {
            const int sp_idx = pick_live(8);
            if (sp_idx < 0 || objs[sp_idx].ptr->isDisabled() || objs[sp_idx].ptr->isDestroyed()) continue;
            SpecialPowerModuleInterface* sp = nullptr;
            for (BehaviorModule** b = objs[sp_idx].ptr->getBehaviorModules(); *b; ++b) {
                if ((sp = (*b)->getSpecialPower()) != nullptr) break;
            }
            if (!sp || !sp->isReady()) continue;
            const Coord3D loc = {
                make_coord(grid.lox, grid.hix, grid.cs),
                make_coord(grid.loy, grid.hiy, grid.cs),
                0.0f
            };
            Object* prev_head = TheGameLogic->getFirstObject();
            sp->doSpecialPowerAtLocation(&loc, 0.0f, 0);
            Object* new_head = TheGameLogic->getFirstObject();
            if (new_head != prev_head && new_head) {
                objs.push_back(TrackedObj{new_head, 9, false});
            }
            ++g_cov_ops[20];
            std::ostringstream a;
            a << "20," << sp_idx << ',' << bits(loc.x) << ',' << bits(loc.y) << ',' << bits(loc.z);
            emit_row(a.str());
        } else if (choice == 37) {
            // Full 8-step TerrainLogic::setActiveBoundary sequence.
            ShroudStatusStoreRestore store;
            ThePartitionManager->processEntirePendingUndoShroudRevealQueue();
            ThePartitionManager->storeFoggedCells(store, TRUE);
            for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
                o->friend_prepareForMapBoundaryAdjust();
            }
            ThePartitionManager->storeFoggedCells(store, FALSE);
            Grid next_g = make_grid(readable);
            next_g.cs = grid.cs;
            grid = next_g;
            set_grid(terrain, grid);
            ThePartitionManager->restoreFoggedCells(store, FALSE);
            for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
                o->friend_notifyOfNewMapBoundary();
            }
            ThePartitionManager->restoreFoggedCells(store, TRUE);
            ++g_cov_ops[21];
            emit_row("21," + grid_json(grid));
        } else {
            const int mk = static_cast<int>(pick(4));
            const int p = static_cast<int>(pick(players));
            switch (mk) {
            case 0: ThePartitionManager->revealMapForPlayer(p); break;
            case 1: ThePartitionManager->revealMapForPlayerPermanently(p); break;
            case 2: ThePartitionManager->undoRevealMapForPlayerPermanently(p); break;
            default: ThePartitionManager->shroudMapForPlayer(p); break;
            }
            ++g_cov_ops[22];
            emit_row("22," + std::to_string(mk) + "," + std::to_string(p));
        }
    }

    frame += 35u;
    w.set_frame(frame);
    emit_row("17," + std::to_string(frame));
    ThePartitionManager->update();
    emit_row("18");
    ss << "]}";

    // Clean up all live objects before next vector.
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        TheGameLogic->destroyObject(o);
    }
    TheGameLogic->processDestroyList();
    return ss.str();
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_partition_object_shroud <out.json>\n");
        return 2;
    }
    const std::string out_path = argv[1];

    oracle_wp51::World w;
    w.load_ini(
        "PlayerTemplate FactionPOS\n  Side = America\n  PlayableSide = Yes\nEnd\n"
        "Upgrade Upg_Shroud\nEnd\n"
        "ObjectCreationList OCL_Empty\nEnd\n"
        "SpecialPower Sp_View\n"
        "  Enum = SPECIAL_SPY_SATELLITE\n"
        "  ReloadTime = 1\n"
        "  ViewObjectRange = 120\n"
        "  ViewObjectDuration = 15\n"
        "End\n"
        "Object PosViewObj\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = DeletionUpdate ModuleTag_Del\n    MinLifetime = 5\n    MaxLifetime = 5\n  End\n"
        "End\n"
        "Object PosUnitInherit\n"
        "  KindOf = INFANTRY\n"
        "  TransportSlotCount = 1\n"
        "  VisionRange = 90\n"
        "  ShroudClearingRange = -1\n"
        "  Geometry = SPHERE\n  GeometryMajorRadius = 10\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "End\n"
        "Object PosUnitExplicit\n"
        "  KindOf = VEHICLE\n"
        "  TransportSlotCount = 1\n"
        "  VisionRange = 90\n"
        "  ShroudClearingRange = 140\n"
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 15\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = ActiveShroudUpgrade ModuleTag_Upg\n"
        "    TriggeredBy = Upg_Shroud\n"
        "    NewShroudRange = 110\n"
        "  End\n"
        "End\n"
        "Object PosUnitZero\n"
        "  KindOf = INFANTRY\n"
        "  TransportSlotCount = 1\n"
        "  VisionRange = 80\n"
        "  ShroudClearingRange = 0\n"
        "  ShroudRevealToAllRange = 55\n"
        "  Geometry = SPHERE\n  GeometryMajorRadius = 10\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "End\n"
        "Object PosRevealAll\n"
        "  KindOf = INFANTRY REVEAL_TO_ALL\n"
        "  TransportSlotCount = 1\n"
        "  VisionRange = 75\n"
        "  ShroudClearingRange = 75\n"
        "  Geometry = SPHERE\n  GeometryMajorRadius = 10\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "End\n"
        "Object PosRevealEnemy\n"
        "  KindOf = INFANTRY\n"
        "  TransportSlotCount = 1\n"
        "  VisionRange = 60\n"
        "  ShroudClearingRange = 85\n"
        "  ShroudRevealToAllRange = 50\n"
        "  Geometry = SPHERE\n  GeometryMajorRadius = 12\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "End\n"
        "Object PosGarrison\n"
        "  KindOf = STRUCTURE\n"
        "  VisionRange = 70\n"
        "  ShroudClearingRange = 70\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 300\n    InitialHealth = 300\n  End\n"
        "  Behavior = GarrisonContain ModuleTag_Contain\n"
        "    ContainMax = 4\n"
        "    EnclosingContainer = Yes\n"
        "  End\n"
        "End\n"
        "Object PosTransport\n"
        "  KindOf = VEHICLE\n"
        "  VisionRange = 70\n"
        "  ShroudClearingRange = 70\n"
        "  Geometry = SPHERE\n  GeometryMajorRadius = 15\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "  Behavior = TransportContain ModuleTag_Contain\n"
        "    ContainMax = 4\n"
        "  End\n"
        "End\n"
        "Object PosShroudCrate\n"
        "  KindOf = CRATE\n"
        "  VisionRange = 50\n"
        "  ShroudClearingRange = 50\n"
        "  Geometry = SPHERE\n  GeometryMajorRadius = 8\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 50\n    InitialHealth = 50\n  End\n"
        "  Behavior = ShroudCrateCollide ModuleTag_Collide\n  End\n"
        "End\n"
        "Object PosSpLauncher\n"
        "  KindOf = STRUCTURE\n"
        "  VisionRange = 60\n"
        "  ShroudClearingRange = 60\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 15\n  GeometryMinorRadius = 15\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "  Behavior = OCLSpecialPower ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_View\n"
        "    OCL = OCL_Empty\n"
        "    CreateLocation = CREATE_AT_LOCATION\n"
        "  End\n"
        "End\n"
    );

    TheWritableGlobalData->m_specialPowerViewObjectName = "PosViewObj";
    g_upg_shroud = TheUpgradeCenter->findUpgrade("Upg_Shroud");
    if (!g_upg_shroud) die("missing upgrade");

    // Replace the local player so shroud cells never touch display or radar.
    Player* nobody = NEW Player(99);
    ThePlayerList->*access_member(PosLocalTag()) = nobody;
    if (rts::getObservedOrLocalPlayer()->getPlayerIndex() != 99) die("local player");

    for (int p = 0; p < MAX_PLAYER_COUNT; ++p) {
        Dict d1, d2;
        char n1[32], n2[32];
        std::snprintf(n1, sizeof(n1), "PosDefTeam_%d", p);
        std::snprintf(n2, sizeof(n2), "PosAltTeam_%d", p);
        TheTeamFactory->initTeam(n1, AsciiString::TheEmptyString, TRUE, &d1);
        TheTeamFactory->initTeam(n2, AsciiString::TheEmptyString, TRUE, &d2);
        g_def_teams[p] = TheTeamFactory->findTeam(n1);
        g_alt_teams[p] = TheTeamFactory->findTeam(n2);
        if (!g_def_teams[p] || !g_alt_teams[p]) die("missing team");
    }

    auto* terrain = static_cast<oracle_wp51::OracleTerrainLogic*>(TheTerrainLogic);
    std::vector<std::string> vectors;
    for (int i = 0; i < kVectors; ++i) {
        vectors.push_back(run_vector(terrain, w, i < kReadable));
    }

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    const std::string sha_pm = oracle::sha256_file(
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/PartitionManager.cpp");
    const std::string sha_obj = oracle::sha256_file(
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Object.cpp");
    std::fprintf(f, "{\n\"packet\": \"PARTITION_OBJECT_SHROUD\",\n\"vector_count\": %d,\n", kVectors);
    std::fprintf(f, "\"sha256_PartitionManager\": \"%s\",\n", sha_pm.c_str());
    std::fprintf(f, "\"sha256_Object\": \"%s\",\n\"vectors\": [\n", sha_obj.c_str());
    for (size_t i = 0; i < vectors.size(); ++i) {
        std::fprintf(f, "%s%s\n", vectors[i].c_str(), i + 1 < vectors.size() ? "," : "");
    }
    std::fprintf(f, "]\n}\n");
    std::fclose(f);
    if (std::rename(tmp.c_str(), out_path.c_str())) { std::perror("rename"); return 1; }

    std::fprintf(stderr, "coverage:");
    for (int i = 0; i < 23; ++i) std::fprintf(stderr, " op%d=%u", i, g_cov_ops[i]);
    std::fprintf(stderr, "\n");
    return 0;
}
