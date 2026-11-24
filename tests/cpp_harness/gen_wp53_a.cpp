// WP-5.3-A Behavior & Contain Module Foundations (9 packets:
// PKT-259 BehaviorModule, PKT-265 ContainModule, PKT-282 OpenContain,
// PKT-295 SlowDeathBehavior, PKT-276 InstantDeathBehavior, PKT-257 AutoHealBehavior,
// PKT-268 FireWeaponWhenDamagedBehavior, PKT-269 FireWeaponWhenDeadBehavior,
// PKT-273 GrantStealthBehavior)
//
// C++ Golden Vector Generator (Zero-Stub Build against oracle_wp51_world.h).
//
// Canonical build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_a.cpp --opt O2 \
//     -o /tmp/gen_wp53_a_o2
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_wp53_a_o2 docs/golden_vectors/WP53_A.json

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/RandomValue.h"
#include "Common/ThingTemplate.h"
#include "Common/Upgrade.h"
#include "GameClient/Drawable.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/AutoHealBehavior.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/FireWeaponWhenDamagedBehavior.h"
#include "GameLogic/Module/FireWeaponWhenDeadBehavior.h"
#include "GameLogic/Module/GrantStealthBehavior.h"
#include "GameLogic/Module/InstantDeathBehavior.h"
#include "GameLogic/Module/OpenContain.h"
#include "GameLogic/Module/SlowDeathBehavior.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Module/UndeadBody.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/UpgradeModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Weapon.h"

#include <sstream>
#include <string>
#include <vector>

struct SnapLoadPostTag { using type = void (Snapshot::*)(); };
template struct MemberAccess<SnapLoadPostTag, &Snapshot::loadPostProcess>;
void (Snapshot::* access_member(SnapLoadPostTag))();

struct PlayerUpgTag { using type = UpgradeMaskType Player::*; };
template struct MemberAccess<PlayerUpgTag, &Player::m_upgradesCompleted>;
UpgradeMaskType Player::* access_member(PlayerUpgTag);

struct SdSinkFrameTag { using type = UnsignedInt SlowDeathBehavior::*; };
template struct MemberAccess<SdSinkFrameTag, &SlowDeathBehavior::m_sinkFrame>;
UnsignedInt SlowDeathBehavior::* access_member(SdSinkFrameTag);

struct SdMidpointFrameTag { using type = UnsignedInt SlowDeathBehavior::*; };
template struct MemberAccess<SdMidpointFrameTag, &SlowDeathBehavior::m_midpointFrame>;
UnsignedInt SlowDeathBehavior::* access_member(SdMidpointFrameTag);

struct SdDestFrameTag { using type = UnsignedInt SlowDeathBehavior::*; };
template struct MemberAccess<SdDestFrameTag, &SlowDeathBehavior::m_destructionFrame>;
UnsignedInt SlowDeathBehavior::* access_member(SdDestFrameTag);

struct SdTimeScaleTag { using type = Real SlowDeathBehavior::*; };
template struct MemberAccess<SdTimeScaleTag, &SlowDeathBehavior::m_acceleratedTimeScale>;
Real SlowDeathBehavior::* access_member(SdTimeScaleTag);

struct SdFlagsTag { using type = UnsignedInt SlowDeathBehavior::*; };
template struct MemberAccess<SdFlagsTag, &SlowDeathBehavior::m_flags>;
UnsignedInt SlowDeathBehavior::* access_member(SdFlagsTag);

struct AhSoonestTag { using type = UnsignedInt AutoHealBehavior::*; };
template struct MemberAccess<AhSoonestTag, &AutoHealBehavior::m_soonestHealFrame>;
UnsignedInt AutoHealBehavior::* access_member(AhSoonestTag);

struct AhStoppedTag { using type = Bool AutoHealBehavior::*; };
template struct MemberAccess<AhStoppedTag, &AutoHealBehavior::m_stopped>;
Bool AutoHealBehavior::* access_member(AhStoppedTag);

struct GsRadiusTag { using type = Real GrantStealthBehavior::*; };
template struct MemberAccess<GsRadiusTag, &GrantStealthBehavior::m_currentScanRadius>;
Real GrantStealthBehavior::* access_member(GsRadiusTag);

struct FwdRPristineTag { using type = Weapon* FireWeaponWhenDamagedBehavior::*; };
template struct MemberAccess<FwdRPristineTag, &FireWeaponWhenDamagedBehavior::m_reactionWeaponPristine>;
Weapon* FireWeaponWhenDamagedBehavior::* access_member(FwdRPristineTag);

struct FwdCPristineTag { using type = Weapon* FireWeaponWhenDamagedBehavior::*; };
template struct MemberAccess<FwdCPristineTag, &FireWeaponWhenDamagedBehavior::m_continuousWeaponPristine>;
Weapon* FireWeaponWhenDamagedBehavior::* access_member(FwdCPristineTag);

struct OcDoorCountdownTag { using type = UnsignedInt OpenContain::*; };
template struct MemberAccess<OcDoorCountdownTag, &OpenContain::m_doorCloseCountdown>;
UnsignedInt OpenContain::* access_member(OcDoorCountdownTag);

struct OcWhichExitTag { using type = Int OpenContain::*; };
template struct MemberAccess<OcWhichExitTag, &OpenContain::m_whichExitPath>;
Int OpenContain::* access_member(OcWhichExitTag);

static std::vector<uint8_t> hex_to_bytes(const std::string& hex) {
    std::vector<uint8_t> out;
    out.reserve(hex.size() / 2);
    auto nib = [](char c) -> uint8_t {
        if (c >= '0' && c <= '9') return static_cast<uint8_t>(c - '0');
        if (c >= 'a' && c <= 'f') return static_cast<uint8_t>(10 + (c - 'a'));
        if (c >= 'A' && c <= 'F') return static_cast<uint8_t>(10 + (c - 'A'));
        return 0;
    };
    for (size_t i = 0; i + 1 < hex.size(); i += 2) {
        out.push_back(static_cast<uint8_t>((nib(hex[i]) << 4) | nib(hex[i + 1])));
    }
    return out;
}

template <typename T>
static T* find_behavior(Object* obj, size_t skip = 0) {
    if (!obj) return nullptr;
    size_t seen = 0;
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (T* typed = dynamic_cast<T*>(*m)) {
            if (seen == skip) return typed;
            ++seen;
        }
    }
    return nullptr;
}

static StealthUpdate* find_stealth(Object* obj) {
    if (!obj) return nullptr;
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (StealthUpdate* st = (*m)->getStealth()) return st;
    }
    return nullptr;
}

static void step_single_update(oracle_wp51::World& w, UpdateModule* u, UnsignedInt frame) {
    if (!u) return;
    const Object* obj = u->friend_getObject();
    if (!obj || obj->isDestroyed()) return;
    w.set_frame(frame);
    if (u->friend_getNextCallFrame() > frame) return;

    UpdateSleepTime sleep_len = UPDATE_SLEEP_NONE;
    DisabledMaskType dis = obj->getDisabledFlags();
    if (u->getDisabledTypesToProcess().testForAll(dis)) {
        sleep_len = u->update();
        if (sleep_len < 1) sleep_len = UPDATE_SLEEP_NONE;
    }
    UnsignedInt next = frame + static_cast<UnsignedInt>(sleep_len);
    if (next > UPDATE_SLEEP_FOREVER) next = UPDATE_SLEEP_FOREVER;
    u->friend_setNextCallFrame(next);
}

static void destroy_and_flush_all(oracle_wp51::World& w) {
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        if (!o->isDestroyed()) {
            TheGameLogic->destroyObject(o);
        }
    }
    w.flush_destroy_list();
}

static void append_u32_array(std::ostringstream& ss, const std::vector<uint32_t>& vals) {
    ss << "[";
    for (size_t i = 0; i < vals.size(); ++i) {
        if (i > 0) ss << ",";
        ss << vals[i];
    }
    ss << "]";
}

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/WP53_A.json";
    oracle_wp51::World w;
    w.init_with_neutral_player();
    if (!w.team_neutral) {
        Dict dict_n;
        TheTeamFactory->initTeam("TeamNeutral", AsciiString::TheEmptyString, TRUE, &dict_n);
        w.team_neutral = TheTeamFactory->findTeam("TeamNeutral");
        if (w.team_neutral) {
            w.team_neutral->setControllingPlayer(ThePlayerList->getNthPlayer(0));
        }
    }

    w.load_ini(
        "Upgrade UpgHeal\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"
        "Upgrade UpgOld\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 50\nEnd\n"
        "Upgrade UpgWpn\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"
        "Upgrade UpgConflict\n  Type = PLAYER\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"
        "Upgrade UpgObjConflict\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"

        "FXList FX_SD_InitA\nEnd\n"
        "FXList FX_SD_InitB\nEnd\n"
        "FXList FX_SD_MidA\nEnd\n"
        "FXList FX_SD_FinalA\nEnd\n"
        "FXList FX_ID_A\nEnd\n"
        "FXList FX_ID_B\nEnd\n"
        "FXList FX_Upg_Heal\nEnd\n"
        "FXList FX_Upg_Wpn\nEnd\n"

        "ObjectCreationList OCL_SD_A\nEnd\n"
        "ObjectCreationList OCL_SD_B\nEnd\n"
        "ObjectCreationList OCL_ID_A\nEnd\n"
        "ObjectCreationList OCL_ID_B\nEnd\n"

        "Armor SecondLifeArmor\n  Armor = Default 50%\nEnd\n"

        "Weapon Wpn_SD_A\n"
        "  PrimaryDamage = 0.0\n"
        "  PrimaryDamageRadius = 0.0\n"
        "  DamageType = EXPLOSION\n"
        "  DeathType = NORMAL\n"
        "  AutoReloadsClip = Yes\n"
        "  ShotsPerBarrel = 1\n"
        "End\n"
        "Weapon Wpn_SD_B\n"
        "  PrimaryDamage = 0.0\n"
        "  PrimaryDamageRadius = 0.0\n"
        "  DamageType = FLAME\n"
        "  DeathType = BURNED\n"
        "  AutoReloadsClip = Yes\n"
        "  ShotsPerBarrel = 1\n"
        "End\n"
        "Weapon Wpn_ReactPris\n"
        "  PrimaryDamage = 0.0\n"
        "  PrimaryDamageRadius = 0.0\n"
        "  DamageType = SMALL_ARMS\n"
        "  DeathType = NORMAL\n"
        "  ClipSize = 2\n"
        "  ClipReloadTime = 300\n"
        "  DelayBetweenShots = 100\n"
        "  AutoReloadsClip = Yes\n"
        "  ShotsPerBarrel = 1\n"
        "End\n"
        "Weapon Wpn_ReactDmg\n"
        "  PrimaryDamage = 0.0\n"
        "  PrimaryDamageRadius = 0.0\n"
        "  DamageType = SMALL_ARMS\n"
        "  DeathType = NORMAL\n"
        "  ClipSize = 3\n"
        "  ClipReloadTime = 0\n"
        "  DelayBetweenShots = 100\n"
        "  AutoReloadsClip = Yes\n"
        "  ShotsPerBarrel = 1\n"
        "End\n"
        "Weapon Wpn_ReactReal\n"
        "  PrimaryDamage = 0.0\n"
        "  PrimaryDamageRadius = 0.0\n"
        "  DamageType = SMALL_ARMS\n"
        "  DeathType = NORMAL\n"
        "  ClipSize = 1\n"
        "  ClipReloadTime = 150\n"
        "  DelayBetweenShots = 0\n"
        "  AutoReloadsClip = Yes\n"
        "  ShotsPerBarrel = 1\n"
        "End\n"
        "Weapon Wpn_ReactRubble\n"
        "  PrimaryDamage = 0.0\n"
        "  PrimaryDamageRadius = 0.0\n"
        "  DamageType = SMALL_ARMS\n"
        "  DeathType = NORMAL\n"
        "  ClipSize = 1\n"
        "  ClipReloadTime = 0\n"
        "  AutoReloadsClip = No\n"
        "  ShotsPerBarrel = 1\n"
        "End\n"
        "Weapon Wpn_ContPris\n"
        "  PrimaryDamage = 0.0\n"
        "  PrimaryDamageRadius = 0.0\n"
        "  DamageType = EXPLOSION\n"
        "  DeathType = NORMAL\n"
        "  ClipSize = 2\n"
        "  ClipReloadTime = 200\n"
        "  DelayBetweenShots = 100\n"
        "  AutoReloadsClip = Yes\n"
        "  ShotsPerBarrel = 1\n"
        "End\n"
        "Weapon Wpn_ContDmg\n"
        "  PrimaryDamage = 0.0\n"
        "  PrimaryDamageRadius = 0.0\n"
        "  DamageType = EXPLOSION\n"
        "  DeathType = NORMAL\n"
        "  ClipSize = 2\n"
        "  ClipReloadTime = 100\n"
        "  DelayBetweenShots = 100\n"
        "  AutoReloadsClip = Yes\n"
        "  ShotsPerBarrel = 1\n"
        "End\n"
        "Weapon Wpn_Death\n"
        "  PrimaryDamage = 15.0\n"
        "  PrimaryDamageRadius = 40.0\n"
        "  RadiusDamageAffects = ENEMIES\n"
        "  DamageType = EXPLOSION\n"
        "  DeathType = EXPLODED\n"
        "  AutoReloadsClip = Yes\n"
        "  ShotsPerBarrel = 1\n"
        "End\n"

        // --- OpenContain test templates ---
        "Object P53_Inf\n"
        "  KindOf = INFANTRY SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "End\n"
        "Object P53_Stealth\n"
        "  KindOf = INFANTRY STEALTH_GARRISON SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 80\n    InitialHealth = 80\n  End\n"
        "  Behavior = StealthUpdate ModuleTag_Stealth\n"
        "    StealthDelay = 100\n"
        "    InnateStealth = Yes\n"
        "  End\n"
        "End\n"
        "Object P53_Grantable\n"
        "  KindOf = INFANTRY SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 80\n    InitialHealth = 80\n  End\n"
        "  Behavior = StealthUpdate ModuleTag_Stealth\n"
        "    StealthDelay = 100\n"
        "    InnateStealth = No\n"
        "    GrantedBySpecialPower = Yes\n"
        "  End\n"
        "End\n"
        "Object P53_Hero\n"
        "  KindOf = INFANTRY HERO SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "End\n"
        "Object P53_VehAI\n"
        "  KindOf = VEHICLE SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 150\n    InitialHealth = 150\n  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "End\n"
        "Object P53_Struct\n"
        "  KindOf = STRUCTURE SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 300\n    InitialHealth = 300\n  End\n"
        "End\n"

        "Object OC_T0\n"
        "  KindOf = STRUCTURE SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 500\n    InitialHealth = 500\n  End\n"
        "  Behavior = OpenContain ModuleTag_Contain\n"
        "    ContainMax = 3\n"
        "    DamagePercentToUnits = 30%\n"
        "    BurnedDeathToUnits = Yes\n"
        "    AllowInsideKindOf = INFANTRY VEHICLE\n"
        "    ForbidInsideKindOf = HERO\n"
        "    PassengersAllowedToFire = No\n"
        "    DoorOpenTime = 100\n"
        "    NumberOfExitPaths = 2\n"
        "    WeaponBonusPassedToPassengers = Yes\n"
        "    AllowAlliesInside = Yes\n"
        "    AllowEnemiesInside = No\n"
        "    AllowNeutralInside = No\n"
        "    DeathTypes = ALL -CRUSHED\n"
        "  End\n"
        "End\n"
        "Object OC_T1\n"
        "  KindOf = VEHICLE SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 400\n    InitialHealth = 400\n  End\n"
        "  Behavior = OpenContain ModuleTag_Contain\n"
        "    ContainMax = 4\n"
        "    DamagePercentToUnits = 50%\n"
        "    BurnedDeathToUnits = No\n"
        "    PassengersAllowedToFire = Yes\n"
        "    DoorOpenTime = 200\n"
        "    AllowAlliesInside = Yes\n"
        "    AllowEnemiesInside = Yes\n"
        "    AllowNeutralInside = Yes\n"
        "  End\n"
        "End\n"

        // --- SlowDeathBehavior templates ---
        "Object SD_Single\n"
        "  KindOf = INFANTRY SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = SlowDeathBehavior ModuleTag_SD\n"
        "    DeathTypes = ALL -BURNED\n"
        "    ProbabilityModifier = 10\n"
        "    ModifierBonusPerOverkillPercent = 25%\n"
        "    SinkRate = 60.0\n"
        "    SinkDelay = 100\n"
        "    SinkDelayVariance = 100\n"
        "    DestructionDelay = 400\n"
        "    DestructionDelayVariance = 200\n"
        "    FX = INITIAL FX_SD_InitA FX_SD_InitB\n"
        "    FX = MIDPOINT FX_SD_MidA\n"
        "    FX = FINAL FX_SD_FinalA\n"
        "    OCL = INITIAL OCL_SD_A OCL_SD_B\n"
        "    OCL = FINAL OCL_SD_A\n"
        "    Weapon = INITIAL Wpn_SD_A Wpn_SD_B\n"
        "    Weapon = MIDPOINT Wpn_SD_B\n"
        "  End\n"
        "End\n"
        "Object SD_Dual_AI\n"
        "  KindOf = VEHICLE SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Behavior = SlowDeathBehavior ModuleTag_SDA\n"
        "    DeathTypes = ALL -CRUSHED\n"
        "    ProbabilityModifier = 10\n"
        "    ModifierBonusPerOverkillPercent = 10%\n"
        "    SinkRate = 30.0\n"
        "    SinkDelay = 60\n"
        "    SinkDelayVariance = 90\n"
        "    DestructionDelay = 300\n"
        "    DestructionDelayVariance = 150\n"
        "    FX = INITIAL FX_SD_InitA\n"
        "    OCL = MIDPOINT OCL_SD_B\n"
        "  End\n"
        "  Behavior = SlowDeathBehavior ModuleTag_SDB\n"
        "    DeathTypes = ALL\n"
        "    ProbabilityModifier = 20\n"
        "    ModifierBonusPerOverkillPercent = 50%\n"
        "    SinkRate = 45.0\n"
        "    SinkDelay = 100\n"
        "    SinkDelayVariance = 100\n"
        "    DestructionDelay = 360\n"
        "    DestructionDelayVariance = 120\n"
        "    Weapon = FINAL Wpn_SD_B\n"
        "  End\n"
        "End\n"
        "Object SD_Undead\n"
        "  KindOf = VEHICLE SELECTABLE\n"
        "  ArmorSet\n    Conditions = SECOND_LIFE\n    Armor = SecondLifeArmor\n  End\n"
        "  Body = UndeadBody ModuleTag_Body\n"
        "    MaxHealth = 100\n    InitialHealth = 100\n    SecondLifeMaxHealth = 60\n"
        "  End\n"
        "  Behavior = SlowDeathBehavior ModuleTag_SDA\n"
        "    ProbabilityModifier = 12\n"
        "    ModifierBonusPerOverkillPercent = 20%\n"
        "    SinkRate = 30.0\n"
        "    SinkDelay = 90\n"
        "    DestructionDelay = 300\n"
        "    FX = INITIAL FX_SD_InitA\n"
        "  End\n"
        "  Behavior = SlowDeathBehavior ModuleTag_SDB\n"
        "    ProbabilityModifier = 18\n"
        "    SinkDelay = 120\n"
        "    DestructionDelay = 360\n"
        "    OCL = FINAL OCL_SD_A\n"
        "  End\n"
        "End\n"
        "Object SD_Fling\n"
        "  KindOf = INFANTRY SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = SlowDeathBehavior ModuleTag_SD\n"
        "    ProbabilityModifier = 10\n"
        "    SinkRate = 30.0\n"
        "    SinkDelay = 90\n"
        "    DestructionDelay = 300\n"
        "    FlingForce = 5.0\n"
        "    FlingForceVariance = 2.0\n"
        "    FlingPitch = 30.0\n"
        "    FlingPitchVariance = 15.0\n"
        "  End\n"
        "End\n"

        // --- InstantDeathBehavior template ---
        "Object ID_Multi\n"
        "  KindOf = VEHICLE SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = InstantDeathBehavior ModuleTag_ID\n"
        "    DeathTypes = ALL -CRUSHED\n"
        "    FX = FX_ID_A FX_ID_B\n"
        "    OCL = OCL_ID_A OCL_ID_B\n"
        "    Weapon = Wpn_SD_A Wpn_SD_B\n"
        "  End\n"
        "End\n"

        // --- AutoHealBehavior templates ---
        "Object AH_Self\n"
        "  KindOf = VEHICLE SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "  Behavior = AutoHealBehavior ModuleTag_AH\n"
        "    StartsActive = No\n"
        "    TriggeredBy = UpgHeal\n"
        "    RemovesUpgrades = UpgOld\n"
        "    FXListUpgrade = FX_Upg_Heal\n"
        "    HealingAmount = 15\n"
        "    HealingDelay = 200\n"
        "    StartHealingDelay = 100\n"
        "    Radius = 0.0\n"
        "  End\n"
        "End\n"
        "Object AH_RadiusCont\n"
        "  KindOf = INFANTRY SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "  Behavior = AutoHealBehavior ModuleTag_AH\n"
        "    StartsActive = Yes\n"
        "    SingleBurst = No\n"
        "    HealingAmount = 20\n"
        "    HealingDelay = 100\n"
        "    Radius = 100.0\n"
        "    KindOf = INFANTRY VEHICLE\n"
        "    ForbiddenKindOf = STRUCTURE\n"
        "    SkipSelfForHealing = No\n"
        "  End\n"
        "End\n"
        "Object AH_RadiusBurst\n"
        "  KindOf = INFANTRY SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "  Behavior = AutoHealBehavior ModuleTag_AH\n"
        "    StartsActive = Yes\n"
        "    SingleBurst = Yes\n"
        "    HealingAmount = 25\n"
        "    HealingDelay = 100\n"
        "    Radius = 100.0\n"
        "    KindOf = INFANTRY VEHICLE\n"
        "    ForbiddenKindOf = STRUCTURE\n"
        "    SkipSelfForHealing = Yes\n"
        "  End\n"
        "End\n"
        "Object AH_WholePlayer\n"
        "  KindOf = STRUCTURE SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "  Behavior = AutoHealBehavior ModuleTag_AH\n"
        "    StartsActive = Yes\n"
        "    AffectsWholePlayer = Yes\n"
        "    SkipSelfForHealing = Yes\n"
        "    HealingAmount = 12\n"
        "    HealingDelay = 150\n"
        "    KindOf = INFANTRY VEHICLE\n"
        "    ForbiddenKindOf = HERO\n"
        "  End\n"
        "End\n"

        // --- FireWeaponWhenDamagedBehavior templates ---
        "Object FWD_Test\n"
        "  KindOf = VEHICLE SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = FireWeaponWhenDamagedBehavior ModuleTag_FWD\n"
        "    StartsActive = No\n"
        "    TriggeredBy = UpgWpn\n"
        "    FXListUpgrade = FX_Upg_Wpn\n"
        "    DamageTypes = ALL -EXPLOSION\n"
        "    DamageAmount = 10.0\n"
        "    ReactionWeaponPristine = Wpn_ReactPris\n"
        "    ReactionWeaponDamaged = Wpn_ReactDmg\n"
        "    ReactionWeaponReallyDamaged = Wpn_ReactReal\n"
        "    ReactionWeaponRubble = Wpn_ReactRubble\n"
        "    ContinuousWeaponPristine = Wpn_ContPris\n"
        "    ContinuousWeaponDamaged = Wpn_ContDmg\n"
        "  End\n"
        "End\n"
        "Object FWD_Active\n"
        "  KindOf = VEHICLE SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = FireWeaponWhenDamagedBehavior ModuleTag_FWD\n"
        "    StartsActive = Yes\n"
        "    DamageTypes = ALL\n"
        "    DamageAmount = 5.0\n"
        "    ReactionWeaponPristine = Wpn_ReactDmg\n"
        "    ContinuousWeaponPristine = Wpn_ContDmg\n"
        "  End\n"
        "End\n"

        // --- FireWeaponWhenDeadBehavior templates ---
        "Object FWDead_Test\n"
        "  KindOf = STRUCTURE SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = FireWeaponWhenDeadBehavior ModuleTag_FWDead\n"
        "    StartsActive = No\n"
        "    TriggeredBy = UpgWpn\n"
        "    ConflictsWith = UpgConflict UpgObjConflict\n"
        "    DeathTypes = ALL -SUICIDED\n"
        "    DeathWeapon = Wpn_Death\n"
        "  End\n"
        "End\n"

        // --- GrantStealthBehavior template ---
        "Object GS_Emitter\n"
        "  KindOf = STRUCTURE SELECTABLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = GrantStealthBehavior ModuleTag_GS\n"
        "    StartRadius = 10.0\n"
        "    FinalRadius = 70.0\n"
        "    RadiusGrowRate = 25.0\n"
        "    KindOf = INFANTRY VEHICLE\n"
        "  End\n"
        "End\n"
    );

    Player* p_a = w.team_a->getControllingPlayer();
    Player* p_b = w.team_b->getControllingPlayer();
    const UpgradeTemplate* ut_heal = TheUpgradeCenter->findUpgrade("UpgHeal");
    const UpgradeTemplate* ut_old = TheUpgradeCenter->findUpgrade("UpgOld");
    const UpgradeTemplate* ut_wpn = TheUpgradeCenter->findUpgrade("UpgWpn");
    const UpgradeTemplate* ut_conf = TheUpgradeCenter->findUpgrade("UpgConflict");
    const UpgradeTemplate* ut_obj_conf = TheUpgradeCenter->findUpgrade("UpgObjConflict");

    std::string s0_diag;
    std::ostringstream vecs;

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        destroy_and_flush_all(w);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        p_a->*access_member(PlayerUpgTag()) = UpgradeMaskType();
        p_b->*access_member(PlayerUpgTag()) = UpgradeMaskType();

        uint32_t rng = seed ^ 0x53a0beefu;
        if (rng == 0) rng = 0x2468ace1u;

        UnsignedInt frame0 = 10u + (oracle::splitmix32(rng) % 200u);
        uint32_t logic_seed = 0x100000u + (oracle::splitmix32(rng) & 0x7fffffu);
        w.set_frame(frame0);
        InitRandom(logic_seed);

        // =========================================================================
        // 1) OpenContain (PKT-282)
        // =========================================================================
        bool oc_use_t1 = (seed & 1u) != 0;
        bool oc_expose = (seed % 3u) == 0;
        int oc_action = static_cast<int>(seed % 4u); // 0=remove, 1=harm_exit, 2=kill_all, 3=on_die
        bool oc_allow_fire_set = (seed % 5u) == 0;
        bool oc_wanter_kill = (seed % 2u) == 0;

        Object* oc_obj = w.spawn_object(oc_use_t1 ? "OC_T1" : "OC_T0", w.team_a, {100.0f, 100.0f, 0.0f}, 0.0f);
        Object* p_inf = w.spawn_object("P53_Inf", w.team_a, {100.0f, 100.0f, 0.0f}, 0.0f);
        Object* p_st = w.spawn_object("P53_Stealth", w.team_b, {100.0f, 100.0f, 0.0f}, 0.0f);
        Object* p_hero = w.spawn_object("P53_Hero", w.team_a, {100.0f, 100.0f, 0.0f}, 0.0f);
        Object* p_neu = w.spawn_object("P53_VehAI", w.team_neutral, {100.0f, 100.0f, 0.0f}, 0.0f);
        Object* p_ally_st = w.spawn_object("P53_Stealth", w.team_a, {100.0f, 100.0f, 0.0f}, 0.0f);

        ContainModuleInterface* oc_iface = oc_obj->getContain();
        OpenContain* oc = oc_iface->asOpenContain();

        int valid_mask = 0;
        if (oc->isValidContainerFor(p_inf, TRUE)) valid_mask |= 1;
        if (oc->isValidContainerFor(p_st, TRUE)) valid_mask |= 2;
        if (oc->isValidContainerFor(p_hero, TRUE)) valid_mask |= 4;
        if (oc->isValidContainerFor(p_neu, TRUE)) valid_mask |= 8;
        if (oc->isValidContainerFor(p_ally_st, TRUE)) valid_mask |= 16;

        oc->addToContain(p_inf);
        oc->addToContain(p_ally_st);
        if (oc_use_t1) {
            oc->addToContain(p_st);
            oc->addToContain(p_neu);
        }
        oc->onObjectWantsToEnterOrExit(p_hero, WANTS_TO_ENTER);
        oc->onObjectWantsToEnterOrExit(p_neu, WANTS_TO_EXIT);
        if (oc_allow_fire_set) {
            oc->setPassengerAllowedToFire(TRUE);
        }
        Coord3D rally{150.0f, 200.0f, 0.0f};
        oc->setRallyPoint(&rally);
        Coord3D nat_rally{};
        oc->getNaturalRallyPoint(nat_rally, TRUE);

        uint32_t oc_pre_count = oc->getContainCount();
        uint32_t oc_pre_stealth = oc->getStealthUnitsContained();
        uint32_t oc_pre_hero = oc->getHeroUnitsContained();
        uint32_t oc_pre_player_mask = oc->getPlayerWhoEntered();
        std::vector<uint32_t> oc_pre_ids;
        for (Object* it : *oc->getContainedItemsList()) {
            oc_pre_ids.push_back(static_cast<uint32_t>(it->getID()));
        }
        uint32_t oc_pre_crc = oracle_wp51::World::snapshot_crc(oc);
        std::string oc_pre_hex = oracle_wp51::World::snapshot_save_hex(oc);

        // Step one update frame (clears playerEnteredMask, prunes dead wanters)
        if (oc_wanter_kill) {
            p_hero->kill();
        }
        step_single_update(w, oc, frame0 + 1u);
        uint32_t oc_mid_player_mask = oc->getPlayerWhoEntered();
        bool oc_mid_wanters = (oc->hasObjectsWantingToEnterOrExit() != FALSE);

        // Execute exit/damage/kill/die action
        if (oc_action == 0) {
            oc->markAllPassengersDetected();
            oc->removeFromContain(p_ally_st, oc_expose ? TRUE : FALSE);
            oc->removeAllContained(oc_expose ? TRUE : FALSE);
        } else if (oc_action == 1) {
            DamageInfo hdmg;
            hdmg.in.m_damageType = DAMAGE_UNRESISTABLE;
            hdmg.in.m_deathType = DEATH_EXPLODED;
            hdmg.in.m_sourceID = oc_obj->getID();
            hdmg.in.m_amount = 35.0f;
            oc->harmAndForceExitAllContained(&hdmg);
        } else if (oc_action == 2) {
            oc->killAllContained();
        } else {
            DamageInfo ddmg;
            ddmg.in.m_damageType = (seed % 8u == 7u) ? DAMAGE_CRUSH : DAMAGE_EXPLOSION;
            ddmg.in.m_deathType = (seed % 8u == 7u) ? DEATH_CRUSHED : DEATH_NORMAL;
            ddmg.in.m_amount = 1000.0f;
            oc_obj->attemptDamage(&ddmg);
        }

        uint32_t oc_post_count = oc->getContainCount();
        uint32_t oc_post_stealth = oc->getStealthUnitsContained();
        uint32_t oc_post_hero = oc->getHeroUnitsContained();
        uint32_t inf_hp_bits = oracle::fbits(p_inf->getBodyModule()->getHealth());
        bool inf_contained = (p_inf->getContainedBy() != nullptr);
        uint32_t oc_post_crc = oracle_wp51::World::snapshot_crc(oc);
        std::string oc_post_hex = oracle_wp51::World::snapshot_save_hex(oc);

        // Verify XferLoad round-trip on a fresh OC container using oc_post_hex
        Object* oc_rt = w.spawn_object(oc_use_t1 ? "OC_T1" : "OC_T0", w.team_a, {100.0f, 100.0f, 0.0f}, 0.0f);
        OpenContain* oc_rt_mod = oc_rt->getContain()->asOpenContain();
        oracle_wp51::World::snapshot_load_bytes(oc_rt_mod, hex_to_bytes(oc_post_hex));
        (static_cast<Snapshot*>(oc_rt_mod)->*access_member(SnapLoadPostTag()))();
        std::string oc_rt_hex = oracle_wp51::World::snapshot_save_hex(oc_rt_mod);

        // Also test OpenContain::onCollide with AI enter target on another container
        Object* oc_col = w.spawn_object("OC_T0", w.team_a, {120.0f, 100.0f, 0.0f}, 0.0f);
        Object* oc_col_enemy = w.spawn_object("P53_Inf", w.team_b, {120.0f, 100.0f, 0.0f}, 0.0f);
        Object* oc_col_veh = w.spawn_object("P53_VehAI", w.team_a, {120.0f, 100.0f, 0.0f}, 0.0f);
        OpenContain* oc_col_mod = oc_col->getContain()->asOpenContain();
        oc_col_mod->addToContain(oc_col_enemy);
        if (AIUpdateInterface* vai = oc_col_veh->getAI()) {
            vai->aiEnter(oc_col, CMD_FROM_AI);
            step_single_update(w, vai, frame0);
        }
        oc_col_mod->onCollide(oc_col_veh, nullptr, nullptr);
        uint32_t oc_col_cnt = oc_col_mod->getContainCount();
        bool oc_col_has_veh = (oc_col_mod->isContained(oc_col_veh) != FALSE);

        destroy_and_flush_all(w);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame0);

        // =========================================================================
        // 2) SlowDeathBehavior (PKT-295)
        // =========================================================================
        int sd_mode = static_cast<int>(seed % 4u); // 0=SD_Single, 1=SD_Dual_AI, 2=SD_Undead, 3=SD_Fling
        const char* sd_tmpl = (sd_mode == 0) ? "SD_Single" :
                              (sd_mode == 1) ? "SD_Dual_AI" :
                              (sd_mode == 2) ? "SD_Undead" : "SD_Fling";
        Real sd_dmg_amt = 100.0f + static_cast<Real>((oracle::splitmix32(rng) % 160u));
        bool sd_burned = (seed % 7u == 0);

        Object* sd_obj = w.spawn_object(sd_tmpl, w.team_a, {200.0f, 200.0f, 5.0f}, 0.0f);
        // Isolate SlowDeathBehavior RNG from unported WP-5.2 AIUpdateInterface spawn RNG on SD_Dual_AI.
        InitRandom(logic_seed);
        SlowDeathBehavior* sd0 = find_behavior<SlowDeathBehavior>(sd_obj, 0);
        SlowDeathBehavior* sd1 = find_behavior<SlowDeathBehavior>(sd_obj, 1);

        DamageInfo sd_dmg;
        sd_dmg.in.m_damageType = DAMAGE_UNRESISTABLE;
        sd_dmg.in.m_deathType = sd_burned ? DEATH_BURNED : DEATH_NORMAL;
        sd_dmg.in.m_amount = sd_dmg_amt;
        sd_dmg.out.m_actualDamageDealt = sd_dmg_amt;
        sd_dmg.out.m_actualDamageClipped = 100.0f;

        Int sd0_prob = sd0->getProbabilityModifier(&sd_dmg);
        bool sd0_app = (sd0->isDieApplicable(&sd_dmg) != FALSE);
        Int sd1_prob = sd1 ? sd1->getProbabilityModifier(&sd_dmg) : 0;

        DamageInfo sd_kill;
        sd_kill.in.m_damageType = (sd_mode == 2) ? DAMAGE_EXPLOSION : DAMAGE_UNRESISTABLE;
        sd_kill.in.m_deathType = sd_burned ? DEATH_BURNED : DEATH_NORMAL;
        sd_kill.in.m_amount = sd_dmg_amt;
        sd_obj->attemptDamage(&sd_kill);
        UnsignedInt sd_post_die_rng_crc = GetGameLogicRandomSeedCRC();

        uint32_t sd0_sink = sd0->*access_member(SdSinkFrameTag());
        uint32_t sd0_mid = sd0->*access_member(SdMidpointFrameTag());
        uint32_t sd0_dest = sd0->*access_member(SdDestFrameTag());
        uint32_t sd0_flags = sd0->*access_member(SdFlagsTag());
        uint32_t sd0_pri = sd0->friend_getPriority();
        uint32_t sd1_flags = sd1 ? (sd1->*access_member(SdFlagsTag())) : 0u;
        uint32_t sd1_pri = sd1 ? sd1->friend_getPriority() : 0u;
        uint32_t sd0_crc = oracle_wp51::World::snapshot_crc(sd0);
        std::string sd0_hex = oracle_wp51::World::snapshot_save_hex(sd0);

        // Step frames up to 25 ticks to observe sink, midpoint, destruction
        uint32_t sd_destroyed_frame = 0;
        for (UnsignedInt k = 1; k <= 25; ++k) {
            UnsignedInt f = frame0 + k;
            step_single_update(w, sd0, f);
            if (sd1) step_single_update(w, sd1, f);
            if (sd_obj->isDestroyed() && sd_destroyed_frame == 0) {
                sd_destroyed_frame = f;
            }
            w.flush_destroy_list();
            if (sd_destroyed_frame != 0) break;
        }
        Real sd_pos_z = (sd_destroyed_frame == 0) ? sd_obj->getPosition()->z : 0.0f;
        uint32_t sd0_flags_post = (sd_destroyed_frame == 0) ? (sd0->*access_member(SdFlagsTag())) : sd0_flags;
        UnsignedInt sd_step_rng_crc = GetGameLogicRandomSeedCRC();

        destroy_and_flush_all(w);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame0);
        InitRandom(logic_seed);

        // =========================================================================
        // 3) InstantDeathBehavior (PKT-276)
        // =========================================================================
        bool id_under_const = (seed % 3u == 0);
        bool id_crushed = (seed % 5u == 0);
        ObjectStatusMaskType id_status = id_under_const
            ? MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION)
            : OBJECT_STATUS_NONE;
        const ThingTemplate* id_tt = TheThingFactory->findTemplate("ID_Multi");
        Coord3D id_pos{50.0f, 60.0f, 0.0f};
        Object* id_obj = TheThingFactory->newObject(id_tt, w.team_a, id_status);
        id_obj->setPosition(&id_pos);
        InstantDeathBehavior* id_mod = find_behavior<InstantDeathBehavior>(id_obj);
        std::string id_pre_hex = oracle_wp51::World::snapshot_save_hex(id_mod);
        uint32_t id_crc = oracle_wp51::World::snapshot_crc(id_mod);

        DamageInfo id_dmg;
        id_dmg.in.m_damageType = DAMAGE_UNRESISTABLE;
        id_dmg.in.m_deathType = id_crushed ? DEATH_CRUSHED : DEATH_NORMAL;
        id_dmg.in.m_amount = 200.0f;
        id_obj->attemptDamage(&id_dmg);
        bool id_destroyed = (id_obj->isDestroyed() != FALSE);
        UnsignedInt id_post_rng_crc = GetGameLogicRandomSeedCRC();

        destroy_and_flush_all(w);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame0);
        InitRandom(logic_seed);

        // =========================================================================
        // 4) AutoHealBehavior (PKT-257)
        // =========================================================================
        int ah_mode = static_cast<int>(seed % 4u); // 0=AH_Self, 1=AH_RadiusCont, 2=AH_RadiusBurst, 3=AH_WholePlayer
        const char* ah_tmpl = (ah_mode == 0) ? "AH_Self" :
                              (ah_mode == 1) ? "AH_RadiusCont" :
                              (ah_mode == 2) ? "AH_RadiusBurst" : "AH_WholePlayer";
        Object* ah_obj = w.spawn_object(ah_tmpl, w.team_a, {300.0f, 300.0f, 0.0f}, 0.0f);
        AutoHealBehavior* ah_mod = find_behavior<AutoHealBehavior>(ah_obj);
        UnsignedInt ah_init_rng_crc = GetGameLogicRandomSeedCRC();
        uint32_t ah_init_pri = ah_mod->friend_getPriority();

        Object* ah_ally = w.spawn_object("P53_Inf", w.team_a, {320.0f, 300.0f, 0.0f}, 0.0f);
        Object* ah_enemy = w.spawn_object("P53_Inf", w.team_b, {320.0f, 300.0f, 0.0f}, 0.0f);
        Object* ah_forbid = w.spawn_object((ah_mode == 3) ? "P53_Hero" : "P53_Struct", w.team_a, {320.0f, 300.0f, 0.0f}, 0.0f);
        ThePartitionManager->update();

        auto hurt = [](Object* o, Real amt) {
            DamageInfo d;
            d.in.m_damageType = DAMAGE_UNRESISTABLE;
            d.in.m_amount = amt;
            o->attemptDamage(&d);
        };
        hurt(ah_obj, 60.0f);
        hurt(ah_ally, 50.0f);
        hurt(ah_enemy, 50.0f);
        hurt(ah_forbid, 50.0f);

        if (ah_mode == 0) {
            ah_obj->giveUpgrade(ut_old);
            ah_obj->giveUpgrade(ut_heal);
            // Trigger upgrade modules
            for (BehaviorModule** m = ah_obj->getBehaviorModules(); m && *m; ++m) {
                if (UpgradeModuleInterface* u = (*m)->getUpgrade()) {
                    u->attemptUpgrade(ut_heal->getUpgradeMask());
                }
            }
            step_single_update(w, ah_mod, frame0 + 1u);
            hurt(ah_obj, 10.0f); // test onDamage resetting wake frame to StartHealingDelay
        }
        bool ah_old_removed = ah_obj->hasUpgrade(ut_old) != FALSE;

        for (UnsignedInt k = 1; k <= 10; ++k) {
            step_single_update(w, ah_mod, frame0 + k);
        }
        if (seed % 6u == 0) {
            ah_mod->stopHealing();
        } else if (seed % 6u == 3) {
            ah_mod->undoUpgrade();
        }
        Real ah_self_hp = ah_obj->getBodyModule()->getHealth();
        Real ah_ally_hp = ah_ally->getBodyModule()->getHealth();
        Real ah_enemy_hp = ah_enemy->getBodyModule()->getHealth();
        Real ah_forbid_hp = ah_forbid->getBodyModule()->getHealth();
        uint32_t ah_soonest = ah_mod->*access_member(AhSoonestTag());
        bool ah_stopped = (ah_mod->*access_member(AhStoppedTag())) != FALSE;
        uint32_t ah_post_pri = ah_mod->friend_getPriority();
        uint32_t ah_crc = oracle_wp51::World::snapshot_crc(ah_mod);
        std::string ah_hex = oracle_wp51::World::snapshot_save_hex(ah_mod);

        destroy_and_flush_all(w);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame0);
        InitRandom(logic_seed);

        // =========================================================================
        // 5) FireWeaponWhenDamagedBehavior (PKT-268)
        // =========================================================================
        bool fwd_starts_active = (seed & 1u) != 0;
        bool fwd_upg = !fwd_starts_active && ((seed & 2u) != 0);
        bool fwd_excluded_dmg = (seed % 4u) == 0;
        Real fwd_dmg_amt = (seed % 3u == 0) ? 5.0f : 35.0f;

        Object* fwd_obj = w.spawn_object(fwd_starts_active ? "FWD_Active" : "FWD_Test", w.team_a, {400.0f, 400.0f, 0.0f}, 0.0f);
        FireWeaponWhenDamagedBehavior* fwd_mod = find_behavior<FireWeaponWhenDamagedBehavior>(fwd_obj);
        uint32_t fwd_init_pri = fwd_mod->friend_getPriority();
        if (fwd_upg) {
            fwd_obj->giveUpgrade(ut_wpn);
            for (BehaviorModule** m = fwd_obj->getBehaviorModules(); m && *m; ++m) {
                if (UpgradeModuleInterface* u = (*m)->getUpgrade()) {
                    u->attemptUpgrade(ut_wpn->getUpgradeMask());
                }
            }
        }
        // Advance past ClipReloadTime (300ms = 9 frames) then damage across body states
        for (UnsignedInt k = 1; k <= 12; ++k) {
            step_single_update(w, fwd_mod, frame0 + k);
        }
        w.set_frame(frame0 + 12u);
        DamageInfo fdmg;
        fdmg.in.m_damageType = fwd_excluded_dmg ? DAMAGE_EXPLOSION : DAMAGE_SMALL_ARMS;
        fdmg.in.m_amount = fwd_dmg_amt;
        fwd_obj->attemptDamage(&fdmg);
        if (!fwd_obj->isDestroyed()) {
            step_single_update(w, fwd_mod, frame0 + 13u);
            // Second hit to cross into DAMAGED / REALLY_DAMAGED when allowed
            w.set_frame(frame0 + 15u);
            DamageInfo fdmg2;
            fdmg2.in.m_damageType = DAMAGE_SMALL_ARMS;
            fdmg2.in.m_amount = 40.0f;
            fwd_obj->attemptDamage(&fdmg2);
            step_single_update(w, fwd_mod, frame0 + 17u);
        }
        uint32_t fwd_post_pri = fwd_mod->friend_getPriority();
        uint32_t fwd_crc = oracle_wp51::World::snapshot_crc(fwd_mod);
        std::string fwd_hex = oracle_wp51::World::snapshot_save_hex(fwd_mod);

        destroy_and_flush_all(w);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame0);
        InitRandom(logic_seed);

        // =========================================================================
        // 6) FireWeaponWhenDeadBehavior (PKT-269)
        // =========================================================================
        bool fwd2_give_upg = (seed % 5u) != 0;
        bool fwd2_under_const = (seed % 7u) == 0;
        bool fwd2_obj_conflict = (seed % 11u) == 0;
        bool fwd2_ply_conflict = (seed % 13u) == 0;
        bool fwd2_suicided = (seed % 9u) == 0;

        ObjectStatusMaskType f2_status = fwd2_under_const
            ? MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION)
            : OBJECT_STATUS_NONE;
        const ThingTemplate* f2_tt = TheThingFactory->findTemplate("FWDead_Test");
        Object* f2_obj = TheThingFactory->newObject(f2_tt, w.team_a, f2_status);
        Coord3D f2_pos{450.0f, 450.0f, 0.0f};
        f2_obj->setPosition(&f2_pos);
        FireWeaponWhenDeadBehavior* f2_mod = find_behavior<FireWeaponWhenDeadBehavior>(f2_obj);
        if (fwd2_give_upg) {
            f2_obj->giveUpgrade(ut_wpn);
            for (BehaviorModule** m = f2_obj->getBehaviorModules(); m && *m; ++m) {
                if (UpgradeModuleInterface* u = (*m)->getUpgrade()) {
                    u->attemptUpgrade(ut_wpn->getUpgradeMask());
                }
            }
        }
        if (fwd2_obj_conflict) {
            f2_obj->giveUpgrade(ut_obj_conf);
        }
        if (fwd2_ply_conflict) {
            (p_a->*access_member(PlayerUpgTag())).set(ut_conf->getUpgradeMask());
        }
        uint32_t f2_crc = oracle_wp51::World::snapshot_crc(f2_mod);
        std::string f2_hex = oracle_wp51::World::snapshot_save_hex(f2_mod);

        Object* f2_target = w.spawn_object("P53_Inf", w.team_b, {460.0f, 450.0f, 0.0f}, 0.0f);
        ThePartitionManager->update();
        DamageInfo f2_dmg;
        f2_dmg.in.m_damageType = DAMAGE_UNRESISTABLE;
        f2_dmg.in.m_deathType = fwd2_suicided ? DEATH_SUICIDED : DEATH_NORMAL;
        f2_dmg.in.m_amount = 500.0f;
        f2_obj->attemptDamage(&f2_dmg);
        bool f2_fired = (f2_target->getBodyModule()->getHealth() < 100.0f);

        destroy_and_flush_all(w);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame0);
        InitRandom(logic_seed);

        // =========================================================================
        // 7) GrantStealthBehavior (PKT-273)
        // =========================================================================
        Object* gs_obj = w.spawn_object("GS_Emitter", w.team_a, {10.0f, 10.0f, 0.0f}, 0.0f);
        Object* gs_near = w.spawn_object("P53_Grantable", w.team_a, {30.0f, 10.0f, 0.0f}, 0.0f);    // dist 20
        Object* gs_mid = w.spawn_object("P53_Grantable", w.team_a, {65.0f, 10.0f, 0.0f}, 0.0f);     // dist 55
        Object* gs_no_st = w.spawn_object("P53_Inf", w.team_a, {25.0f, 10.0f, 0.0f}, 0.0f);         // no StealthUpdate
        Object* gs_enemy = w.spawn_object("P53_Grantable", w.team_b, {20.0f, 10.0f, 0.0f}, 0.0f);   // enemy
        Object* gs_offmap = w.spawn_object("P53_Grantable", w.team_a, {-5.0f, -5.0f, 0.0f}, 0.0f);  // off-map!
        ThePartitionManager->update();

        GrantStealthBehavior* gs_mod = find_behavior<GrantStealthBehavior>(gs_obj);
        uint32_t gs_init_pri = gs_mod->friend_getPriority();

        // Step 1: radius 10 + 25 = 35 -> hits gs_near (20), misses gs_mid (55)
        step_single_update(w, gs_mod, frame0 + 1u);
        bool gs_s1_near = (gs_near->getStatusBits().test(OBJECT_STATUS_CAN_STEALTH) != FALSE);
        bool gs_s1_mid = (gs_mid->getStatusBits().test(OBJECT_STATUS_CAN_STEALTH) != FALSE);
        bool gs_s1_nost = (gs_no_st->getStatusBits().test(OBJECT_STATUS_CAN_STEALTH) != FALSE);
        bool gs_s1_enemy = (gs_enemy->getStatusBits().test(OBJECT_STATUS_CAN_STEALTH) != FALSE);
        bool gs_s1_offmap = (gs_offmap->getStatusBits().test(OBJECT_STATUS_CAN_STEALTH) != FALSE);
        Real gs_s1_rad = gs_mod->*access_member(GsRadiusTag());
        uint32_t gs_crc = oracle_wp51::World::snapshot_crc(gs_mod);
        std::string gs_hex = oracle_wp51::World::snapshot_save_hex(gs_mod);

        // Step 2: radius 35 + 25 = 60 -> hits gs_mid (55)
        step_single_update(w, gs_mod, frame0 + 2u);
        bool gs_s2_mid = (gs_mid->getStatusBits().test(OBJECT_STATUS_CAN_STEALTH) != FALSE);
        bool gs_s2_dest = (gs_obj->isDestroyed() != FALSE);

        // Step 3: radius 60 + 25 = 85 -> clamps 70, destroys gs_obj
        step_single_update(w, gs_mod, frame0 + 3u);
        bool gs_s3_dest = (gs_obj->isDestroyed() != FALSE);
        Real gs_s3_rad = gs_mod->*access_member(GsRadiusTag());

        destroy_and_flush_all(w);

        if (seed == 0) {
            std::ostringstream d;
            d << "{"
              << "\"oc_pre_count\":" << oc_pre_count
              << ",\"oc_pre_stealth\":" << oc_pre_stealth
              << ",\"oc_pre_hero\":" << oc_pre_hero
              << ",\"oc_pre_player_mask\":" << oc_pre_player_mask
              << ",\"oc_mid_player_mask\":" << oc_mid_player_mask
              << ",\"oc_mid_wanters\":" << (oc_mid_wanters ? 1 : 0)
              << ",\"oc_post_count\":" << oc_post_count
              << ",\"oc_rt_match\":" << ((oc_rt_hex == oc_post_hex) ? 1 : 0)
              << ",\"sd0_sink\":" << sd0_sink
              << ",\"sd0_mid\":" << sd0_mid
              << ",\"sd0_dest\":" << sd0_dest
              << ",\"sd0_flags\":" << sd0_flags
              << ",\"sd0_pri\":" << sd0_pri
              << ",\"sd_destroyed_frame\":" << sd_destroyed_frame
              << ",\"id_destroyed\":" << (id_destroyed ? 1 : 0)
              << ",\"ah_init_pri\":" << ah_init_pri
              << ",\"ah_self_hp_bits\":" << oracle::fbits(ah_self_hp)
              << ",\"fwd_init_pri\":" << fwd_init_pri
              << ",\"fwd_post_pri\":" << fwd_post_pri
              << ",\"gs_s1_near\":" << (gs_s1_near ? 1 : 0)
              << ",\"gs_s1_mid\":" << (gs_s1_mid ? 1 : 0)
              << ",\"gs_s2_mid\":" << (gs_s2_mid ? 1 : 0)
              << ",\"gs_s3_dest\":" << (gs_s3_dest ? 1 : 0)
              << "}";
            s0_diag = d.str();
        }

        if (seed > 0) vecs << ",\n";
        vecs << "    {"
             << "\"seed\":" << seed
             << ",\"in\":{"
             << "\"frame\":" << frame0
             << ",\"logic_seed\":" << logic_seed
             << ",\"oc_t1\":" << (oc_use_t1 ? 1 : 0)
             << ",\"oc_expose\":" << (oc_expose ? 1 : 0)
             << ",\"oc_action\":" << oc_action
             << ",\"oc_allow_fire\":" << (oc_allow_fire_set ? 1 : 0)
             << ",\"oc_wanter_kill\":" << (oc_wanter_kill ? 1 : 0)
             << ",\"sd_mode\":" << sd_mode
             << ",\"sd_dmg_bits\":" << oracle::fbits(sd_dmg_amt)
             << ",\"sd_burned\":" << (sd_burned ? 1 : 0)
             << ",\"id_uc\":" << (id_under_const ? 1 : 0)
             << ",\"id_crushed\":" << (id_crushed ? 1 : 0)
             << ",\"ah_mode\":" << ah_mode
             << ",\"fwd_sa\":" << (fwd_starts_active ? 1 : 0)
             << ",\"fwd_upg\":" << (fwd_upg ? 1 : 0)
             << ",\"fwd_excl\":" << (fwd_excluded_dmg ? 1 : 0)
             << ",\"fwd_dmg_bits\":" << oracle::fbits(fwd_dmg_amt)
             << ",\"f2_upg\":" << (fwd2_give_upg ? 1 : 0)
             << ",\"f2_uc\":" << (fwd2_under_const ? 1 : 0)
             << ",\"f2_oc\":" << (fwd2_obj_conflict ? 1 : 0)
             << ",\"f2_pc\":" << (fwd2_ply_conflict ? 1 : 0)
             << ",\"f2_sui\":" << (fwd2_suicided ? 1 : 0)
             << "}"
             << ",\"out\":{"
             << "\"oc_valid\":" << valid_mask
             << ",\"oc_pre_cnt\":" << oc_pre_count
             << ",\"oc_pre_st\":" << oc_pre_stealth
             << ",\"oc_pre_hr\":" << oc_pre_hero
             << ",\"oc_pre_pm\":" << oc_pre_player_mask
             << ",\"oc_pre_ids\":";
        append_u32_array(vecs, oc_pre_ids);
        vecs << ",\"oc_pre_crc\":" << oc_pre_crc
             << ",\"oc_pre_xfer\":\"" << oc_pre_hex << "\""
             << ",\"oc_mid_pm\":" << oc_mid_player_mask
             << ",\"oc_mid_w\":" << (oc_mid_wanters ? 1 : 0)
             << ",\"oc_post_cnt\":" << oc_post_count
             << ",\"oc_post_st\":" << oc_post_stealth
             << ",\"oc_post_hr\":" << oc_post_hero
             << ",\"oc_inf_hp_bits\":" << inf_hp_bits
             << ",\"oc_inf_cont\":" << (inf_contained ? 1 : 0)
             << ",\"oc_post_crc\":" << oc_post_crc
             << ",\"oc_post_xfer\":\"" << oc_post_hex << "\""
             << ",\"oc_col_cnt\":" << oc_col_cnt
             << ",\"oc_col_veh\":" << (oc_col_has_veh ? 1 : 0)
             << ",\"sd0_prob\":" << sd0_prob
             << ",\"sd0_app\":" << (sd0_app ? 1 : 0)
             << ",\"sd1_prob\":" << sd1_prob
             << ",\"sd_die_rng\":" << sd_post_die_rng_crc
             << ",\"sd0_sink\":" << sd0_sink
             << ",\"sd0_mid\":" << sd0_mid
             << ",\"sd0_dest\":" << sd0_dest
             << ",\"sd0_flags\":" << sd0_flags
             << ",\"sd0_pri\":" << sd0_pri
             << ",\"sd1_flags\":" << sd1_flags
             << ",\"sd1_pri\":" << sd1_pri
             << ",\"sd0_crc\":" << sd0_crc
             << ",\"sd0_xfer\":\"" << sd0_hex << "\""
             << ",\"sd_dest_f\":" << sd_destroyed_frame
             << ",\"sd_pos_z_bits\":" << oracle::fbits(sd_pos_z)
             << ",\"sd0_flags_post\":" << sd0_flags_post
             << ",\"sd_step_rng\":" << sd_step_rng_crc
             << ",\"id_dest\":" << (id_destroyed ? 1 : 0)
             << ",\"id_rng\":" << id_post_rng_crc
             << ",\"id_crc\":" << id_crc
             << ",\"id_xfer\":\"" << id_pre_hex << "\""
             << ",\"ah_rng\":" << ah_init_rng_crc
             << ",\"ah_init_pri\":" << ah_init_pri
             << ",\"ah_old_rem\":" << (ah_old_removed ? 1 : 0)
             << ",\"ah_hp_bits\":" << oracle::fbits(ah_self_hp)
             << ",\"ah_ally_hp_bits\":" << oracle::fbits(ah_ally_hp)
             << ",\"ah_enemy_hp_bits\":" << oracle::fbits(ah_enemy_hp)
             << ",\"ah_forbid_hp_bits\":" << oracle::fbits(ah_forbid_hp)
             << ",\"ah_soon\":" << ah_soonest
             << ",\"ah_stop\":" << (ah_stopped ? 1 : 0)
             << ",\"ah_post_pri\":" << ah_post_pri
             << ",\"ah_crc\":" << ah_crc
             << ",\"ah_xfer\":\"" << ah_hex << "\""
             << ",\"fwd_init_pri\":" << fwd_init_pri
             << ",\"fwd_post_pri\":" << fwd_post_pri
             << ",\"fwd_crc\":" << fwd_crc
             << ",\"fwd_xfer\":\"" << fwd_hex << "\""
             << ",\"f2_fired\":" << (f2_fired ? 1 : 0)
             << ",\"f2_crc\":" << f2_crc
             << ",\"f2_xfer\":\"" << f2_hex << "\""
             << ",\"gs_init_pri\":" << gs_init_pri
             << ",\"gs_s1\":[" << (gs_s1_near ? 1 : 0) << "," << (gs_s1_mid ? 1 : 0)
             << "," << (gs_s1_nost ? 1 : 0) << "," << (gs_s1_enemy ? 1 : 0)
             << "," << (gs_s1_offmap ? 1 : 0) << "]"
             << ",\"gs_s1_rad_bits\":" << oracle::fbits(gs_s1_rad)
             << ",\"gs_s2\":[" << (gs_s2_mid ? 1 : 0) << "," << (gs_s2_dest ? 1 : 0) << "]"
             << ",\"gs_s3_dest\":" << (gs_s3_dest ? 1 : 0)
             << ",\"gs_s3_rad_bits\":" << oracle::fbits(gs_s3_rad)
             << ",\"gs_crc\":" << gs_crc
             << ",\"gs_xfer\":\"" << gs_hex << "\""
             << "}"
             << "}";
    }

    std::ostringstream doc;
    doc << "{\n"
        << "  \"packet\": \"WP-5.3-A\",\n"
        << "  \"modules\": [\"BehaviorModule\",\"ContainModule\",\"OpenContain\",\"SlowDeathBehavior\",\"InstantDeathBehavior\",\"AutoHealBehavior\",\"FireWeaponWhenDamagedBehavior\",\"FireWeaponWhenDeadBehavior\",\"GrantStealthBehavior\"],\n"
        << "  \"seed_count\": 1000,\n"
        << "  \"source_sha256\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/SlowDeathBehavior.cpp") << "\",\n"
        << "  \"seed0_diagnostics\": " << s0_diag << ",\n"
        << "  \"vectors\": [\n" << vecs.str() << "\n  ]\n}\n";

    oracle::write_file_atomic(out_path, doc.str());
    return 0;
}
