// WP-5.2-D1 CollideModule & Sabotage CrateCollide hierarchy (10 packets:
// PKT-199, 204, 232, 233, 234, 235, 236, 237, 238, 239)
// C++ Golden Vector Generator (Zero-Stub Build against oracle_wp51_world.h).
//
// Canonical build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp52_collide_sabotage.cpp --opt O2 \
//     --out /tmp/gen_wp52_d1_o2
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_wp52_d1_o2 docs/golden_vectors/WP52_CollideSabotage.json

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Science.h"
#include "Common/SpecialPower.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "GameClient/FXList.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/CollideModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/CrateCollide.h"
#include "GameLogic/Module/OCLUpdate.h"
#include "GameLogic/Module/SabotageCommandCenterCrateCollide.h"
#include "GameLogic/Module/SabotageFakeBuildingCrateCollide.h"
#include "GameLogic/Module/SabotageInternetCenterCrateCollide.h"
#include "GameLogic/Module/SabotageMilitaryFactoryCrateCollide.h"
#include "GameLogic/Module/SabotagePowerPlantCrateCollide.h"
#include "GameLogic/Module/SabotageSuperweaponCrateCollide.h"
#include "GameLogic/Module/SabotageSupplyCenterCrateCollide.h"
#include "GameLogic/Module/SabotageSupplyDropzoneCrateCollide.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"

#include <sstream>
#include <string>
#include <vector>

struct PlayerTypeTag { using type = PlayerType Player::*; };
template struct MemberAccess<PlayerTypeTag, &Player::m_playerType>;
PlayerType Player::* access_member(PlayerTypeTag);

struct PlayerSciD1Tag { using type = ScienceVec Player::*; };
template struct MemberAccess<PlayerSciD1Tag, &Player::m_sciences>;
ScienceVec Player::* access_member(PlayerSciD1Tag);

struct OclTimerStartTag { using type = UnsignedInt OCLUpdate::*; };
template struct MemberAccess<OclTimerStartTag, &OCLUpdate::m_timerStartedFrame>;
UnsignedInt OCLUpdate::* access_member(OclTimerStartTag);

struct OclNextCreateTag { using type = UnsignedInt OCLUpdate::*; };
template struct MemberAccess<OclNextCreateTag, &OCLUpdate::m_nextCreationFrame>;
UnsignedInt OCLUpdate::* access_member(OclNextCreateTag);

static CollideModule* find_first_collide(Object* obj) {
    if (!obj) return nullptr;
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if ((*m)->getCollide()) {
            return static_cast<CollideModule*>(*m);
        }
    }
    return nullptr;
}

static std::vector<SpecialPowerModule*> find_all_sp(Object* obj) {
    std::vector<SpecialPowerModule*> out;
    if (!obj) return out;
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if ((*m)->getSpecialPower()) {
            out.push_back(static_cast<SpecialPowerModule*>(*m));
        }
    }
    return out;
}

static OCLUpdate* find_ocl_update(Object* obj) {
    if (!obj) return nullptr;
    static NameKeyType key_ocl = NAMEKEY("OCLUpdate");
    return static_cast<OCLUpdate*>(obj->findUpdateModule(key_ocl));
}

static void set_player_money(Player* p, UnsignedInt amount) {
    if (!p) return;
    Money* m = p->getMoney();
    if (!m) return;
    UnsignedInt cur = m->countMoney();
    if (cur > 0) m->withdraw(cur, FALSE);
    if (amount > 0) m->deposit(amount, FALSE, FALSE);
}

static std::vector<uint8_t> hex_to_bytes(const std::string& hex) {
    std::vector<uint8_t> out;
    out.reserve(hex.size() / 2);
    auto nibble = [](char c) -> uint8_t {
        if (c >= '0' && c <= '9') return static_cast<uint8_t>(c - '0');
        if (c >= 'a' && c <= 'f') return static_cast<uint8_t>(10 + (c - 'a'));
        if (c >= 'A' && c <= 'F') return static_cast<uint8_t>(10 + (c - 'A'));
        return 0;
    };
    for (size_t i = 0; i + 1 < hex.size(); i += 2) {
        out.push_back(static_cast<uint8_t>((nibble(hex[i]) << 4) | nibble(hex[i + 1])));
    }
    return out;
}

static void destroy_if_live(Object* obj) {
    if (obj && !obj->isDestroyed()) {
        TheGameLogic->destroyObject(obj);
    }
}

class HeadlessInGameUI : public oracle_wp51::NullInGameUI {
public:
    void addFloatingText(const UnicodeString&, const Coord3D*, Color) override {}
};

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/WP52_CollideSabotage.json";
    oracle_wp51::World w;
    TheInGameUI = new HeadlessInGameUI;

    // Player 0 (p_a / team_a): Neutral.
    // Player 1 (p_b / team_b): Active player A (saboteurs/crates).
    // Player 2 (p_c / team_c): Active player B (victim structures / test targets).
    Player* p_a = ThePlayerList->getNthPlayer(0);
    Player* p_b = ThePlayerList->getNthPlayer(1);
    Player* p_c = ThePlayerList->getNthPlayer(2);
    ThePlayerList->*access_member(Wp51PlayerListCountTag()) = 3;
    p_c->init(nullptr);
    if (!p_c->getResourceGatheringManager()) {
        p_c->*access_member(Wp51PlayerRgmTag()) = newInstance(ResourceGatheringManager);
    }
    Dict dict_c;
    TheTeamFactory->initTeam("TeamC", AsciiString::TheEmptyString, TRUE, &dict_c);
    Team* team_c = TheTeamFactory->findTeam("TeamC");
    team_c->setControllingPlayer(p_c);
    p_c->*access_member(Wp51PlayerDefaultTeamTag()) = team_c;

    p_a->setPlayerRelationship(p_b, NEUTRAL);
    p_b->setPlayerRelationship(p_a, NEUTRAL);
    p_a->setPlayerRelationship(p_c, NEUTRAL);
    p_c->setPlayerRelationship(p_a, NEUTRAL);

    // Static rules and target templates loaded once before the 1,000-seed loop.
    w.load_ini(
        "Science SCI_WP52_COL\n"
        "  SciencePurchasePointCost = 1\n"
        "  IsGrantable = Yes\n"
        "End\n"
        "FXList FX_CollideExec\n"
        "End\n"
        "SpecialPower SpWP52_Col1\n"
        "  Enum = SPECIAL_CASH_HACK\n"
        "  ReloadTime = 1000\n"
        "End\n"
        "SpecialPower SpWP52_Col2\n"
        "  Enum = SPECIAL_SPY_SATELLITE\n"
        "  ReloadTime = 2000\n"
        "End\n"
        "Object Col_DummyItem\n"
        "  KindOf = INFANTRY\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 50\n"
        "    InitialHealth = 50\n"
        "  End\n"
        "End\n"
        "ObjectCreationList OCL_DropzoneTest\n"
        "  CreateObject\n"
        "    ObjectNames = Col_DummyItem\n"
        "    Count = 1\n"
        "  End\n"
        "End\n"
        "Object Col_TargetParachute\n"
        "  KindOf = PARACHUTE INFANTRY\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 100\n"
        "    InitialHealth = 100\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "End\n"
        "Object Col_TargetCmdCenter\n"
        "  KindOf = STRUCTURE COMMANDCENTER\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = CashHackSpecialPower ModuleTag_SP1\n"
        "    SpecialPowerTemplate = SpWP52_Col1\n"
        "    MoneyAmount = 500\n"
        "  End\n"
        "  Behavior = SpyVisionSpecialPower ModuleTag_SP2\n"
        "    SpecialPowerTemplate = SpWP52_Col2\n"
        "  End\n"
        "End\n"
        "Object Col_TargetFake\n"
        "  KindOf = STRUCTURE FS_FAKE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 150\n"
        "    InitialHealth = 150\n"
        "  End\n"
        "End\n"
        "Object Col_TargetInternet\n"
        "  KindOf = STRUCTURE FS_INTERNET_CENTER\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = InternetHackContain ModuleTag_Contain\n"
        "    Slots = 4\n"
        "    AllowInsideKindOf = INFANTRY\n"
        "  End\n"
        "End\n"
        "Object Col_TargetBarracks\n"
        "  KindOf = STRUCTURE FS_BARRACKS\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "End\n"
        "Object Col_TargetWarfactory\n"
        "  KindOf = STRUCTURE FS_WARFACTORY\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "End\n"
        "Object Col_TargetAirfield\n"
        "  KindOf = STRUCTURE FS_AIRFIELD\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "End\n"
        "Object Col_TargetCarrier\n"
        "  KindOf = STRUCTURE FS_AIRFIELD AIRCRAFT_CARRIER\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "End\n"
        "Object Col_TargetPower\n"
        "  KindOf = STRUCTURE FS_POWER\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "End\n"
        "Object Col_TargetSuperweapon\n"
        "  KindOf = STRUCTURE FS_SUPERWEAPON\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 300\n"
        "    InitialHealth = 300\n"
        "  End\n"
        "  Behavior = CashHackSpecialPower ModuleTag_SP1\n"
        "    SpecialPowerTemplate = SpWP52_Col1\n"
        "    MoneyAmount = 500\n"
        "  End\n"
        "  Behavior = SpyVisionSpecialPower ModuleTag_SP2\n"
        "    SpecialPowerTemplate = SpWP52_Col2\n"
        "  End\n"
        "End\n"
        "Object Col_TargetStrategy\n"
        "  KindOf = STRUCTURE FS_STRATEGY_CENTER\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 300\n"
        "    InitialHealth = 300\n"
        "  End\n"
        "  Behavior = CashHackSpecialPower ModuleTag_SP1\n"
        "    SpecialPowerTemplate = SpWP52_Col1\n"
        "    MoneyAmount = 500\n"
        "  End\n"
        "End\n"
        "Object Col_TargetSupplyCenter\n"
        "  KindOf = STRUCTURE FS_SUPPLY_CENTER\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "End\n"
        "Object Col_TargetDropzone\n"
        "  KindOf = STRUCTURE FS_SUPPLY_DROPZONE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = OCLUpdate ModuleTag_OCL\n"
        "    OCL = OCL_DropzoneTest\n"
        "    MinDelay = 3000\n"
        "    MaxDelay = 3000\n"
        "  End\n"
        "End\n"
    );

    const ScienceType sci_col = TheScienceStore->getScienceFromInternalName("SCI_WP52_COL");

    std::ostringstream vecs;
    std::string s0_diag;
    uint32_t rng = 0x52D199u;
    auto next_u32 = [&]() -> uint32_t {
        return oracle::splitmix32(rng);
    };

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        const std::string sfx = std::to_string(seed);
        const UnsignedInt frame0 = 10u + (next_u32() % 200u);
        w.set_frame(frame0);

        const uint32_t rel_roll = next_u32() % 4u; // 0,1,2 -> ENEMIES; 3 -> ALLIES or NEUTRAL
        const Relationship rel_bc = (rel_roll < 3u)
            ? ENEMIES
            : ((next_u32() & 1u) ? ALLIES : NEUTRAL);
        p_b->setPlayerRelationship(p_c, rel_bc);
        p_c->setPlayerRelationship(p_b, rel_bc);

        const bool pc_human = (next_u32() & 1u) != 0;
        const bool pb_human = (next_u32() & 1u) != 0;
        p_b->*access_member(PlayerTypeTag()) = pb_human ? PLAYER_HUMAN : PLAYER_COMPUTER;
        p_c->*access_member(PlayerTypeTag()) = pc_human ? PLAYER_HUMAN : PLAYER_COMPUTER;

        const bool pc_has_sci = (next_u32() & 1u) != 0;
        (p_c->*access_member(PlayerSciD1Tag())).clear();
        if (pc_has_sci) {
            (p_c->*access_member(PlayerSciD1Tag())).push_back(sci_col);
        }
        (p_b->*access_member(PlayerSciD1Tag())).clear();

        // Part A inputs (CrateCollide matrix via SabotageMilitaryFactoryCrateCollide)
        const bool forbid_owner = (next_u32() & 1u) != 0;
        const bool building_pickup = (next_u32() % 4u) != 0; // 75% Yes
        const bool human_only = (next_u32() % 4u) == 0;      // 25% Yes
        const bool allow_multi = (next_u32() % 3u) == 0;     // 33% Yes
        const bool req_science = (next_u32() % 4u) == 0;     // 25% SCI_WP52_COL
        const bool use_fx_a = (next_u32() & 1u) != 0;
        const uint32_t req_k_mode = next_u32() % 3u;         // 0=none,1=STRUCTURE,2=INFANTRY
        const uint32_t forb_k_mode = next_u32() % 2u;        // 0=none,1=FS_AIRFIELD
        const bool crate_above = (next_u32() % 4u) == 0;     // 25% airborne
        const bool crate_has_ai = (next_u32() % 5u) != 0;    // 80% has AI
        const uint32_t goal_mode_a = next_u32() % 3u;        // 0=null,1=target,2=decoy
        const bool target_same_owner = (next_u32() % 6u) == 0;
        const uint32_t target_sel_a = next_u32() % 11u;      // 0..8 (+ 9,10 alias normal barracks for higher execution hit rate)

        // Part B inputs (Sabotage* suite)
        const UnsignedInt sab_dur_ms = (1u + (next_u32() % 10u)) * 1000u; // 30..300 frames
        const UnsignedInt pow_dur_ms = (1u + (next_u32() % 12u)) * 1000u; // 30..360 frames
        const UnsignedInt steal_amt = 100u + (next_u32() % 1200u);
        const UnsignedInt pb_money0 = next_u32() % 800u;
        const UnsignedInt pc_money0 = next_u32() % 1500u;
        const bool use_fx_b = (next_u32() & 1u) != 0;
        const bool sab_goal_match = (next_u32() % 4u) != 0;  // 75% matches target
        const uint32_t sab_status_mode = next_u32() % 6u;    // 0..3 normal, 4 UNDER_CONST, 5 SOLD
        const uint32_t fac_kind_sel = next_u32() % 4u;       // 0=Barracks,1=Warfactory,2=Airfield,3=Carrier
        const uint32_t sw_kind_sel = next_u32() & 1u;        // 0=Superweapon,1=StrategyCenter

        const std::string t_probe = "Sab52_Probe_" + sfx;
        const std::string t_cmd_name = "Sab52_Cmd_" + sfx;
        const std::string t_fake_name = "Sab52_Fake_" + sfx;
        const std::string t_net_name = "Sab52_Net_" + sfx;
        const std::string t_fac_name = "Sab52_Fac_" + sfx;
        const std::string t_pow_name = "Sab52_Pow_" + sfx;
        const std::string t_sw_name = "Sab52_Sw_" + sfx;
        const std::string t_sup_name = "Sab52_Sup_" + sfx;
        const std::string t_drop_name = "Sab52_Drop_" + sfx;

        // Build per-seed INI so every seed parses fresh ModuleData (avoids ThingTemplate tag reuse).
        std::ostringstream dyn_ini;
        dyn_ini << "Object " << t_probe << "\n"
                << "  KindOf = INFANTRY\n"
                << "  Body = ActiveBody ModuleTag_Body\n"
                << "    MaxHealth = 100\n"
                << "    InitialHealth = 100\n"
                << "  End\n";
        if (crate_has_ai) {
            dyn_ini << "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n";
        }
        dyn_ini << "  Behavior = SabotageMilitaryFactoryCrateCollide ModuleTag_Collide\n"
                << "    SabotageDuration = " << sab_dur_ms << "\n"
                << "    ForbidOwnerPlayer = " << (forbid_owner ? "Yes" : "No") << "\n"
                << "    BuildingPickup = " << (building_pickup ? "Yes" : "No") << "\n"
                << "    HumanOnly = " << (human_only ? "Yes" : "No") << "\n"
                << "    AllowMultiPickup = " << (allow_multi ? "Yes" : "No") << "\n";
        if (req_k_mode == 1) dyn_ini << "    RequiredKindOf = STRUCTURE\n";
        else if (req_k_mode == 2) dyn_ini << "    RequiredKindOf = INFANTRY\n";
        if (forb_k_mode == 1) dyn_ini << "    ForbiddenKindOf = FS_AIRFIELD\n";
        if (req_science) dyn_ini << "    PickupScience = SCI_WP52_COL\n";
        if (use_fx_a) dyn_ini << "    ExecuteFX = FX_CollideExec\n";
        dyn_ini << "    ExecuteAnimation = AnimProbe\n"
                << "    ExecuteAnimationTime = 1.5\n"
                << "    ExecuteAnimationZRise = 2.5\n"
                << "    ExecuteAnimationFades = No\n"
                << "  End\n"
                << "End\n";

        auto append_sab_obj = [&](const std::string& name, const char* mod_name, const char* extra_line) {
            dyn_ini << "Object " << name << "\n"
                    << "  KindOf = INFANTRY\n"
                    << "  Body = ActiveBody ModuleTag_Body\n"
                    << "    MaxHealth = 100\n"
                    << "    InitialHealth = 100\n"
                    << "  End\n"
                    << "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
                    << "  Behavior = " << mod_name << " ModuleTag_Collide\n"
                    << "    BuildingPickup = Yes\n";
            if (use_fx_b) dyn_ini << "    ExecuteFX = FX_CollideExec\n";
            if (extra_line && extra_line[0]) dyn_ini << extra_line;
            dyn_ini << "  End\n"
                    << "End\n";
        };
        std::string dur_line = "    SabotageDuration = " + std::to_string(sab_dur_ms) + "\n";
        std::string pow_line = "    SabotagePowerDuration = " + std::to_string(pow_dur_ms) + "\n";
        std::string steal_line = "    StealCashAmount = " + std::to_string(steal_amt) + "\n";

        append_sab_obj(t_cmd_name, "SabotageCommandCenterCrateCollide", "");
        append_sab_obj(t_fake_name, "SabotageFakeBuildingCrateCollide", "");
        append_sab_obj(t_net_name, "SabotageInternetCenterCrateCollide", dur_line.c_str());
        append_sab_obj(t_fac_name, "SabotageMilitaryFactoryCrateCollide", dur_line.c_str());
        append_sab_obj(t_pow_name, "SabotagePowerPlantCrateCollide", pow_line.c_str());
        append_sab_obj(t_sw_name, "SabotageSuperweaponCrateCollide", "");
        append_sab_obj(t_sup_name, "SabotageSupplyCenterCrateCollide", steal_line.c_str());
        append_sab_obj(t_drop_name, "SabotageSupplyDropzoneCrateCollide", steal_line.c_str());

        w.load_ini(dyn_ini.str(), INI_LOAD_OVERWRITE);

        // =========================================================================
        // Part A: CrateCollide base validity, multi-pickup guard, AI goal, snapshot
        // =========================================================================
        Team* target_team_a = target_same_owner ? w.team_b : team_c;
        Object* decoy_a = w.spawn_object("Col_DummyItem", team_c);
        Coord3D crate_pos = {0.0f, 0.0f, crate_above ? 25.0f : 0.0f};
        Object* crate = w.spawn_object(t_probe.c_str(), w.team_b, crate_pos);
        Object* target_a = nullptr;
        switch (target_sel_a) {
            case 0: target_a = nullptr; break;
            case 1: target_a = w.spawn_object("Col_TargetBarracks", w.team_a); break;
            case 2: target_a = w.spawn_object("Col_TargetParachute", target_team_a); break;
            case 3:
                target_a = w.spawn_object("Col_TargetBarracks", target_team_a);
                target_a->setEffectivelyDead(TRUE);
                break;
            case 4: target_a = w.spawn_object("Col_DummyItem", target_team_a); break;
            case 6:
                target_a = w.spawn_object("Col_TargetBarracks", target_team_a);
                target_a->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
                break;
            case 7:
                target_a = w.spawn_object("Col_TargetBarracks", target_team_a);
                target_a->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_SOLD));
                break;
            case 8: target_a = w.spawn_object("Col_TargetAirfield", target_team_a); break;
            case 5:
            case 9:
            case 10:
            default: target_a = w.spawn_object("Col_TargetBarracks", target_team_a); break;
        }
        if (crate_has_ai && crate->getAIUpdateInterface()) {
            if (goal_mode_a == 1) crate->getAIUpdateInterface()->friend_setGoalObject(target_a);
            else if (goal_mode_a == 2) crate->getAIUpdateInterface()->friend_setGoalObject(decoy_a);
            else crate->getAIUpdateInterface()->friend_setGoalObject(nullptr);
        }

        CollideModule* cm_a = find_first_collide(crate);
        const Bool a_would_ground = cm_a->wouldLikeToCollideWith(nullptr);
        const Bool a_would_1 = cm_a->wouldLikeToCollideWith(target_a);
        const Coord3D col_loc = {0.0f, 0.0f, 0.0f};
        const Coord3D col_norm = {0.0f, 0.0f, 1.0f};
        cm_a->onCollide(target_a, &col_loc, &col_norm);
        const Bool a_dest_1 = crate->isDestroyed();
        const Bool a_would_2 = cm_a->wouldLikeToCollideWith(target_a);
        const UnsignedInt a_until_1 = target_a ? target_a->getDisabledUntil(DISABLED_HACKED) : 0u;
        cm_a->onCollide(target_a, &col_loc, &col_norm);
        const Bool a_dest_2 = crate->isDestroyed();
        const Bool a_hacked_2 = target_a ? target_a->isDisabledByType(DISABLED_HACKED) : FALSE;
        const UnsignedInt a_until_2 = target_a ? target_a->getDisabledUntil(DISABLED_HACKED) : 0u;

        const uint32_t a_crc = oracle_wp51::World::snapshot_crc(cm_a);
        uint32_t a_save_crc = 0;
        const std::string a_xfer = oracle_wp51::World::snapshot_save_hex(cm_a, &a_save_crc);
        oracle_wp51::World::snapshot_load_bytes(cm_a, hex_to_bytes(a_xfer));
        const std::string a_rt_xfer = oracle_wp51::World::snapshot_save_hex(cm_a);

        destroy_if_live(crate);
        destroy_if_live(target_a);
        destroy_if_live(decoy_a);
        w.flush_destroy_list();

        // =========================================================================
        // Part B: All 8 Sabotage* CrateCollide modules
        // =========================================================================
        w.set_frame(frame0);
        Object* decoy_b = w.spawn_object("Col_DummyItem", team_c);
        Object* t_cmd = w.spawn_object("Col_TargetCmdCenter", team_c);
        Object* t_fake = w.spawn_object("Col_TargetFake", team_c);
        Object* t_net = w.spawn_object("Col_TargetInternet", team_c);
        Object* t_hacker = w.spawn_object("Col_DummyItem", team_c);
        t_net->getContain()->addToContain(t_hacker);
        const char* fac_tmpl = (fac_kind_sel == 0) ? "Col_TargetBarracks"
                             : (fac_kind_sel == 1) ? "Col_TargetWarfactory"
                             : (fac_kind_sel == 2) ? "Col_TargetAirfield"
                                                   : "Col_TargetCarrier";
        Object* t_fac = w.spawn_object(fac_tmpl, team_c);
        Object* t_carrier = w.spawn_object("Col_TargetCarrier", team_c);
        Object* t_pow = w.spawn_object("Col_TargetPower", team_c);
        Object* t_sw = w.spawn_object(sw_kind_sel == 0 ? "Col_TargetSuperweapon" : "Col_TargetStrategy", team_c);
        Object* t_sup = w.spawn_object("Col_TargetSupplyCenter", team_c);
        Object* t_drop = w.spawn_object("Col_TargetDropzone", team_c);

        if (sab_status_mode == 4) {
            Object* all_t[] = {t_cmd, t_fake, t_net, t_fac, t_pow, t_sw, t_sup, t_drop};
            for (Object* o : all_t) {
                o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
            }
        } else if (sab_status_mode == 5) {
            Object* all_t[] = {t_cmd, t_fake, t_net, t_fac, t_pow, t_sw, t_sup, t_drop};
            for (Object* o : all_t) {
                o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_SOLD));
            }
        }

        // Advance frame so CommandCenter & Superweapon powers are ready before sabotage.
        const UnsignedInt sab_frame = frame0 + 50u;
        w.set_frame(sab_frame);
        p_c->getEnergy()->setPowerSabotagedTillFrame(0u);

        auto spawn_sab = [&](const std::string& tmpl, Object* target) -> std::pair<Object*, CollideModule*> {
            Object* s = w.spawn_object(tmpl.c_str(), w.team_b);
            if (s->getAIUpdateInterface()) {
                s->getAIUpdateInterface()->friend_setGoalObject(sab_goal_match ? target : decoy_b);
            }
            return {s, find_first_collide(s)};
        };

        // 1. SabotageCommandCenterCrateCollide
        auto [s_cmd, m_cmd] = spawn_sab(t_cmd_name, t_cmd);
        const Bool cmd_is_sab = m_cmd->isSabotageBuildingCrateCollide();
        const Bool cmd_would_bad = m_cmd->wouldLikeToCollideWith(t_fake);
        const Bool cmd_would_ok = m_cmd->wouldLikeToCollideWith(t_cmd);
        m_cmd->onCollide(t_cmd, &col_loc, &col_norm);
        const Bool cmd_dest = s_cmd->isDestroyed();
        auto cmd_sps = find_all_sp(t_cmd);
        const UnsignedInt cmd_sp1_ready = cmd_sps.size() > 0 ? cmd_sps[0]->getReadyFrame() : 0u;
        const UnsignedInt cmd_sp2_ready = cmd_sps.size() > 1 ? cmd_sps[1]->getReadyFrame() : 0u;
        const uint32_t cmd_crc = oracle_wp51::World::snapshot_crc(m_cmd);
        const std::string cmd_xfer = oracle_wp51::World::snapshot_save_hex(m_cmd);

        // 2. SabotageFakeBuildingCrateCollide
        auto [s_fake, m_fake] = spawn_sab(t_fake_name, t_fake);
        const Bool fake_is_sab = m_fake->isSabotageBuildingCrateCollide();
        const Bool fake_would_bad = m_fake->wouldLikeToCollideWith(t_cmd);
        const Bool fake_would_ok = m_fake->wouldLikeToCollideWith(t_fake);
        m_fake->onCollide(t_fake, &col_loc, &col_norm);
        const Bool fake_dest = s_fake->isDestroyed();
        const Bool fake_t_dead = t_fake->isEffectivelyDead();
        const Real fake_t_hp = t_fake->getBodyModule()->getHealth();
        const uint32_t fake_crc = oracle_wp51::World::snapshot_crc(m_fake);
        const std::string fake_xfer = oracle_wp51::World::snapshot_save_hex(m_fake);

        // 3. SabotageInternetCenterCrateCollide
        auto [s_net, m_net] = spawn_sab(t_net_name, t_net);
        const Bool net_is_sab = m_net->isSabotageBuildingCrateCollide();
        const Bool net_would_bad = m_net->wouldLikeToCollideWith(t_fake);
        const Bool net_would_ok = m_net->wouldLikeToCollideWith(t_net);
        m_net->onCollide(t_net, &col_loc, &col_norm);
        const Bool net_dest = s_net->isDestroyed();
        const Bool net_t_hacked = t_net->isDisabledByType(DISABLED_HACKED);
        const UnsignedInt net_t_until = t_net->getDisabledUntil(DISABLED_HACKED);
        const Bool net_h_hacked = t_hacker->isDisabledByType(DISABLED_HACKED);
        const UnsignedInt net_h_until = t_hacker->getDisabledUntil(DISABLED_HACKED);
        const uint32_t net_crc = oracle_wp51::World::snapshot_crc(m_net);
        const std::string net_xfer = oracle_wp51::World::snapshot_save_hex(m_net);

        // 4. SabotageMilitaryFactoryCrateCollide
        auto [s_fac, m_fac] = spawn_sab(t_fac_name, t_fac);
        const Bool fac_is_sab = m_fac->isSabotageBuildingCrateCollide();
        const Bool fac_would_carrier = m_fac->wouldLikeToCollideWith(t_carrier);
        const Bool fac_would_ok = m_fac->wouldLikeToCollideWith(t_fac);
        m_fac->onCollide(t_fac, &col_loc, &col_norm);
        const Bool fac_dest = s_fac->isDestroyed();
        const Bool fac_t_hacked = t_fac->isDisabledByType(DISABLED_HACKED);
        const UnsignedInt fac_t_until = t_fac->getDisabledUntil(DISABLED_HACKED);
        const uint32_t fac_crc = oracle_wp51::World::snapshot_crc(m_fac);
        const std::string fac_xfer = oracle_wp51::World::snapshot_save_hex(m_fac);

        // 5. SabotagePowerPlantCrateCollide
        auto [s_pow, m_pow] = spawn_sab(t_pow_name, t_pow);
        const Bool pow_is_sab = m_pow->isSabotageBuildingCrateCollide();
        const Bool pow_would_bad = m_pow->wouldLikeToCollideWith(t_fake);
        const Bool pow_would_ok = m_pow->wouldLikeToCollideWith(t_pow);
        m_pow->onCollide(t_pow, &col_loc, &col_norm);
        const Bool pow_dest = s_pow->isDestroyed();
        const UnsignedInt pow_until = p_c->getEnergy()->getPowerSabotagedTillFrame();
        const uint32_t pow_crc = oracle_wp51::World::snapshot_crc(m_pow);
        const std::string pow_xfer = oracle_wp51::World::snapshot_save_hex(m_pow);

        // 6. SabotageSuperweaponCrateCollide
        auto [s_sw, m_sw] = spawn_sab(t_sw_name, t_sw);
        const Bool sw_is_sab = m_sw->isSabotageBuildingCrateCollide();
        const Bool sw_would_bad = m_sw->wouldLikeToCollideWith(t_cmd);
        const Bool sw_would_ok = m_sw->wouldLikeToCollideWith(t_sw);
        m_sw->onCollide(t_sw, &col_loc, &col_norm);
        const Bool sw_dest = s_sw->isDestroyed();
        auto sw_sps = find_all_sp(t_sw);
        const UnsignedInt sw_sp1_ready = sw_sps.size() > 0 ? sw_sps[0]->getReadyFrame() : 0u;
        const UnsignedInt sw_sp2_ready = sw_sps.size() > 1 ? sw_sps[1]->getReadyFrame() : 0u;
        const uint32_t sw_crc = oracle_wp51::World::snapshot_crc(m_sw);
        const std::string sw_xfer = oracle_wp51::World::snapshot_save_hex(m_sw);

        // 7. SabotageSupplyCenterCrateCollide
        set_player_money(p_b, pb_money0);
        set_player_money(p_c, pc_money0);
        auto [s_sup, m_sup] = spawn_sab(t_sup_name, t_sup);
        const Bool sup_is_sab = m_sup->isSabotageBuildingCrateCollide();
        const Bool sup_would_bad = m_sup->wouldLikeToCollideWith(t_drop);
        const Bool sup_would_ok = m_sup->wouldLikeToCollideWith(t_sup);
        m_sup->onCollide(t_sup, &col_loc, &col_norm);
        const Bool sup_dest = s_sup->isDestroyed();
        const UnsignedInt sup_pb_money = p_b->getMoney()->countMoney();
        const UnsignedInt sup_pc_money = p_c->getMoney()->countMoney();
        const uint32_t sup_crc = oracle_wp51::World::snapshot_crc(m_sup);
        const std::string sup_xfer = oracle_wp51::World::snapshot_save_hex(m_sup);

        // 8. SabotageSupplyDropzoneCrateCollide
        set_player_money(p_b, pb_money0);
        set_player_money(p_c, pc_money0);
        auto [s_drop, m_drop] = spawn_sab(t_drop_name, t_drop);
        const Bool drop_is_sab = m_drop->isSabotageBuildingCrateCollide();
        const Bool drop_would_bad = m_drop->wouldLikeToCollideWith(t_sup);
        const Bool drop_would_ok = m_drop->wouldLikeToCollideWith(t_drop);
        m_drop->onCollide(t_drop, &col_loc, &col_norm);
        const Bool drop_dest = s_drop->isDestroyed();
        const UnsignedInt drop_pb_money = p_b->getMoney()->countMoney();
        const UnsignedInt drop_pc_money = p_c->getMoney()->countMoney();
        OCLUpdate* drop_ocl = find_ocl_update(t_drop);
        const UnsignedInt drop_ocl_start = drop_ocl ? drop_ocl->*access_member(OclTimerStartTag()) : 0u;
        const UnsignedInt drop_ocl_next = drop_ocl ? drop_ocl->*access_member(OclNextCreateTag()) : 0u;
        const uint32_t drop_crc = oracle_wp51::World::snapshot_crc(m_drop);
        const std::string drop_xfer = oracle_wp51::World::snapshot_save_hex(m_drop);

        const uint32_t is_sab_mask =
            (cmd_is_sab ? 1u : 0u) |
            ((fake_is_sab ? 1u : 0u) << 1) |
            ((net_is_sab ? 1u : 0u) << 2) |
            ((fac_is_sab ? 1u : 0u) << 3) |
            ((pow_is_sab ? 1u : 0u) << 4) |
            ((sw_is_sab ? 1u : 0u) << 5) |
            ((sup_is_sab ? 1u : 0u) << 6) |
            ((drop_is_sab ? 1u : 0u) << 7);

        // Clean up all Part B objects cleanly (contained hacker first).
        destroy_if_live(t_hacker);
        destroy_if_live(s_cmd);
        destroy_if_live(s_fake);
        destroy_if_live(s_net);
        destroy_if_live(s_fac);
        destroy_if_live(s_pow);
        destroy_if_live(s_sw);
        destroy_if_live(s_sup);
        destroy_if_live(s_drop);
        destroy_if_live(t_cmd);
        destroy_if_live(t_fake);
        destroy_if_live(t_net);
        destroy_if_live(t_fac);
        destroy_if_live(t_carrier);
        destroy_if_live(t_pow);
        destroy_if_live(t_sw);
        destroy_if_live(t_sup);
        destroy_if_live(t_drop);
        destroy_if_live(decoy_b);
        w.flush_destroy_list();

        if (seed == 0) {
            std::ostringstream d;
            d << "{"
              << "\"crate\":{\"would_ground\":" << (a_would_ground ? 1 : 0)
              << ",\"would_1\":" << (a_would_1 ? 1 : 0)
              << ",\"dest_1\":" << (a_dest_1 ? 1 : 0)
              << ",\"would_2\":" << (a_would_2 ? 1 : 0)
              << ",\"dest_2\":" << (a_dest_2 ? 1 : 0)
              << ",\"until_1\":" << a_until_1
              << ",\"until_2\":" << a_until_2
              << ",\"xfer\":\"" << a_xfer << "\",\"rt_xfer\":\"" << a_rt_xfer << "\"},"
              << "\"cmd\":{\"would_ok\":" << (cmd_would_ok ? 1 : 0) << ",\"dest\":" << (cmd_dest ? 1 : 0)
              << ",\"sp1_ready\":" << cmd_sp1_ready << ",\"sp2_ready\":" << cmd_sp2_ready
              << ",\"xfer\":\"" << cmd_xfer << "\"},"
              << "\"fake\":{\"would_ok\":" << (fake_would_ok ? 1 : 0) << ",\"dest\":" << (fake_dest ? 1 : 0)
              << ",\"dead\":" << (fake_t_dead ? 1 : 0) << ",\"hp_bits\":" << oracle::fbits(fake_t_hp)
              << ",\"xfer\":\"" << fake_xfer << "\"},"
              << "\"net\":{\"would_ok\":" << (net_would_ok ? 1 : 0) << ",\"dest\":" << (net_dest ? 1 : 0)
              << ",\"t_hacked\":" << (net_t_hacked ? 1 : 0) << ",\"t_until\":" << net_t_until
              << ",\"h_hacked\":" << (net_h_hacked ? 1 : 0) << ",\"h_until\":" << net_h_until
              << ",\"xfer\":\"" << net_xfer << "\"},"
              << "\"fac\":{\"would_carrier\":" << (fac_would_carrier ? 1 : 0) << ",\"would_ok\":" << (fac_would_ok ? 1 : 0)
              << ",\"dest\":" << (fac_dest ? 1 : 0) << ",\"t_until\":" << fac_t_until
              << ",\"xfer\":\"" << fac_xfer << "\"},"
              << "\"pow\":{\"would_ok\":" << (pow_would_ok ? 1 : 0) << ",\"dest\":" << (pow_dest ? 1 : 0)
              << ",\"until\":" << pow_until << ",\"xfer\":\"" << pow_xfer << "\"},"
              << "\"sw\":{\"would_ok\":" << (sw_would_ok ? 1 : 0) << ",\"dest\":" << (sw_dest ? 1 : 0)
              << ",\"sp1_ready\":" << sw_sp1_ready << ",\"sp2_ready\":" << sw_sp2_ready
              << ",\"xfer\":\"" << sw_xfer << "\"},"
              << "\"sup\":{\"would_ok\":" << (sup_would_ok ? 1 : 0) << ",\"dest\":" << (sup_dest ? 1 : 0)
              << ",\"pb\":" << sup_pb_money << ",\"pc\":" << sup_pc_money
              << ",\"xfer\":\"" << sup_xfer << "\"},"
              << "\"drop\":{\"would_ok\":" << (drop_would_ok ? 1 : 0) << ",\"dest\":" << (drop_dest ? 1 : 0)
              << ",\"pb\":" << drop_pb_money << ",\"pc\":" << drop_pc_money
              << ",\"ocl_start\":" << drop_ocl_start << ",\"ocl_next\":" << drop_ocl_next
              << ",\"xfer\":\"" << drop_xfer << "\"}"
              << "}";
            s0_diag = d.str();
        }

        if (seed > 0) vecs << ",\n";
        vecs << "    {"
             << "\"seed\":" << seed
             << ",\"in\":{"
             << "\"frame\":" << frame0
             << ",\"rel_bc\":" << static_cast<int>(rel_bc)
             << ",\"pc_human\":" << (pc_human ? 1 : 0)
             << ",\"pb_human\":" << (pb_human ? 1 : 0)
             << ",\"pc_has_sci\":" << (pc_has_sci ? 1 : 0)
             << ",\"forbid_owner\":" << (forbid_owner ? 1 : 0)
             << ",\"building_pickup\":" << (building_pickup ? 1 : 0)
             << ",\"human_only\":" << (human_only ? 1 : 0)
             << ",\"allow_multi\":" << (allow_multi ? 1 : 0)
             << ",\"req_science\":" << (req_science ? 1 : 0)
             << ",\"use_fx_a\":" << (use_fx_a ? 1 : 0)
             << ",\"req_k_mode\":" << req_k_mode
             << ",\"forb_k_mode\":" << forb_k_mode
             << ",\"crate_above\":" << (crate_above ? 1 : 0)
             << ",\"crate_has_ai\":" << (crate_has_ai ? 1 : 0)
             << ",\"goal_mode_a\":" << goal_mode_a
             << ",\"target_same_owner\":" << (target_same_owner ? 1 : 0)
             << ",\"target_sel_a\":" << target_sel_a
             << ",\"sab_dur_ms\":" << sab_dur_ms
             << ",\"pow_dur_ms\":" << pow_dur_ms
             << ",\"steal_amt\":" << steal_amt
             << ",\"pb_money0\":" << pb_money0
             << ",\"pc_money0\":" << pc_money0
             << ",\"use_fx_b\":" << (use_fx_b ? 1 : 0)
             << ",\"sab_goal_match\":" << (sab_goal_match ? 1 : 0)
             << ",\"sab_status_mode\":" << sab_status_mode
             << ",\"fac_kind_sel\":" << fac_kind_sel
             << ",\"sw_kind_sel\":" << sw_kind_sel
             << "}"
             << ",\"out\":{"
             << "\"is_sab_mask\":" << is_sab_mask
             << ",\"a\":[" << (a_would_ground ? 1 : 0) << "," << (a_would_1 ? 1 : 0) << ","
             << (a_dest_1 ? 1 : 0) << "," << (a_would_2 ? 1 : 0) << "," << (a_dest_2 ? 1 : 0) << ","
             << (a_hacked_2 ? 1 : 0) << "," << a_until_1 << "," << a_until_2 << "]"
             << ",\"a_crc\":" << a_crc
             << ",\"a_xfer\":\"" << a_xfer << "\""
             << ",\"a_rt_xfer\":\"" << a_rt_xfer << "\""
             << ",\"cmd\":[" << (cmd_would_bad ? 1 : 0) << "," << (cmd_would_ok ? 1 : 0) << ","
             << (cmd_dest ? 1 : 0) << "," << cmd_sp1_ready << "," << cmd_sp2_ready << "]"
             << ",\"cmd_crc\":" << cmd_crc << ",\"cmd_xfer\":\"" << cmd_xfer << "\""
             << ",\"fake\":[" << (fake_would_bad ? 1 : 0) << "," << (fake_would_ok ? 1 : 0) << ","
             << (fake_dest ? 1 : 0) << "," << (fake_t_dead ? 1 : 0) << "," << oracle::fbits(fake_t_hp) << "]"
             << ",\"fake_crc\":" << fake_crc << ",\"fake_xfer\":\"" << fake_xfer << "\""
             << ",\"net\":[" << (net_would_bad ? 1 : 0) << "," << (net_would_ok ? 1 : 0) << ","
             << (net_dest ? 1 : 0) << "," << (net_t_hacked ? 1 : 0) << "," << net_t_until << ","
             << (net_h_hacked ? 1 : 0) << "," << net_h_until << "]"
             << ",\"net_crc\":" << net_crc << ",\"net_xfer\":\"" << net_xfer << "\""
             << ",\"fac\":[" << (fac_would_carrier ? 1 : 0) << "," << (fac_would_ok ? 1 : 0) << ","
             << (fac_dest ? 1 : 0) << "," << (fac_t_hacked ? 1 : 0) << "," << fac_t_until << "]"
             << ",\"fac_crc\":" << fac_crc << ",\"fac_xfer\":\"" << fac_xfer << "\""
             << ",\"pow\":[" << (pow_would_bad ? 1 : 0) << "," << (pow_would_ok ? 1 : 0) << ","
             << (pow_dest ? 1 : 0) << "," << pow_until << "]"
             << ",\"pow_crc\":" << pow_crc << ",\"pow_xfer\":\"" << pow_xfer << "\""
             << ",\"sw\":[" << (sw_would_bad ? 1 : 0) << "," << (sw_would_ok ? 1 : 0) << ","
             << (sw_dest ? 1 : 0) << "," << sw_sp1_ready << "," << sw_sp2_ready << "]"
             << ",\"sw_crc\":" << sw_crc << ",\"sw_xfer\":\"" << sw_xfer << "\""
             << ",\"sup\":[" << (sup_would_bad ? 1 : 0) << "," << (sup_would_ok ? 1 : 0) << ","
             << (sup_dest ? 1 : 0) << "," << sup_pb_money << "," << sup_pc_money << "]"
             << ",\"sup_crc\":" << sup_crc << ",\"sup_xfer\":\"" << sup_xfer << "\""
             << ",\"drop\":[" << (drop_would_bad ? 1 : 0) << "," << (drop_would_ok ? 1 : 0) << ","
             << (drop_dest ? 1 : 0) << "," << drop_pb_money << "," << drop_pc_money << ","
             << drop_ocl_start << "," << drop_ocl_next << "]"
             << ",\"drop_crc\":" << drop_crc << ",\"drop_xfer\":\"" << drop_xfer << "\""
             << "}"
             << "}";
    }

    std::ostringstream doc;
    doc << "{\n"
        << "  \"packet\": \"WP-5.2-D1\",\n"
        << "  \"modules\": [\"CollideModule\",\"CrateCollide\",\"SabotageCommandCenterCrateCollide\",\"SabotageFakeBuildingCrateCollide\",\"SabotageInternetCenterCrateCollide\",\"SabotageMilitaryFactoryCrateCollide\",\"SabotagePowerPlantCrateCollide\",\"SabotageSuperweaponCrateCollide\",\"SabotageSupplyCenterCrateCollide\",\"SabotageSupplyDropzoneCrateCollide\"],\n"
        << "  \"seed_count\": 1000,\n"
        << "  \"source_sha256\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Collide/CrateCollide/CrateCollide.cpp") << "\",\n"
        << "  \"seed0_diagnostics\": " << s0_diag << ",\n"
        << "  \"vectors\": [\n" << vecs.str() << "\n  ]\n}\n";

    oracle::write_file_atomic(out_path, doc.str());
    return 0;
}
