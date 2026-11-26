// C++ Golden Vector Generator for PKT-372 (SpyVisionUpdate) & PKT-323 (DynamicShroudClearingRangeUpdate).
// Also verifies wiring from SpyVisionSpecialPower (PKT-245) and SabotageInternetCenterCrateCollide (PKT-234)
// and integration with PartitionManager Stage 4a/4b shroud cells.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_shroud_updates.cpp --opt O0 -o /tmp/gen_su_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_shroud_updates.cpp --opt O2 -o /tmp/gen_su_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_shroud_updates.cpp --opt O2 --sanitize -o /tmp/gen_su_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_su_o2 docs/golden_vectors/SHROUD_UPDATES.json

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
#include "GameLogic/Module/DynamicShroudClearingRangeUpdate.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Module/SpyVisionUpdate.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

#include <algorithm>
#include <climits>
#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

struct SuLocalTag { using type = Player* PlayerList::*; };
template struct MemberAccess<SuLocalTag, &PlayerList::m_local>;
inline Player* PlayerList::* access_member(SuLocalTag);

struct SuSleepyTag { using type = std::vector<UpdateModulePtr> GameLogic::*; };
template struct MemberAccess<SuSleepyTag, &GameLogic::m_sleepyUpdates>;
inline std::vector<UpdateModulePtr> GameLogic::* access_member(SuSleepyTag);

struct SuCurUpdateTag { using type = UpdateModulePtr GameLogic::*; };
template struct MemberAccess<SuCurUpdateTag, &GameLogic::m_curUpdateModule>;
inline UpdateModulePtr GameLogic::* access_member(SuCurUpdateTag);

struct ObjVisionSpiedByTag { using type = Int (Object::*)[MAX_PLAYER_COUNT]; };
template struct MemberAccess<ObjVisionSpiedByTag, &Object::m_visionSpiedBy>;
inline Int (Object::* access_member(ObjVisionSpiedByTag))[MAX_PLAYER_COUNT];

struct ObjVisionSpiedMaskTag { using type = PlayerMaskType Object::*; };
template struct MemberAccess<ObjVisionSpiedMaskTag, &Object::m_visionSpiedMask>;
inline PlayerMaskType Object::* access_member(ObjVisionSpiedMaskTag);

static uint32_t bits(Real v) {
    uint32_t b;
    std::memcpy(&b, &v, 4);
    return b;
}

static uint32_t g_state = 0x91E4A73Bu;
static uint32_t next_u32() {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state;
}
static uint32_t pick(uint32_t n) { return next_u32() % n; }

static void die(const char* what) {
    std::fprintf(stderr, "gen_shroud_updates: %s\n", what);
    std::exit(1);
}

static void rebalance_child(std::vector<UpdateModulePtr>& heap, Int i) {
    const Int sz = static_cast<Int>(heap.size());
    while (true) {
        Int child = 2 * i + 1;
        if (child >= sz) break;
        if (child + 1 < sz && heap[child + 1]->friend_getPriority() < heap[child]->friend_getPriority()) {
            child = child + 1;
        }
        if (!(heap[child]->friend_getPriority() < heap[i]->friend_getPriority())) {
            break;
        }
        UpdateModulePtr a = heap[child];
        UpdateModulePtr b = heap[i];
        heap[i] = a;
        heap[child] = b;
        heap[i]->friend_setIndexInLogic(i);
        heap[child]->friend_setIndexInLogic(child);
        i = child;
    }
}

static void step_sleepy_updates() {
    auto& heap = TheGameLogic->*access_member(SuSleepyTag());
    auto& cur = TheGameLogic->*access_member(SuCurUpdateTag());
    const UnsignedInt now = TheGameLogic->getFrame();
    int guard = 0;
    while (!heap.empty()) {
        UpdateModulePtr u = heap.front();
        if (u->friend_getNextCallFrame() > now) break;
        if (++guard > 2048) die("sleepy update guard exceeded");

        UpdateSleepTime sleepLen = UPDATE_SLEEP_NONE;
        const DisabledMaskType dis = u->friend_getObject()->getDisabledFlags();
        if (u->getDisabledTypesToProcess().testForAll(dis)) {
            cur = u;
            sleepLen = u->update();
            if (sleepLen < 1) sleepLen = UPDATE_SLEEP_NONE;
            cur = nullptr;
        }
        if (u->friend_getIndexInLogic() >= 0) {
            u->friend_setNextCallFrame(now + static_cast<UnsignedInt>(sleepLen));
            rebalance_child(heap, 0);
        }
    }
}

static void step_frame(oracle_wp51::World& w, UnsignedInt& frame) {
    ++frame;
    w.set_frame(frame);
    step_sleepy_updates();
    ThePartitionManager->update();
    TheGameLogic->processDestroyList();
}

static uint32_t pm_crc() {
    XferCRC xc;
    xc.open("crc");
    ThePartitionManager->crc(&xc);
    xc.close();
    return xc.getCRC();
}

static SpyVisionUpdate* find_sv(Object* obj) {
    if (!obj) return nullptr;
    for (BehaviorModule** b = obj->getBehaviorModules(); *b; ++b) {
        if (SpyVisionUpdate* sv = (*b)->getSpyVisionUpdate()) return sv;
    }
    return nullptr;
}

static UpdateModule* find_update_by_name(Object* obj, const char* name) {
    if (!obj) return nullptr;
    return obj->findUpdateModule(TheNameKeyGenerator->nameToKey(name));
}

static SpecialPowerModuleInterface* find_sp(Object* obj) {
    if (!obj) return nullptr;
    for (BehaviorModule** b = obj->getBehaviorModules(); *b; ++b) {
        if (SpecialPowerModuleInterface* sp = (*b)->getSpecialPower()) return sp;
    }
    return nullptr;
}

static CollideModuleInterface* find_col(Object* obj) {
    if (!obj) return nullptr;
    for (BehaviorModule** b = obj->getBehaviorModules(); *b; ++b) {
        if (CollideModuleInterface* c = (*b)->getCollide()) return c;
    }
    return nullptr;
}

static void append_obj_spy_json(std::ostringstream& ss, Object* obj) {
    if (!obj || obj->isDestroyed()) {
        ss << "[-1,0,0,0]";
        return;
    }
    const Int* spied_by = obj->*access_member(ObjVisionSpiedByTag());
    const PlayerMaskType mask = obj->*access_member(ObjVisionSpiedMaskTag());
    ss << "[" << mask << "," << spied_by[1] << "," << spied_by[2] << "," << spied_by[3] << "]";
}

static Team* g_def_teams[4] = {};
static Team* g_alt_teams[4] = {};

static std::string run_seed(
    oracle_wp51::OracleTerrainLogic* terrain,
    oracle_wp51::World& w,
    int seed,
    std::string* s0_diag_out
) {
    TheWritableGlobalData->m_partitionCellSize = 20.0f;
    terrain->extent.lo = {0.0f, 0.0f, 0.0f};
    terrain->extent.hi = {400.0f, 400.0f, 100.0f};
    ThePartitionManager->reset();
    ThePartitionManager->init();

    const int players = 4;
    ThePlayerList->*access_member(Wp51PlayerListCountTag()) = players;
    for (int p = 0; p < players; ++p) {
        Player* pl = ThePlayerList->getNthPlayer(p);
        pl->init(ThePlayerTemplateStore->findPlayerTemplate(NAMEKEY("FactionSU")));
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

    // Player 0 is Neutral; 1 <-> 3 Enemies; 1 <-> 2 Allies or Enemies depending on seed; 2 <-> 3 Allies or Enemies.
    std::vector<int> rels(players * players, 2);
    for (int i = 0; i < players; ++i) {
        Player* pi = ThePlayerList->getNthPlayer(i);
        pi->setPlayerRelationship(pi, ALLIES);
        for (int j = 0; j < players; ++j) {
            if (i == j) continue;
            Relationship r = NEUTRAL;
            if ((i == 1 && j == 3) || (i == 3 && j == 1)) r = ENEMIES;
            else if ((i == 1 && j == 2) || (i == 2 && j == 1)) {
                r = (seed < 4 || (seed % 3) != 0) ? ENEMIES : ALLIES;
            } else if ((i == 2 && j == 3) || (i == 3 && j == 2)) {
                r = (seed % 2 == 0) ? ENEMIES : ALLIES;
            }
            if (seed >= 20 && i >= 1 && j >= 1) {
                if (!((i == 1 && j == 3) || (i == 3 && j == 1) || (i == 1 && j == 2) || (i == 2 && j == 1))) {
                    const uint32_t rv = pick(3);
                    r = (rv == 0) ? ENEMIES : ((rv == 1) ? NEUTRAL : ALLIES);
                }
            }
            pi->setPlayerRelationship(ThePlayerList->getNthPlayer(j), r);
            rels[i * players + j] = static_cast<int>(r);
        }
    }

    const UnsignedInt persist = (seed == 0) ? 3u : (seed % 5 == 0 ? 0u : (1u + pick(6)));
    TheWritableGlobalData->m_unlookPersistDuration = persist;
    UnsignedInt frame = (seed == 0) ? 1u : (1u + pick(40));
    w.set_frame(frame);

    // Seed parameters serialized explicitly in JSON.
    const UnsignedInt base_ms = (seed == 1) ? 0u : ((seed == 0) ? 200u : (pick(5) ? (100u + pick(400)) : 0u));
    const UnsignedInt bonus_ms = (seed == 0) ? 100u : (pick(4) * 100u);
    const UnsignedInt max_ms = std::max(base_ms + bonus_ms * 2u, 300u + pick(400));
    const int riders = (seed == 0) ? 2 : static_cast<int>(pick(4));

    const bool sv2_needs_upgrade = (seed == 0 || seed == 2 || (seed % 4 != 1));
    const bool sv2_self_powered = (seed == 0 || seed == 3 || (seed % 3 != 0));
    const UnsignedInt self_dur_ms = (seed == 2) ? 0u : ((seed == 0) ? 150u : (pick(4) == 0 ? 0u : (100u + pick(250))));
    const UnsignedInt self_int_ms = (seed == 3) ? 0u : ((seed == 0) ? 100u : (pick(4) == 0 ? 0u : (50u + pick(200))));
    const int spy_kind_mode = (seed == 0) ? 1 : (seed < 5 ? seed % 3 : static_cast<int>(pick(3)));
    const char* spy_kind_str = (spy_kind_mode == 0) ? "" : ((spy_kind_mode == 1) ? "VEHICLE" : "INFANTRY STRUCTURE");
    const UnsignedInt sab_ms = (seed == 0) ? 150u : (100u + pick(300));
    const bool test_disabled_sp = (seed == 0 || (seed % 5 == 0));
    const bool clear_hacked = (seed == 0 || (seed % 2 == 0));
    const UnsignedInt t_step1 = (seed == 0) ? 3u : (1u + pick(5));
    const UnsignedInt t_step2 = (seed == 0) ? 6u : ((seed == 3) ? 10u : (2u + pick(10)));
    const UnsignedInt t_step3 = (seed == 0) ? 8u : (2u + pick(10));

    // DynamicShroudClearingRangeUpdate parameters honoring C++ invariant: growDelay + growTime <= shrinkDelay.
    const Real dscru_vision = (seed == 0) ? 120.0f : static_cast<Real>(60 + pick(100));
    const bool dscru_inherit = (seed == 0 || seed == 11 || (seed % 3 == 0));
    const Real dscru_shroud_clear = dscru_inherit ? -1.0f : static_cast<Real>(80 + pick(120));
    const UnsignedInt grow_delay_ms = (seed == 10) ? 0u : ((seed == 0) ? 100u : pick(5) * 50u);
    const UnsignedInt grow_time_ms = (seed == 10) ? 0u : ((seed == 0) ? 200u : pick(6) * 50u);
    const UnsignedInt sustain_pad_ms = (seed == 10) ? 200u : ((seed == 0) ? 150u : (pick(5) * 50u));
    const UnsignedInt shrink_delay_ms = grow_delay_ms + grow_time_ms + sustain_pad_ms;
    const UnsignedInt shrink_time_ms = (seed == 10) ? 150u : ((seed == 0) ? 200u : (pick(5) == 0 ? 0u : (1u + pick(6)) * 50u));
    const UnsignedInt grow_interval_ms = (seed == 10) ? 0u : ((seed == 0) ? 50u : (pick(4) * 50u));
    const UnsignedInt change_interval_ms = (seed == 10) ? 0u : ((seed == 0) ? 50u : (pick(4) * 50u));
    const Real final_vision = (seed == 10) ? 0.0f : ((seed == 12) ? 180.0f : ((seed == 0) ? 40.0f : static_cast<Real>(pick(6) * 30u)));

    const std::string suffix = std::to_string(seed);
    const std::string sp_name = "SpSu_" + suffix;
    const std::string upg_name = "UpgSu_" + suffix;
    const std::string t_sv1 = "SuSv1_" + suffix;
    const std::string t_sv2 = "SuSv2_" + suffix;
    const std::string t_sab = "SuSab_" + suffix;
    const std::string t_dscru = "SuDscru_" + suffix;

    {
        std::ostringstream ini;
        ini << "Upgrade " << upg_name << "\n  Type = OBJECT\nEnd\n"
            << "SpecialPower " << sp_name << "\n  Enum = SPECIAL_SPY_SATELLITE\n  ReloadTime = 1\nEnd\n"
            << "Object " << t_sv1 << "\n"
            << "  KindOf = STRUCTURE FS_INTERNET_CENTER\n"
            << "  VisionRange = 40\n  ShroudClearingRange = 40\n"
            << "  Geometry = BOX\n  GeometryMajorRadius = 15\n  GeometryMinorRadius = 15\n"
            << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 300\n    InitialHealth = 300\n  End\n"
            << "  Behavior = InternetHackContain ModuleTag_Contain\n"
            << "    ContainMax = 5\n    Slots = 5\n    AllowInsideKindOf = INFANTRY\n"
            << "  End\n"
            << "  Behavior = SpyVisionUpdate ModuleTag_SVU\n"
            << "    NeedsUpgrade = No\n"
            << "    SelfPowered = No\n"
            << "  End\n"
            << "  Behavior = SpyVisionSpecialPower ModuleTag_SP\n"
            << "    SpecialPowerTemplate = " << sp_name << "\n"
            << "    BaseDuration = " << base_ms << "\n"
            << "    BonusDurationPerCaptured = " << bonus_ms << "\n"
            << "    MaxDuration = " << max_ms << "\n"
            << "  End\n"
            << "End\n"
            << "Object " << t_sv2 << "\n"
            << "  KindOf = STRUCTURE FS_INTERNET_CENTER\n"
            << "  VisionRange = 40\n  ShroudClearingRange = 40\n"
            << "  Geometry = BOX\n  GeometryMajorRadius = 15\n  GeometryMinorRadius = 15\n"
            << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 300\n    InitialHealth = 300\n  End\n"
            << "  Behavior = SpyVisionUpdate ModuleTag_SVU\n"
            << "    NeedsUpgrade = " << (sv2_needs_upgrade ? "Yes" : "No") << "\n"
            << "    TriggeredBy = " << upg_name << "\n"
            << "    SelfPowered = " << (sv2_self_powered ? "Yes" : "No") << "\n"
            << "    SelfPoweredDuration = " << self_dur_ms << "\n"
            << "    SelfPoweredInterval = " << self_int_ms << "\n";
        if (spy_kind_str[0] != '\0') {
            ini << "    SpyOnKindof = " << spy_kind_str << "\n";
        }
        ini << "  End\n"
            << "End\n"
            << "Object " << t_sab << "\n"
            << "  KindOf = INFANTRY\n"
            << "  VisionRange = 30\n  ShroudClearingRange = 30\n"
            << "  Geometry = SPHERE\n  GeometryMajorRadius = 8\n"
            << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
            << "  Behavior = SabotageInternetCenterCrateCollide ModuleTag_Collide\n"
            << "    SabotageDuration = " << sab_ms << "\n"
            << "    BuildingPickup = Yes\n"
            << "  End\n"
            << "End\n"
            << "Object " << t_dscru << "\n"
            << "  KindOf = IMMOBILE\n"
            << "  VisionRange = " << dscru_vision << "\n"
            << "  ShroudClearingRange = " << dscru_shroud_clear << "\n"
            << "  Geometry = SPHERE\n  GeometryMajorRadius = 10\n"
            << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
            << "  Behavior = DynamicShroudClearingRangeUpdate ModuleTag_DSCRU\n"
            << "    GrowDelay = " << grow_delay_ms << "\n"
            << "    GrowTime = " << grow_time_ms << "\n"
            << "    ShrinkDelay = " << shrink_delay_ms << "\n"
            << "    ShrinkTime = " << shrink_time_ms << "\n"
            << "    GrowInterval = " << grow_interval_ms << "\n"
            << "    ChangeInterval = " << change_interval_ms << "\n"
            << "    FinalVision = " << final_vision << "\n"
            << "    GridDecalTemplate\n"
            << "      Texture = SCCA10\n"
            << "      OpacityMin = 25%\n"
            << "      OpacityMax = 50%\n"
            << "      OpacityThrobTime = 500\n"
            << "      Color = R:255 G:255 B:0 A:255\n"
            << "      OnlyVisibleToOwningPlayer = Yes\n"
            << "    End\n"
            << "  End\n"
            << "End\n";
        w.load_ini(ini.str(), INI_LOAD_OVERWRITE);
    }

    const UpgradeTemplate* upg = TheUpgradeCenter->findUpgrade(upg_name.c_str());
    if (!upg) die("missing upgrade");

    std::ostringstream ss;
    ss << "{\"seed\":" << seed
       << ",\"persist\":" << persist
       << ",\"frame0\":" << frame
       << ",\"rels\":[";
    for (size_t i = 0; i < rels.size(); ++i) {
        if (i) ss << ',';
        ss << rels[i];
    }
    ss << "]"
       << ",\"base_ms\":" << base_ms
       << ",\"bonus_ms\":" << bonus_ms
       << ",\"max_ms\":" << max_ms
       << ",\"riders\":" << riders
       << ",\"sv2_needs_upg\":" << (sv2_needs_upgrade ? 1 : 0)
       << ",\"sv2_self\":" << (sv2_self_powered ? 1 : 0)
       << ",\"self_dur_ms\":" << self_dur_ms
       << ",\"self_int_ms\":" << self_int_ms
       << ",\"spy_kind\":\"" << spy_kind_str << "\""
       << ",\"sab_ms\":" << sab_ms
       << ",\"test_dis\":" << (test_disabled_sp ? 1 : 0)
       << ",\"clear_hack\":" << (clear_hacked ? 1 : 0)
       << ",\"steps\":[" << t_step1 << "," << t_step2 << "," << t_step3 << "]"
       << ",\"dscru_vis\":" << bits(dscru_vision)
       << ",\"dscru_scr\":" << bits(dscru_shroud_clear)
       << ",\"gd_ms\":" << grow_delay_ms
       << ",\"gt_ms\":" << grow_time_ms
       << ",\"sd_ms\":" << shrink_delay_ms
       << ",\"st_ms\":" << shrink_time_ms
       << ",\"gi_ms\":" << grow_interval_ms
       << ",\"ci_ms\":" << change_interval_ms
       << ",\"fv\":" << bits(final_vision);

    // Part A: SpyVisionUpdate scenario
    Object* o0 = w.spawn_object(t_sv1.c_str(), g_def_teams[1], {60.0f, 60.0f, 0.0f}, 0.0f);
    Object* o1 = w.spawn_object(t_sv2.c_str(), g_def_teams[1], {100.0f, 60.0f, 0.0f}, 0.0f);
    Object* o2 = w.spawn_object("SuTargetVeh", g_def_teams[3], {220.0f, 220.0f, 0.0f}, 0.0f);
    Object* o3 = w.spawn_object("SuTargetInf", g_alt_teams[3], {300.0f, 220.0f, 0.0f}, 0.0f);
    Object* o4 = w.spawn_object("SuTargetStruct", g_def_teams[2], {220.0f, 300.0f, 0.0f}, 0.0f);
    Object* o5 = w.spawn_object(t_sab.c_str(), g_def_teams[2], {360.0f, 60.0f, 0.0f}, 0.0f);
    if (!o0 || !o1 || !o2 || !o3 || !o4 || !o5) die("spawn part A");

    ContainModuleInterface* c0 = o0->getContain();
    for (int r = 0; r < riders; ++r) {
        Object* passenger = w.spawn_object("SuRiderInf", g_def_teams[1], {60.0f, 60.0f, 0.0f}, 0.0f);
        if (!passenger || !c0) die("spawn rider");
        c0->addToContain(passenger);
    }

    SpyVisionUpdate* sv0 = find_sv(o0);
    SpyVisionUpdate* sv1 = find_sv(o1);
    SpecialPowerModuleInterface* sp0 = find_sp(o0);
    CollideModuleInterface* col5 = find_col(o5);
    if (!sv0 || !sv1 || !sp0 || !col5) die("missing module part A");

    auto emit_sv_cp = [&](std::ostringstream& out, Object* s0, SpyVisionUpdate* u0, Object* s1, SpyVisionUpdate* u1) {
        out << "[";
        if (s0 && !s0->isDestroyed() && u0) {
            out << oracle_wp51::World::snapshot_crc(u0) << ",\"" << oracle_wp51::World::snapshot_save_hex(u0) << "\","
                << u0->getDisabledUntilFrame();
        } else {
            out << "0,\"\",0";
        }
        out << ",";
        if (s1 && !s1->isDestroyed() && u1) {
            out << oracle_wp51::World::snapshot_crc(u1) << ",\"" << oracle_wp51::World::snapshot_save_hex(u1) << "\","
                << u1->getDisabledUntilFrame();
        } else {
            out << "0,\"\",0";
        }
        out << "," << pm_crc() << ",";
        append_obj_spy_json(out, o2);
        out << ",";
        append_obj_spy_json(out, o3);
        out << ",";
        append_obj_spy_json(out, o4);
        out << "]";
    };

    std::ostringstream sv_cps;
    sv_cps << "[";
    ThePartitionManager->update();
    step_frame(w, frame);
    emit_sv_cp(sv_cps, o0, sv0, o1, sv1);

    // Optional disabled special-power check before triggering SP
    if (test_disabled_sp) {
        o0->setDisabledUntil(DISABLED_EMP, frame + 4u);
        sp0->doSpecialPower(0);
        o0->clearDisabled(DISABLED_EMP);
    }
    sp0->doSpecialPower(0);
    sv_cps << ",";
    emit_sv_cp(sv_cps, o0, sv0, o1, sv1);

    for (UnsignedInt k = 0; k < t_step1; ++k) step_frame(w, frame);
    // Grant upgrade on o1 so both SpyVisionUpdates can be active & test stacking
    o1->giveUpgrade(upg);
    sv_cps << ",";
    emit_sv_cp(sv_cps, o0, sv0, o1, sv1);

    // Sabotage o0 (internet center), which disables SpyVision on both o0 and o1
    const Coord3D col_loc = *o0->getPosition();
    const Coord3D col_norm = {0.0f, 0.0f, 1.0f};
    col5->onCollide(o0, &col_loc, &col_norm);
    sv_cps << ",";
    emit_sv_cp(sv_cps, o0, sv0, o1, sv1);

    for (UnsignedInt k = 0; k < t_step2; ++k) step_frame(w, frame);
    if (clear_hacked && !o0->isDestroyed()) {
        o0->clearDisabled(DISABLED_HACKED);
    }
    sv_cps << ",";
    emit_sv_cp(sv_cps, o0, sv0, o1, sv1);

    // Capture o1 from player 1 -> player 2 while potentially active
    if (!o1->isDestroyed()) {
        o1->setTeam(g_def_teams[2]);
    }
    for (UnsignedInt k = 0; k < t_step3; ++k) step_frame(w, frame);
    sv_cps << ",";
    emit_sv_cp(sv_cps, o0, sv0, o1, sv1);

    // Test SpyVisionUpdate XferSave -> XferLoad roundtrip on o1
    if (!o1->isDestroyed()) {
        oracle::MemoryXfer xs(XFER_SAVE);
        xs.open("sv_rt");
        oracle::call_snapshot_xfer(sv1, &xs);
        xs.close();
        const Int saved_idx = sv1->friend_getIndexInLogic();
        oracle_wp51::World::snapshot_load_bytes(sv1, xs.bytes);
        sv1->friend_setIndexInLogic(saved_idx);
    }

    // Destroy o0 (triggers onDelete), drain unlook persist
    TheGameLogic->destroyObject(o0);
    o0 = nullptr;
    sv0 = nullptr;
    for (UnsignedInt k = 0; k < persist + 2u; ++k) step_frame(w, frame);
    sv_cps << ",";
    emit_sv_cp(sv_cps, o0, sv0, o1, sv1);

    // Clean up all Part A objects and drain unlook queue before Part B
    for (Object* obj = TheGameLogic->getFirstObject(); obj; obj = obj->getNextObject()) {
        TheGameLogic->destroyObject(obj);
    }
    TheGameLogic->processDestroyList();
    for (UnsignedInt k = 0; k < persist + 2u; ++k) step_frame(w, frame);
    sv_cps << ",";
    emit_sv_cp(sv_cps, nullptr, nullptr, nullptr, nullptr);
    sv_cps << "]";

    // Part B: DynamicShroudClearingRangeUpdate scenario
    Object* od = w.spawn_object(t_dscru.c_str(), g_def_teams[1], {180.0f, 180.0f, 0.0f}, 0.0f);
    if (!od) die("spawn part B");
    UpdateModule* dscru = find_update_by_name(od, "DynamicShroudClearingRangeUpdate");
    if (!dscru) die("missing DynamicShroudClearingRangeUpdate");

    const UnsignedInt dscru_frames =
        (shrink_delay_ms + shrink_time_ms) / 33u + 16u;
    const UnsignedInt total_run = std::max<UnsignedInt>(dscru_frames + 10u, 30u);
    const UnsignedInt rt_frame = std::max<UnsignedInt>(2u, total_run / 2u);

    std::ostringstream ds_cps;
    ds_cps << "[";
    // Initial snapshot right after spawn + partition update (frame unchanged)
    ThePartitionManager->update();
    ds_cps << "[" << frame << "," << bits(od->getShroudClearingRange()) << ","
           << oracle_wp51::World::snapshot_crc(dscru) << ",\""
           << oracle_wp51::World::snapshot_save_hex(dscru) << "\"," << pm_crc() << "]";

    std::string s0_range_trace;
    for (UnsignedInt step = 1; step <= total_run; ++step) {
        step_frame(w, frame);
        if (step == rt_frame) {
            oracle::MemoryXfer xs(XFER_SAVE);
            xs.open("ds_rt");
            oracle::call_snapshot_xfer(dscru, &xs);
            xs.close();
            const Int saved_idx = dscru->friend_getIndexInLogic();
            oracle_wp51::World::snapshot_load_bytes(dscru, xs.bytes);
            dscru->friend_setIndexInLogic(saved_idx);
        }
        if (seed == 0) {
            if (!s0_range_trace.empty()) s0_range_trace += ",";
            s0_range_trace += std::to_string(bits(od->getShroudClearingRange()));
        }
        if (step == 1 || step == 3 || step == 6 || step == rt_frame || step == total_run / 3 ||
            step == (total_run * 2) / 3 || step == total_run - 2 || step == total_run) {
            ds_cps << ",[" << frame << "," << bits(od->getShroudClearingRange()) << ","
                   << oracle_wp51::World::snapshot_crc(dscru) << ",\""
                   << oracle_wp51::World::snapshot_save_hex(dscru) << "\"," << pm_crc() << "]";
        }
    }
    ds_cps << "]";

    // Clean up Part B objects and drain queue
    for (Object* obj = TheGameLogic->getFirstObject(); obj; obj = obj->getNextObject()) {
        TheGameLogic->destroyObject(obj);
    }
    TheGameLogic->processDestroyList();
    for (UnsignedInt k = 0; k < persist + 2u; ++k) step_frame(w, frame);

    ss << ",\"sv_cps\":" << sv_cps.str()
       << ",\"ds_cps\":" << ds_cps.str()
       << "}";

    if (seed == 0 && s0_diag_out) {
        std::ostringstream d;
        d << "{\"frame0\":" << 1
          << ",\"sv_steps\":[" << t_step1 << "," << t_step2 << "," << t_step3 << "]"
          << ",\"dscru_total_run\":" << total_run
          << ",\"dscru_range_bits\":[" << s0_range_trace << "]}";
        *s0_diag_out = d.str();
    }
    return ss.str();
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_shroud_updates <out.json>\n");
        return 2;
    }
    const std::string out_path = argv[1];

    oracle_wp51::World w;
    w.load_ini(
        "PlayerTemplate FactionSU\n  Side = America\n  PlayableSide = Yes\nEnd\n"
        "Object SuRiderInf\n"
        "  KindOf = INFANTRY\n  TransportSlotCount = 1\n  VisionRange = 50\n  ShroudClearingRange = 50\n"
        "  Geometry = SPHERE\n  GeometryMajorRadius = 5\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 50\n    InitialHealth = 50\n  End\n"
        "End\n"
        "Object SuTargetVeh\n"
        "  KindOf = VEHICLE\n  VisionRange = 70\n  ShroudClearingRange = 70\n"
        "  Geometry = SPHERE\n  GeometryMajorRadius = 10\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "End\n"
        "Object SuTargetInf\n"
        "  KindOf = INFANTRY\n  VisionRange = 60\n  ShroudClearingRange = 60\n"
        "  Geometry = SPHERE\n  GeometryMajorRadius = 6\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 80\n    InitialHealth = 80\n  End\n"
        "End\n"
        "Object SuTargetStruct\n"
        "  KindOf = STRUCTURE\n  VisionRange = 80\n  ShroudClearingRange = 80\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 15\n  GeometryMinorRadius = 15\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 300\n    InitialHealth = 300\n  End\n"
        "End\n"
    );

    Player* nobody = NEW Player(99);
    ThePlayerList->*access_member(SuLocalTag()) = nobody;
    if (rts::getObservedOrLocalPlayer()->getPlayerIndex() != 99) die("local player");

    for (int p = 0; p < 4; ++p) {
        Dict d1, d2;
        char n1[32], n2[32];
        std::snprintf(n1, sizeof(n1), "SuDefTeam_%d", p);
        std::snprintf(n2, sizeof(n2), "SuAltTeam_%d", p);
        TheTeamFactory->initTeam(n1, AsciiString::TheEmptyString, TRUE, &d1);
        TheTeamFactory->initTeam(n2, AsciiString::TheEmptyString, TRUE, &d2);
        g_def_teams[p] = TheTeamFactory->findTeam(n1);
        g_alt_teams[p] = TheTeamFactory->findTeam(n2);
        if (!g_def_teams[p] || !g_alt_teams[p]) die("missing team");
    }

    auto* terrain = static_cast<oracle_wp51::OracleTerrainLogic*>(TheTerrainLogic);
    const int kVectors = 300;
    std::string s0_diag = "{}";
    std::vector<std::string> vectors;
    vectors.reserve(kVectors);
    for (int s = 0; s < kVectors; ++s) {
        vectors.push_back(run_seed(terrain, w, s, s == 0 ? &s0_diag : nullptr));
    }

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    std::fprintf(
        f,
        "{\n\"packet\": \"SHROUD_UPDATES\",\n\"vector_count\": %d,\n"
        "\"sha256_SpyVisionUpdate\": \"%s\",\n"
        "\"sha256_DynamicShroudClearingRangeUpdate\": \"%s\",\n"
        "\"seed0_diagnostics\": %s,\n\"vectors\": [\n",
        kVectors,
        oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/SpyVisionUpdate.cpp").c_str(),
        oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/DynamicShroudClearingRangeUpdate.cpp").c_str(),
        s0_diag.c_str()
    );
    for (size_t i = 0; i < vectors.size(); ++i) {
        std::fprintf(f, "%s%s\n", vectors[i].c_str(), i + 1 < vectors.size() ? "," : "");
    }
    std::fprintf(f, "]\n}\n");
    std::fclose(f);
    if (std::rename(tmp.c_str(), out_path.c_str())) { std::perror("rename"); return 1; }
    return 0;
}
