// WP-5.2-C SpecialPowerModule hierarchy (11 packets: PKT-195, 196, 197, 198,
// 209, 210, 216, 226, 242, 244, 245)
// C++ Golden Vector Generator (Zero-Stub Build against oracle_wp51_world.h).

#include "oracle_wp51_world.h"

#include "Common/Money.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Science.h"
#include "Common/SpecialPower.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "GameClient/DisplayStringManager.h"
#include "GameClient/Drawable.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BaikonurLaunchPower.h"
#include "GameLogic/Module/CashBountyPower.h"
#include "GameLogic/Module/CashHackSpecialPower.h"
#include "GameLogic/Module/CleanupAreaPower.h"
#include "GameLogic/Module/DefectorSpecialPower.h"
#include "GameLogic/Module/FireWeaponPower.h"
#include "GameLogic/Module/OCLSpecialPower.h"
#include "GameLogic/Module/SpecialAbility.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Module/SpyVisionSpecialPower.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"

#ifdef ALLOW_DEMORALIZE
#include "GameLogic/Module/DemoralizeSpecialPower.h"
#endif

#include <sstream>
#include <string>
#include <unordered_set>
#include <vector>

struct PlayerSciTag { using type = ScienceVec Player::*; };
template struct MemberAccess<PlayerSciTag, &Player::m_sciences>;
ScienceVec Player::* access_member(PlayerSciTag);

static SpecialPowerModule* find_sp_module(Object* obj) {
    if (!obj) return nullptr;
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if ((*m)->getSpecialPower()) {
            return static_cast<SpecialPowerModule*>(*m);
        }
    }
    return nullptr;
}

static uint32_t obj_count_live() {
    uint32_t cnt = 0;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        if (!o->isDestroyed()) ++cnt;
    }
    return cnt;
}

static std::vector<Object*> collect_live_objects() {
    std::vector<Object*> out;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        if (!o->isDestroyed()) out.push_back(o);
    }
    return out;
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

static void set_player_money(Player* p, UnsignedInt amount) {
    Money* m = p->getMoney();
    if (!m) return;
    UnsignedInt cur = m->countMoney();
    if (cur > 0) m->withdraw(cur);
    if (amount > 0) m->deposit(amount);
}

class StubDisplayString : public DisplayString {
public:
    void* operator new(size_t s) { return std::malloc(s); }
    void operator delete(void* p) { std::free(p); }
    MemoryPool* getObjectMemoryPool() override { return nullptr; }
    void setWordWrap(Int) override {}
    void setWordWrapCentered(Bool) override {}
    void draw(Int, Int, Color, Color) override {}
    void draw(Int, Int, Color, Color, Int, Int) override {}
    void getSize(Int* w, Int* h) override { if (w) *w = 0; if (h) *h = 0; }
    Int getWidth(Int = -1) override { return 0; }
    void setUseHotkey(Bool, Color) override {}
};

class StubDisplayStringManager : public DisplayStringManager {
public:
    DisplayString* newDisplayString() override {
        DisplayString* s = new StubDisplayString;
        link(s);
        return s;
    }
    void freeDisplayString(DisplayString* s) override {
        if (s) {
            unLink(s);
            delete static_cast<StubDisplayString*>(s);
        }
    }
    DisplayString* getGroupNumeralString(Int) override { return nullptr; }
    DisplayString* getFormationLetterString() override { return nullptr; }
};

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/WP52_SpecialPower.json";
    oracle_wp51::World w;
    delete TheDisplayStringManager;
    TheDisplayStringManager = new StubDisplayStringManager;
    TheDisplayStringManager->init();

    const Int demoralize_mask =
        TheModuleFactory->findModuleInterfaceMask(AsciiString("DemoralizeSpecialPower"), MODULETYPE_BEHAVIOR);
    const bool demoralize_compiled = (demoralize_mask != 0);

    w.load_ini(
        "Science SCI_BOUNTY\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n"
        "Science SCI_CASH_UPG\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n"
        "Science SCI_OCL_UPG\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n"
        "Science SCI_SHARED_REQ\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n"

        "Weapon WP52_Wpn\n"
        "  PrimaryDamage = 15\n"
        "  AttackRange = 100\n"
        "  DamageType = SMALL_ARMS\n"
        "  DeathType = NORMAL\n"
        "  WeaponSpeed = 999999\n"
        "  ClipSize = 2\n"
        "  ClipReloadTime = 1000\n"
        "  DelayBetweenShots = 100\n"
        "End\n"

        "SpecialPower Sp_Normal\n"
        "  Enum = SPECIAL_CASH_HACK\n"
        "  ReloadTime = 900\n"
        "  InitiateSound = NoSound\n"
        "End\n"
        "SpecialPower Sp_Ready\n"
        "  Enum = SPECIAL_SPY_SATELLITE\n"
        "  ReloadTime = 600\n"
        "  ViewObjectRange = 200\n"
        "  ViewObjectDuration = 150\n"
        "End\n"
        "SpecialPower Sp_Shared\n"
        "  Enum = SPECIAL_PARADROP_AMERICA\n"
        "  ReloadTime = 1200\n"
        "  SharedSyncedTimer = Yes\n"
        "  PublicTimer = No\n"
        "  RequiredScience = SCI_SHARED_REQ\n"
        "End\n"
        "SpecialPower Sp_Bounty\n"
        "  Enum = SPECIAL_CASH_BOUNTY\n"
        "  ReloadTime = 0\n"
        "  RequiredScience = SCI_BOUNTY\n"
        "End\n"
        "SpecialPower Sp_Defect\n"
        "  Enum = SPECIAL_DEFECTOR\n"
        "  ReloadTime = 1500\n"
        "  DetectionTime = 240\n"
        "End\n"
        "SpecialPower Sp_Baikonur\n"
        "  Enum = SPECIAL_LAUNCH_BAIKONUR_ROCKET\n"
        "  ReloadTime = 1800\n"
        "End\n"
        "SpecialPower Sp_Cleanup\n"
        "  Enum = SPECIAL_CLEANUP_AREA\n"
        "  ReloadTime = 0\n"
        "End\n"
        "SpecialPower Sp_FireWep\n"
        "  Enum = SPECIAL_DAISY_CUTTER\n"
        "  ReloadTime = 300\n"
        "End\n"
        "SpecialPower Sp_Script\n"
        "  Enum = SPECIAL_CARPET_BOMB\n"
        "  ReloadTime = 450\n"
        "End\n"

        "Object WP52_Detonation\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 50\n    InitialHealth = 50\n  End\n"
        "End\n"
        "Object WP52_OCLDefault\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 50\n    InitialHealth = 50\n  End\n"
        "End\n"
        "Object WP52_OCLUpgrade\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 50\n    InitialHealth = 50\n  End\n"
        "End\n"

        "ObjectCreationList OCL_WP52_Default\n"
        "  CreateObject\n    ObjectNames = WP52_OCLDefault\n    Count = 1\n  End\n"
        "End\n"
        "ObjectCreationList OCL_WP52_Upgrade\n"
        "  CreateObject\n    ObjectNames = WP52_OCLUpgrade\n    Count = 1\n  End\n"
        "End\n"

        "Object Obj_SP_Basic\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = SpecialAbility ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Normal\n"
        "    StartsPaused = No\n"
        "    StartsReady = No\n"
        "    ScriptedSpecialPowerOnly = No\n"
        "  End\n"
        "End\n"
        "Object Obj_SP_Ready\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = SpecialAbility ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Ready\n"
        "    StartsPaused = No\n"
        "    StartsReady = Yes\n"
        "    InitiateSound = NoSound\n"
        "  End\n"
        "End\n"
        "Object Obj_SP_Paused\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = SpecialAbility ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Normal\n"
        "    StartsPaused = Yes\n"
        "    StartsReady = Yes\n"
        "  End\n"
        "End\n"
        "Object Obj_SP_SharedStruct\n"
        "  KindOf = STRUCTURE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = SpecialAbility ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Shared\n"
        "    StartsPaused = No\n"
        "    StartsReady = No\n"
        "  End\n"
        "End\n"
        "Object Obj_SP_Script\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = SpecialAbility ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Script\n"
        "    ScriptedSpecialPowerOnly = Yes\n"
        "  End\n"
        "End\n"
        "Object Obj_Baikonur\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = BaikonurLaunchPower ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Baikonur\n"
        "    DetonationObject = WP52_Detonation\n"
        "  End\n"
        "End\n"
        "Object Obj_CashBounty_Low\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = CashBountyPower ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Bounty\n"
        "    Bounty = 10%\n"
        "  End\n"
        "End\n"
        "Object Obj_CashBounty_High\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = CashBountyPower ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Bounty\n"
        "    Bounty = 25%\n"
        "  End\n"
        "End\n"
        "Object Obj_CashHack\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = CashHackSpecialPower ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Normal\n"
        "    UpgradeMoneyAmount = SCI_CASH_UPG 2500\n"
        "    MoneyAmount = 1000\n"
        "  End\n"
        "End\n"
        "Object Obj_Cleanup\n"
        "  KindOf = VEHICLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Behavior = CleanupHazardUpdate ModuleTag_CHU\n    WeaponSlot = PRIMARY\n  End\n"
        "  Behavior = CleanupAreaPower ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Cleanup\n"
        "    MaxMoveDistanceFromLocation = 225.0\n"
        "  End\n"
        "End\n"
        "Object Obj_Defector\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = DefectorSpecialPower ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Defect\n"
        "    FatCursorRadius = 45.0\n"
        "  End\n"
        "End\n"
        "Object Obj_TargetInf\n"
        "  KindOf = INFANTRY SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "End\n"
        "Object Obj_FireWep\n"
        "  KindOf = CAN_ATTACK SELECTABLE\n"
        "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY WP52_Wpn\n  End\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Behavior = FireWeaponPower ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_FireWep\n"
        "    MaxShotsToFire = 3\n"
        "  End\n"
        "End\n"
        "Object Obj_OCL_Loc\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = OCLSpecialPower ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Normal\n"
        "    OCL = OCL_WP52_Default\n"
        "    UpgradeOCL = SCI_OCL_UPG OCL_WP52_Upgrade\n"
        "    CreateLocation = CREATE_AT_LOCATION\n"
        "    ReferenceObject = WP52_Detonation\n"
        "    OCLAdjustPositionToPassable = No\n"
        "  End\n"
        "End\n"
        "Object Obj_OCL_Above\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = OCLSpecialPower ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Normal\n"
        "    OCL = OCL_WP52_Default\n"
        "    UpgradeOCL = SCI_OCL_UPG OCL_WP52_Upgrade\n"
        "    CreateLocation = CREATE_ABOVE_LOCATION\n"
        "    ReferenceObject = WP52_Detonation\n"
        "    OCLAdjustPositionToPassable = Yes\n"
        "  End\n"
        "End\n"
        "Object Obj_OCL_Owner\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = OCLSpecialPower ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Normal\n"
        "    OCL = OCL_WP52_Default\n"
        "    CreateLocation = USE_OWNER_OBJECT\n"
        "  End\n"
        "End\n"
        "Object Obj_SpyVision\n"
        "  KindOf = STRUCTURE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = SpyVisionUpdate ModuleTag_SVU\n    NeedsUpgrade = No\n  End\n"
        "  Behavior = SpyVisionSpecialPower ModuleTag_SP\n"
        "    SpecialPowerTemplate = Sp_Ready\n"
        "    BaseDuration = 5000\n"
        "    BonusDurationPerCaptured = 2000\n"
        "    MaxDuration = 12000\n"
        "  End\n"
        "End\n"
    );

    Player* p_a = ThePlayerList->getNthPlayer(0);
    Player* p_b = ThePlayerList->getNthPlayer(1);

    const SpecialPowerTemplate* sp_normal = TheSpecialPowerStore->findSpecialPowerTemplate("Sp_Normal");
    const SpecialPowerTemplate* sp_ready = TheSpecialPowerStore->findSpecialPowerTemplate("Sp_Ready");
    const SpecialPowerTemplate* sp_shared = TheSpecialPowerStore->findSpecialPowerTemplate("Sp_Shared");
    const SpecialPowerTemplate* sp_bounty = TheSpecialPowerStore->findSpecialPowerTemplate("Sp_Bounty");

    ScienceType sci_bounty = TheScienceStore->friend_lookupScience("SCI_BOUNTY");
    ScienceType sci_cash_upg = TheScienceStore->friend_lookupScience("SCI_CASH_UPG");
    ScienceType sci_ocl_upg = TheScienceStore->friend_lookupScience("SCI_OCL_UPG");

    const ThingTemplate* tt_basic = TheThingFactory->findTemplate("Obj_SP_Basic");

    std::string s0_diag;
    std::ostringstream vecs;

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed ^ 0x52c01234u;
        if (rng == 0) rng = 0x98765431u;

        UnsignedInt frame0 = 20u + (oracle::splitmix32(rng) % 400u);
        UnsignedInt step_dt = 5u + (oracle::splitmix32(rng) % 20u);
        UnsignedInt custom_ready = frame0 + 10u + (oracle::splitmix32(rng) % 50u);
        bool under_const = (seed == 0) ? false : ((oracle::splitmix32(rng) % 4u) == 0);
        bool disabled_flag = (seed == 0) ? false : ((oracle::splitmix32(rng) % 5u) == 0);
        bool has_bounty_sci = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        bool has_cash_upg = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        bool has_ocl_upg = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        bool ocl_above = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        bool ocl_reject = (seed == 0) ? true : ((oracle::splitmix32(rng) % 3u) == 0);
        UnsignedInt money_b = (seed == 0) ? 1600u : (oracle::splitmix32(rng) % 4000u);
        UnsignedInt money_a = (seed == 0) ? 500u : (oracle::splitmix32(rng) % 2000u);
        Real loc_x = 120.0f + static_cast<Real>(oracle::splitmix32(rng) % 400u);
        Real loc_y = 140.0f + static_cast<Real>(oracle::splitmix32(rng) % 400u);

        // Reset player state deterministically
        (p_a->*access_member(PlayerSciTag())).clear();
        (p_b->*access_member(PlayerSciTag())).clear();
        p_a->setCashBounty(0.0f);
        p_b->setCashBounty(0.0f);
        set_player_money(p_a, money_a);
        set_player_money(p_b, money_b);
        p_a->expressSpecialPowerReadyFrame(sp_shared, frame0);

        if (has_bounty_sci) (p_a->*access_member(PlayerSciTag())).push_back(sci_bounty);
        if (has_cash_upg) (p_a->*access_member(PlayerSciTag())).push_back(sci_cash_upg);
        if (has_ocl_upg) (p_a->*access_member(PlayerSciTag())).push_back(sci_ocl_upg);

        w.set_frame(frame0);

        // --- 1) SpecialPowerModule (PKT-244) & SpecialAbility (PKT-242) ---
        ObjectStatusMaskType init_status;
        if (under_const) {
            init_status.set(OBJECT_STATUS_UNDER_CONSTRUCTION);
        }
        Object* o_basic = TheThingFactory->newObject(tt_basic, w.team_a, init_status);
        SpecialPowerModule* sp_b = find_sp_module(o_basic);
        bool sp_for_norm = sp_b->isModuleForPower(sp_normal);
        bool sp_for_ready = sp_b->isModuleForPower(sp_ready);
        UnsignedInt sp_ready_init = static_cast<const SpecialPowerModule*>(sp_b)->getReadyFrame();
        bool sp_is_ready_init = sp_b->isReady();

        w.set_frame(frame0 + step_dt);
        Real pct_mid = sp_b->getPercentReady();
        // Nested pause_countdown(TRUE) x2 -> unpause once -> still paused -> unpause twice
        sp_b->pauseCountdown(TRUE);
        sp_b->pauseCountdown(TRUE);
        Real pct_paused = sp_b->getPercentReady();
        UnsignedInt ready_paused = static_cast<const SpecialPowerModule*>(sp_b)->getReadyFrame();
        sp_b->pauseCountdown(FALSE);
        bool still_paused = (sp_b->getPercentReady() == pct_paused);
        w.set_frame(frame0 + step_dt + 7u);
        sp_b->pauseCountdown(FALSE);
        UnsignedInt ready_unpaused = static_cast<const SpecialPowerModule*>(sp_b)->getReadyFrame();

        sp_b->setReadyFrame(custom_ready);
        UnsignedInt ready_after_set = static_cast<const SpecialPowerModule*>(sp_b)->getReadyFrame();

        // Snapshot save/crc/load roundtrip on paused-adjusted state
        uint32_t sp_xfer_crc = 0;
        std::string sp_hex = oracle_wp51::World::snapshot_save_hex(sp_b, &sp_xfer_crc);
        uint32_t sp_crc = oracle_wp51::World::snapshot_crc(sp_b);
        sp_b->setReadyFrame(123456u);
        oracle_wp51::World::snapshot_load_bytes(sp_b, hex_to_bytes(sp_hex));
        UnsignedInt ready_after_load = static_cast<const SpecialPowerModule*>(sp_b)->getReadyFrame();

        // Disabled & Ready & ScriptOnly & SharedNSync checks
        Object* o_rdy = w.spawn_object("Obj_SP_Ready", w.team_a);
        SpecialPowerModule* sp_r = find_sp_module(o_rdy);
        bool rdy_starts_ready = sp_r->startsReady();
        UnsignedInt rdy_frame0 = static_cast<const SpecialPowerModule*>(sp_r)->getReadyFrame();
        bool rdy_is_ready0 = sp_r->isReady();

        Object* o_pau = w.spawn_object("Obj_SP_Paused", w.team_a);
        SpecialPowerModule* sp_p = find_sp_module(o_pau);
        Real pct_paused_100 = sp_p->getPercentReady();
        if (disabled_flag) o_pau->setDisabled(DISABLED_EMP);
        sp_p->doSpecialPower(0);
        UnsignedInt pau_ready_after_do = static_cast<const SpecialPowerModule*>(sp_p)->getReadyFrame();

        Object* o_scr = w.spawn_object("Obj_SP_Script", w.team_a);
        SpecialPowerModule* sp_sc = find_sp_module(o_scr);
        bool script_only_true = sp_sc->isScriptOnly();
        bool script_only_false = sp_b->isScriptOnly();

        Object* o_shr = w.spawn_object("Obj_SP_SharedStruct", w.team_a);
        SpecialPowerModule* sp_sh = find_sp_module(o_shr);
        UnsignedInt shr_ready_pre = static_cast<const SpecialPowerModule*>(sp_sh)->getReadyFrame();
        sp_sh->onSpecialPowerCreation();
        UnsignedInt shr_ready_post_create = static_cast<const SpecialPowerModule*>(sp_sh)->getReadyFrame();
        sp_sh->doSpecialPower(0);
        UnsignedInt shr_ready_post_do = static_cast<const SpecialPowerModule*>(sp_sh)->getReadyFrame();

        // --- 2) BaikonurLaunchPower (PKT-195) ---
        Object* o_baik = w.spawn_object("Obj_Baikonur", w.team_a);
        SpecialPowerModule* sp_baik = find_sp_module(o_baik);
        if (disabled_flag) o_baik->setDisabled(DISABLED_EMP);
        sp_baik->doSpecialPower(0);
        bool baik_door = o_baik->getDrawable() && o_baik->getDrawable()->getModelConditionFlags().test(MODELCONDITION_DOOR_1_OPENING);
        UnsignedInt baik_ready = static_cast<const SpecialPowerModule*>(sp_baik)->getReadyFrame();
        uint32_t baik_cnt_pre = obj_count_live();
        Coord3D baik_loc = { loc_x, loc_y, 0.0f };
        sp_baik->doSpecialPowerAtLocation(&baik_loc, 0.0f, 0);
        uint32_t baik_cnt_post = obj_count_live();
        std::string baik_hex = oracle_wp51::World::snapshot_save_hex(sp_baik);

        // --- 3) CashBountyPower (PKT-196) ---
        Real cb_before = p_a->getCashBounty();
        Object* o_cb_low = w.spawn_object("Obj_CashBounty_Low", w.team_a);
        SpecialPowerModule* sp_cb_low = find_sp_module(o_cb_low);
        Real cb_after_low_create = p_a->getCashBounty();
        ScienceType cb_req_sci = sp_cb_low->getRequiredScience();
        bool cb_sci_matches = (cb_req_sci == sp_bounty->getRequiredScience());
        Object* o_cb_high = w.spawn_object("Obj_CashBounty_High", w.team_a);
        SpecialPowerModule* sp_cb_high = find_sp_module(o_cb_high);
        Real cb_after_high_create = p_a->getCashBounty();
        if (!has_bounty_sci) {
            (p_a->*access_member(PlayerSciTag())).push_back(sci_bounty);
        }
        sp_cb_high->onSpecialPowerCreation();
        Real cb_after_sp_creation = p_a->getCashBounty();
        UnsignedInt cb_ready_pre = static_cast<const SpecialPowerModule*>(sp_cb_high)->getReadyFrame();
        sp_cb_high->doSpecialPower(0);
        UnsignedInt cb_ready_post = static_cast<const SpecialPowerModule*>(sp_cb_high)->getReadyFrame();
        std::string cb_hex = oracle_wp51::World::snapshot_save_hex(sp_cb_high);

        // --- 4) CashHackSpecialPower (PKT-197) ---
        Object* o_ch = w.spawn_object("Obj_CashHack", w.team_a);
        Object* o_ch_tgt = w.spawn_object("Obj_TargetInf", w.team_b, { loc_x + 20.0f, loc_y, 0.0f });
        SpecialPowerModule* sp_ch = find_sp_module(o_ch);
        if (disabled_flag) o_ch->setDisabled(DISABLED_EMP);
        // Location call must be ignored by CashHackSpecialPower
        sp_ch->doSpecialPowerAtLocation(&baik_loc, 0.0f, 0);
        UnsignedInt ch_ready_after_loc = static_cast<const SpecialPowerModule*>(sp_ch)->getReadyFrame();
        sp_ch->doSpecialPowerAtObject(o_ch_tgt, 0);
        UnsignedInt ch_money_a = p_a->getMoney()->countMoney();
        UnsignedInt ch_money_b = p_b->getMoney()->countMoney();
        UnsignedInt ch_ready_after_obj = static_cast<const SpecialPowerModule*>(sp_ch)->getReadyFrame();
        std::string ch_hex = oracle_wp51::World::snapshot_save_hex(sp_ch);

        // --- 5) CleanupAreaPower (PKT-198) ---
        Object* o_cln = w.spawn_object("Obj_Cleanup", w.team_a);
        SpecialPowerModule* sp_cln = find_sp_module(o_cln);
        if (disabled_flag) o_cln->setDisabled(DISABLED_EMP);
        sp_cln->doSpecialPowerAtLocation(&baik_loc, 0.0f, 0);
        UnsignedInt cln_ready = static_cast<const SpecialPowerModule*>(sp_cln)->getReadyFrame();
        std::string cln_hex = oracle_wp51::World::snapshot_save_hex(sp_cln);

        // --- 6) DefectorSpecialPower (PKT-209) ---
        Object* o_def = w.spawn_object("Obj_Defector", w.team_a);
        Object* o_def_tgt = w.spawn_object("Obj_TargetInf", w.team_b, { loc_x, loc_y + 30.0f, 0.0f });
        SpecialPowerModule* sp_def = find_sp_module(o_def);
        if (disabled_flag) o_def->setDisabled(DISABLED_EMP);
        sp_def->doSpecialPowerAtLocation(&baik_loc, 0.0f, 0);
        bool def_team_after_loc = (o_def_tgt->getTeam() == w.team_a);
        sp_def->doSpecialPowerAtObject(o_def_tgt, 0);
        bool def_team_after_obj = (o_def_tgt->getTeam() == w.team_a);
        bool def_undetected = o_def_tgt->getIsUndetectedDefector() != FALSE;
        UnsignedInt def_ready = static_cast<const SpecialPowerModule*>(sp_def)->getReadyFrame();
        std::string def_hex = oracle_wp51::World::snapshot_save_hex(sp_def);

        // --- 7) FireWeaponPower (PKT-216) ---
        Object* o_fw = w.spawn_object("Obj_FireWep", w.team_a, { 50.0f, 50.0f, 0.0f });
        SpecialPowerModule* sp_fw = find_sp_module(o_fw);
        if (disabled_flag) o_fw->setDisabled(DISABLED_EMP);
        sp_fw->doSpecialPowerAtLocation(&baik_loc, 0.0f, 0);
        UnsignedInt fw_ready = static_cast<const SpecialPowerModule*>(sp_fw)->getReadyFrame();
        std::string fw_hex = oracle_wp51::World::snapshot_save_hex(sp_fw);

        // --- 8) OCLSpecialPower (PKT-226) ---
        Object* o_ocl = w.spawn_object(ocl_above ? "Obj_OCL_Above" : "Obj_OCL_Loc", w.team_a, { 10.0f, 20.0f, 0.0f });
        SpecialPowerModule* sp_ocl = find_sp_module(o_ocl);
        const ThingTemplate* ocl_ref = sp_ocl->getReferenceThingTemplate();
        std::string ocl_ref_name = ocl_ref ? ocl_ref->getName().str() : "";
        if (ocl_reject) {
            sp_ocl->setReadyFrame(0xFFFFFFFFu);
        }
        if (disabled_flag) o_ocl->setDisabled(DISABLED_EMP);
        std::unordered_set<ObjectID> before_ids;
        for (Object* o : collect_live_objects()) {
            before_ids.insert(o->getID());
        }
        sp_ocl->doSpecialPowerAtLocation(&baik_loc, 0.0f, 0);
        std::vector<Object*> spawned_objs;
        for (Object* o : collect_live_objects()) {
            if (before_ids.find(o->getID()) == before_ids.end()) {
                spawned_objs.push_back(o);
            }
        }
        uint32_t ocl_spawned_cnt = static_cast<uint32_t>(spawned_objs.size());
        std::string ocl_spawned_name;
        Real ocl_spawned_z = -1.0f;
        if (ocl_spawned_cnt > 0) {
            Object* spawned = spawned_objs.front();
            ocl_spawned_name = spawned->getTemplate()->getName().str();
            ocl_spawned_z = spawned->getPosition()->z;
        }
        UnsignedInt ocl_ready = static_cast<const SpecialPowerModule*>(sp_ocl)->getReadyFrame();
        std::string ocl_hex = oracle_wp51::World::snapshot_save_hex(sp_ocl);

        // --- 9) SpyVisionSpecialPower (PKT-245) ---
        Object* o_sv = w.spawn_object("Obj_SpyVision", w.team_a);
        SpecialPowerModule* sp_sv = find_sp_module(o_sv);
        if (disabled_flag) o_sv->setDisabled(DISABLED_EMP);
        sp_sv->doSpecialPower(0);
        UnsignedInt sv_ready = static_cast<const SpecialPowerModule*>(sp_sv)->getReadyFrame();
        std::string sv_hex = oracle_wp51::World::snapshot_save_hex(sp_sv);

        // Cleanup all live objects before next seed
        std::vector<Object*> to_kill;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
            to_kill.push_back(o);
        }
        for (Object* o : to_kill) {
            TheGameLogic->destroyObject(o);
        }
        TheGameLogic->processDestroyList();

        if (seed == 0) {
            std::ostringstream d;
            d << "{"
              << "\"demoralize_compiled\":" << (demoralize_compiled ? 1 : 0) << ","
              << "\"sp\":{\"for_norm\":" << (sp_for_norm?1:0) << ",\"for_ready\":" << (sp_for_ready?1:0)
              << ",\"ready_init\":" << sp_ready_init << ",\"is_ready_init\":" << (sp_is_ready_init?1:0)
              << ",\"pct_mid_bits\":" << oracle::fbits(pct_mid)
              << ",\"pct_paused_bits\":" << oracle::fbits(pct_paused)
              << ",\"ready_paused\":" << ready_paused
              << ",\"still_paused\":" << (still_paused?1:0)
              << ",\"ready_unpaused\":" << ready_unpaused
              << ",\"ready_set\":" << ready_after_set
              << ",\"ready_load\":" << ready_after_load
              << ",\"rdy_starts\":" << (rdy_starts_ready?1:0)
              << ",\"rdy_f0\":" << rdy_frame0
              << ",\"rdy_is0\":" << (rdy_is_ready0?1:0)
              << ",\"pct_pau100_bits\":" << oracle::fbits(pct_paused_100)
              << ",\"pau_do\":" << pau_ready_after_do
              << ",\"scr_t\":" << (script_only_true?1:0) << ",\"scr_f\":" << (script_only_false?1:0)
              << ",\"shr_pre\":" << shr_ready_pre << ",\"shr_cr\":" << shr_ready_post_create << ",\"shr_do\":" << shr_ready_post_do
              << ",\"xfer\":\"" << sp_hex << "\",\"crc\":" << sp_crc << ",\"xcrc\":" << sp_xfer_crc << "},"
              << "\"baik\":{\"door\":" << (baik_door?1:0) << ",\"ready\":" << baik_ready
              << ",\"cnt_pre\":" << baik_cnt_pre << ",\"cnt_post\":" << baik_cnt_post << ",\"xfer\":\"" << baik_hex << "\"},"
              << "\"cb\":{\"before_bits\":" << oracle::fbits(cb_before)
              << ",\"low_bits\":" << oracle::fbits(cb_after_low_create)
              << ",\"high_bits\":" << oracle::fbits(cb_after_high_create)
              << ",\"sp_bits\":" << oracle::fbits(cb_after_sp_creation)
              << ",\"sci_match\":" << (cb_sci_matches?1:0)
              << ",\"r_pre\":" << cb_ready_pre << ",\"r_post\":" << cb_ready_post
              << ",\"xfer\":\"" << cb_hex << "\"},"
              << "\"ch\":{\"r_loc\":" << ch_ready_after_loc << ",\"ma\":" << ch_money_a << ",\"mb\":" << ch_money_b
              << ",\"r_obj\":" << ch_ready_after_obj << ",\"xfer\":\"" << ch_hex << "\"},"
              << "\"cln\":{\"ready\":" << cln_ready << ",\"xfer\":\"" << cln_hex << "\"},"
              << "\"def\":{\"loc_t\":" << (def_team_after_loc?1:0) << ",\"obj_t\":" << (def_team_after_obj?1:0)
              << ",\"undet\":" << (def_undetected?1:0) << ",\"ready\":" << def_ready << ",\"xfer\":\"" << def_hex << "\"},"
              << "\"fw\":{\"ready\":" << fw_ready << ",\"xfer\":\"" << fw_hex << "\"},"
              << "\"ocl\":{\"ref\":\"" << ocl_ref_name << "\",\"cnt\":" << ocl_spawned_cnt
              << ",\"name\":\"" << ocl_spawned_name << "\",\"z_bits\":" << oracle::fbits(ocl_spawned_z)
              << ",\"ready\":" << ocl_ready << ",\"xfer\":\"" << ocl_hex << "\"},"
              << "\"sv\":{\"ready\":" << sv_ready << ",\"xfer\":\"" << sv_hex << "\"}"
              << "}";
            s0_diag = d.str();
        }

        if (seed > 0) vecs << ",\n";
        vecs << "    {"
             << "\"seed\":" << seed
             << ",\"in\":{\"frame\":" << frame0
             << ",\"dt\":" << step_dt
             << ",\"custom_ready\":" << custom_ready
             << ",\"under_const\":" << (under_const?1:0)
             << ",\"disabled\":" << (disabled_flag?1:0)
             << ",\"has_bounty_sci\":" << (has_bounty_sci?1:0)
             << ",\"has_cash_upg\":" << (has_cash_upg?1:0)
             << ",\"has_ocl_upg\":" << (has_ocl_upg?1:0)
             << ",\"ocl_above\":" << (ocl_above?1:0)
             << ",\"ocl_reject\":" << (ocl_reject?1:0)
             << ",\"money_a\":" << money_a
             << ",\"money_b\":" << money_b
             << ",\"loc_x_bits\":" << oracle::fbits(loc_x)
             << ",\"loc_y_bits\":" << oracle::fbits(loc_y)
             << "}"
             << ",\"out\":{"
             << "\"sp\":[" << (sp_for_norm?1:0) << "," << (sp_for_ready?1:0) << "," << sp_ready_init
             << "," << (sp_is_ready_init?1:0) << "," << oracle::fbits(pct_mid)
             << "," << oracle::fbits(pct_paused) << "," << ready_paused << "," << (still_paused?1:0)
             << "," << ready_unpaused << "," << ready_after_set << "," << ready_after_load
             << "," << (rdy_starts_ready?1:0) << "," << rdy_frame0 << "," << (rdy_is_ready0?1:0)
             << "," << oracle::fbits(pct_paused_100) << "," << pau_ready_after_do
             << "," << (script_only_true?1:0) << "," << (script_only_false?1:0)
             << "," << shr_ready_pre << "," << shr_ready_post_create << "," << shr_ready_post_do << "]"
             << ",\"sp_crc\":" << sp_crc
             << ",\"sp_xcrc\":" << sp_xfer_crc
             << ",\"sp_xfer\":\"" << sp_hex << "\""
             << ",\"baik\":[" << (baik_door?1:0) << "," << baik_ready << "," << (baik_cnt_post - baik_cnt_pre) << "]"
             << ",\"baik_xfer\":\"" << baik_hex << "\""
             << ",\"cb\":[" << oracle::fbits(cb_before) << "," << oracle::fbits(cb_after_low_create)
             << "," << oracle::fbits(cb_after_high_create) << "," << oracle::fbits(cb_after_sp_creation)
             << "," << (cb_sci_matches?1:0) << "," << cb_ready_pre << "," << cb_ready_post << "]"
             << ",\"ch\":[" << ch_ready_after_loc << "," << ch_money_a << "," << ch_money_b << "," << ch_ready_after_obj << "]"
             << ",\"cln\":" << cln_ready
             << ",\"def\":[" << (def_team_after_loc?1:0) << "," << (def_team_after_obj?1:0)
             << "," << (def_undetected?1:0) << "," << def_ready << "]"
             << ",\"fw\":" << fw_ready
             << ",\"ocl\":[" << ocl_spawned_cnt << "," << oracle::fbits(ocl_spawned_z) << "," << ocl_ready << "]"
             << ",\"ocl_ref\":\"" << ocl_ref_name << "\""
             << ",\"ocl_name\":\"" << ocl_spawned_name << "\""
             << ",\"sv\":" << sv_ready
             << "}"
             << "}";
    }

    std::ostringstream doc;
    doc << "{\n"
        << "  \"packet\": \"WP-5.2-C\",\n"
        << "  \"modules\": [\"SpecialPowerModule\",\"BaikonurLaunchPower\",\"CashBountyPower\",\"CashHackSpecialPower\",\"CleanupAreaPower\",\"DefectorSpecialPower\",\"DemoralizeSpecialPower\",\"FireWeaponPower\",\"OCLSpecialPower\",\"SpecialAbility\",\"SpyVisionSpecialPower\"],\n"
        << "  \"demoralize_compiled\": " << (demoralize_compiled ? 1 : 0) << ",\n"
        << "  \"seed_count\": 1000,\n"
        << "  \"source_sha256\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/SpecialPower/SpecialPowerModule.cpp") << "\",\n"
        << "  \"seed0_diagnostics\": " << s0_diag << ",\n"
        << "  \"vectors\": [\n" << vecs.str() << "\n  ]\n}\n";

    oracle::write_file_atomic(out_path, doc.str());
    return 0;
}
