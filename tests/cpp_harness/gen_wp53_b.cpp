// WP-5.3-B Behavior & Contain Modules (8 retail modules:
// PKT-258 BattleBusSlowDeathBehavior, PKT-278 JetSlowDeathBehavior,
// PKT-299 TransportContain, PKT-271 GarrisonContain, PKT-274 HealContain,
// PKT-277 InternetHackContain, PKT-280 MobNexusContain, PKT-285 ParachuteContain;
// note PKT-289 PrisonBehavior is #ifdef ALLOW_SURRENDER with 0 symbols in libz_gameengine.a).
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_b.cpp --opt O0 -o /tmp/gen_wp53_b_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_b.cpp --opt O2 -o /tmp/gen_wp53_b_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_b.cpp --opt O2 --sanitize -o /tmp/gen_wp53_b_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_wp53_b_o2 docs/golden_vectors/WP53_B.json

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/RandomValue.h"
#include "Common/Team.h"
#include "Common/ThingTemplate.h"
#include "GameClient/Drawable.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BattleBusSlowDeathBehavior.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/GarrisonContain.h"
#include "GameLogic/Module/HealContain.h"
#include "GameLogic/Module/InternetHackContain.h"
#include "GameLogic/Module/JetSlowDeathBehavior.h"
#include "GameLogic/Module/MobNexusContain.h"
#include "GameLogic/Module/OpenContain.h"
#include "GameLogic/Module/ParachuteContain.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/SlowDeathBehavior.h"
#include "GameLogic/Module/TransportContain.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

#include <sstream>
#include <string>
#include <vector>

struct BbRealDeathTag { using type = Bool BattleBusSlowDeathBehavior::*; };
template struct MemberAccess<BbRealDeathTag, &BattleBusSlowDeathBehavior::m_isRealDeath>;
Bool BattleBusSlowDeathBehavior::* access_member(BbRealDeathTag);

struct BbFirstDeathTag { using type = Bool BattleBusSlowDeathBehavior::*; };
template struct MemberAccess<BbFirstDeathTag, &BattleBusSlowDeathBehavior::m_isInFirstDeath>;
Bool BattleBusSlowDeathBehavior::* access_member(BbFirstDeathTag);

struct BbGroundFrameTag { using type = UnsignedInt BattleBusSlowDeathBehavior::*; };
template struct MemberAccess<BbGroundFrameTag, &BattleBusSlowDeathBehavior::m_groundCheckFrame>;
UnsignedInt BattleBusSlowDeathBehavior::* access_member(BbGroundFrameTag);

struct BbPenaltyFrameTag { using type = UnsignedInt BattleBusSlowDeathBehavior::*; };
template struct MemberAccess<BbPenaltyFrameTag, &BattleBusSlowDeathBehavior::m_penaltyDeathFrame>;
UnsignedInt BattleBusSlowDeathBehavior::* access_member(BbPenaltyFrameTag);

struct JetDeathFrameTag { using type = UnsignedInt JetSlowDeathBehavior::*; };
template struct MemberAccess<JetDeathFrameTag, &JetSlowDeathBehavior::m_timerDeathFrame>;
UnsignedInt JetSlowDeathBehavior::* access_member(JetDeathFrameTag);

struct JetGroundFrameTag { using type = UnsignedInt JetSlowDeathBehavior::*; };
template struct MemberAccess<JetGroundFrameTag, &JetSlowDeathBehavior::m_timerOnGroundFrame>;
UnsignedInt JetSlowDeathBehavior::* access_member(JetGroundFrameTag);

struct JetRollRateTag { using type = Real JetSlowDeathBehavior::*; };
template struct MemberAccess<JetRollRateTag, &JetSlowDeathBehavior::m_rollRate>;
Real JetSlowDeathBehavior::* access_member(JetRollRateTag);

struct TcPayloadCreatedTag { using type = Bool TransportContain::*; };
template struct MemberAccess<TcPayloadCreatedTag, &TransportContain::m_payloadCreated>;
Bool TransportContain::* access_member(TcPayloadCreatedTag);

struct TcExtraSlotsTag { using type = Int TransportContain::*; };
template struct MemberAccess<TcExtraSlotsTag, &TransportContain::m_extraSlotsInUse>;
Int TransportContain::* access_member(TcExtraSlotsTag);

struct TcFrameNotBusyTag { using type = UnsignedInt TransportContain::*; };
template struct MemberAccess<TcFrameNotBusyTag, &TransportContain::m_frameExitNotBusy>;
UnsignedInt TransportContain::* access_member(TcFrameNotBusyTag);

struct GcOrigTeamTag { using type = Team* GarrisonContain::*; };
template struct MemberAccess<GcOrigTeamTag, &GarrisonContain::m_originalTeam>;
Team* GarrisonContain::* access_member(GcOrigTeamTag);

struct GcHideStealthTag { using type = Bool GarrisonContain::*; };
template struct MemberAccess<GcHideStealthTag, &GarrisonContain::m_hideGarrisonedStateFromNonallies>;
Bool GarrisonContain::* access_member(GcHideStealthTag);

struct GcPtsInUseTag { using type = Int GarrisonContain::*; };
template struct MemberAccess<GcPtsInUseTag, &GarrisonContain::m_garrisonPointsInUse>;
Int GarrisonContain::* access_member(GcPtsInUseTag);

struct GcPtsInitTag { using type = Bool GarrisonContain::*; };
template struct MemberAccess<GcPtsInitTag, &GarrisonContain::m_garrisonPointsInitialized>;
Bool GarrisonContain::* access_member(GcPtsInitTag);

struct GcRallyValidTag { using type = Bool GarrisonContain::*; };
template struct MemberAccess<GcRallyValidTag, &GarrisonContain::m_rallyValid>;
Bool GarrisonContain::* access_member(GcRallyValidTag);

struct MnExtraSlotsTag { using type = Int MobNexusContain::*; };
template struct MemberAccess<MnExtraSlotsTag, &MobNexusContain::m_extraSlotsInUse>;
Int MobNexusContain::* access_member(MnExtraSlotsTag);

struct PcPitchTag { using type = Real ParachuteContain::*; };
template struct MemberAccess<PcPitchTag, &ParachuteContain::m_pitch>;
Real ParachuteContain::* access_member(PcPitchTag);

struct PcRollTag { using type = Real ParachuteContain::*; };
template struct MemberAccess<PcRollTag, &ParachuteContain::m_roll>;
Real ParachuteContain::* access_member(PcRollTag);

struct PcPitchRateTag { using type = Real ParachuteContain::*; };
template struct MemberAccess<PcPitchRateTag, &ParachuteContain::m_pitchRate>;
Real ParachuteContain::* access_member(PcPitchRateTag);

struct PcRollRateTag { using type = Real ParachuteContain::*; };
template struct MemberAccess<PcRollRateTag, &ParachuteContain::m_rollRate>;
Real ParachuteContain::* access_member(PcRollRateTag);

struct PcStartZTag { using type = Real ParachuteContain::*; };
template struct MemberAccess<PcStartZTag, &ParachuteContain::m_startZ>;
Real ParachuteContain::* access_member(PcStartZTag);

struct PcOpenedTag { using type = Bool ParachuteContain::*; };
template struct MemberAccess<PcOpenedTag, &ParachuteContain::m_opened>;
Bool ParachuteContain::* access_member(PcOpenedTag);

struct PcOverrideTag { using type = Bool ParachuteContain::*; };
template struct MemberAccess<PcOverrideTag, &ParachuteContain::m_isLandingOverrideSet>;
Bool ParachuteContain::* access_member(PcOverrideTag);

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

static uint64_t fnv1a64(const std::vector<uint8_t>& bytes) {
    uint64_t h = 0xcbf29ce484222325ULL;
    for (uint8_t b : bytes) {
        h ^= static_cast<uint64_t>(b);
        h *= 0x100000001b3ULL;
    }
    return h;
}

struct XferCapture {
    uint32_t crc = 0;
    uint32_t save_crc = 0;
    uint32_t len = 0;
    uint64_t fnv1a = 0;
    std::string hex;
};

static XferCapture capture_snapshot(Snapshot* s) {
    XferCapture cap;
    cap.crc = oracle_wp51::World::snapshot_crc(s);
    oracle::MemoryXfer xs(XFER_SAVE);
    xs.open("save");
    oracle::call_snapshot_xfer(s, &xs);
    xs.close();
    cap.save_crc = xs.bytes_crc();
    cap.len = static_cast<uint32_t>(xs.bytes.size());
    cap.fnv1a = fnv1a64(xs.bytes);
    cap.hex = xs.hex();
    return cap;
}

static void append_capture(
    std::ostringstream& ss,
    const char* prefix,
    const XferCapture& cap,
    bool force_hex
) {
    ss << ",\"" << prefix << "_crc\":" << cap.crc
       << ",\"" << prefix << "_xfer_len\":" << cap.len
       << ",\"" << prefix << "_xfer_fnv\":\"" << std::hex << cap.fnv1a << std::dec << "\""
       << ",\"" << prefix << "_xfer_crc\":" << cap.save_crc;
    if (force_hex || cap.len <= 256) {
        ss << ",\"" << prefix << "_xfer_hex\":\"" << cap.hex << "\"";
    }
}

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "/tmp/wp53_b_out.json";
    oracle_wp51::World w;
    w.init_with_neutral_player();

    w.load_ini(
        "FXList FX_BB_Start\nEnd\n"
        "FXList FX_BB_Ground\nEnd\n"
        "FXList FX_Jet_Ground\nEnd\n"
        "FXList FX_Jet_Init\nEnd\n"
        "FXList FX_Jet_Sec\nEnd\n"
        "FXList FX_Jet_Hit\nEnd\n"
        "FXList FX_Jet_Final\nEnd\n"

        "ObjectCreationList OCL_BB_Start\nEnd\n"
        "ObjectCreationList OCL_BB_Ground\nEnd\n"
        "ObjectCreationList OCL_Jet_Ground\nEnd\n"
        "ObjectCreationList OCL_Jet_Init\nEnd\n"
        "ObjectCreationList OCL_Jet_Sec\nEnd\n"
        "ObjectCreationList OCL_Jet_Hit\nEnd\n"
        "ObjectCreationList OCL_Jet_Final\nEnd\n"

        "Locomotor InfLoco\n"
        "  Surfaces = GROUND\n"
        "  Speed = 30\n"
        "  SpeedDamaged = 20\n"
        "  TurnRate = 180\n"
        "  TurnRateDamaged = 180\n"
        "  Acceleration = 100\n"
        "  Braking = 100\n"
        "  Appearance = TWO_LEGS\n"
        "End\n"
        "Locomotor JetLoco\n"
        "  Surfaces = AIR\n"
        "  Speed = 60\n"
        "  SpeedDamaged = 40\n"
        "  TurnRate = 90\n"
        "  TurnRateDamaged = 90\n"
        "  Acceleration = 60\n"
        "  Braking = 60\n"
        "  Appearance = WINGS\n"
        "  PitchStiffness = 0.2\n"
        "  RollStiffness = 0.2\n"
        "  PitchDamping = 0.8\n"
        "  RollDamping = 0.8\n"
        "End\n"
        "Locomotor ParaFreefallLoco\n"
        "  Surfaces = AIR\n"
        "  Speed = 20\n"
        "  SpeedDamaged = 20\n"
        "  TurnRate = 60\n"
        "  TurnRateDamaged = 60\n"
        "  Acceleration = 50\n"
        "  Braking = 50\n"
        "  Appearance = TWO_LEGS\n"
        "  PitchStiffness = 0.1\n"
        "  RollStiffness = 0.1\n"
        "  PitchDamping = 0.5\n"
        "  RollDamping = 0.5\n"
        "End\n"
        "Locomotor ParaNormalLoco\n"
        "  Surfaces = AIR\n"
        "  Speed = 15\n"
        "  SpeedDamaged = 15\n"
        "  TurnRate = 60\n"
        "  TurnRateDamaged = 60\n"
        "  Acceleration = 50\n"
        "  Braking = 50\n"
        "  Appearance = TWO_LEGS\n"
        "  PitchStiffness = 0.15\n"
        "  RollStiffness = 0.12\n"
        "  PitchDamping = 0.85\n"
        "  RollDamping = 0.82\n"
        "End\n"

        "Weapon Wpn_Rifle\n"
        "  PrimaryDamage = 10.0\n"
        "  AttackRange = 100.0\n"
        "  DamageType = SMALL_ARMS\n"
        "  DeathType = NORMAL\n"
        "  AutoReloadsClip = Yes\n"
        "  ShotsPerBarrel = 1\n"
        "End\n"
        "Weapon Wpn_Melee\n"
        "  PrimaryDamage = 10.0\n"
        "  AttackRange = 5.0\n"
        "  DamageType = SMALL_ARMS\n"
        "  DeathType = NORMAL\n"
        "  AutoReloadsClip = Yes\n"
        "  ShotsPerBarrel = 1\n"
        "End\n"

        "Object P53B_InfArmed\n"
        "  KindOf = PRELOAD SELECTABLE CAN_ATTACK INFANTRY\n"
        "  TransportSlotCount = 1\n"
        "  WeaponSet\n"
        "    Conditions = None\n"
        "    Weapon = PRIMARY Wpn_Rifle\n"
        "  End\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 100.0\n"
        "    InitialHealth = 100.0\n"
        "  End\n"
        "  Behavior = PhysicsBehavior ModuleTag_Phys\n"
        "    Mass = 5.0\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "  Locomotor = SET_NORMAL InfLoco\n"
        "End\n"

        "Object P53B_InfUnarmed\n"
        "  KindOf = PRELOAD SELECTABLE INFANTRY\n"
        "  TransportSlotCount = 1\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 100.0\n"
        "    InitialHealth = 100.0\n"
        "  End\n"
        "  Behavior = PhysicsBehavior ModuleTag_Phys\n"
        "    Mass = 5.0\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "  Locomotor = SET_NORMAL InfLoco\n"
        "End\n"

        "Object P53B_InfMelee\n"
        "  KindOf = PRELOAD SELECTABLE CAN_ATTACK INFANTRY\n"
        "  TransportSlotCount = 1\n"
        "  WeaponSet\n"
        "    Conditions = None\n"
        "    Weapon = PRIMARY Wpn_Melee\n"
        "  End\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 100.0\n"
        "    InitialHealth = 100.0\n"
        "  End\n"
        "  Behavior = PhysicsBehavior ModuleTag_Phys\n"
        "    Mass = 5.0\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "  Locomotor = SET_NORMAL InfLoco\n"
        "End\n"

        "Object P53B_InfStealth\n"
        "  KindOf = PRELOAD SELECTABLE CAN_ATTACK INFANTRY STEALTH_GARRISON\n"
        "  TransportSlotCount = 1\n"
        "  WeaponSet\n"
        "    Conditions = None\n"
        "    Weapon = PRIMARY Wpn_Rifle\n"
        "  End\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 100.0\n"
        "    InitialHealth = 100.0\n"
        "  End\n"
        "  Behavior = StealthUpdate ModuleTag_Stealth\n"
        "    InnateStealth = Yes\n"
        "    StealthDelay = 100\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "  Locomotor = SET_NORMAL InfLoco\n"
        "End\n"

        "Object P53B_InfHeavy\n"
        "  KindOf = PRELOAD SELECTABLE INFANTRY\n"
        "  TransportSlotCount = 2\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 120.0\n"
        "    InitialHealth = 120.0\n"
        "  End\n"
        "  Behavior = PhysicsBehavior ModuleTag_Phys\n"
        "    Mass = 10.0\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "  Locomotor = SET_NORMAL InfLoco\n"
        "End\n"

        "Object P53B_InfZeroSlot\n"
        "  KindOf = PRELOAD SELECTABLE INFANTRY\n"
        "  TransportSlotCount = 0\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 100.0\n"
        "    InitialHealth = 100.0\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "  Locomotor = SET_NORMAL InfLoco\n"
        "End\n"

        "Object P53B_InfNoGarrison\n"
        "  KindOf = PRELOAD SELECTABLE INFANTRY NO_GARRISON\n"
        "  TransportSlotCount = 1\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 100.0\n"
        "    InitialHealth = 100.0\n"
        "  End\n"
        "End\n"

        "Object P53B_VehNoTrans\n"
        "  KindOf = PRELOAD SELECTABLE VEHICLE\n"
        "  TransportSlotCount = 1\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200.0\n"
        "    InitialHealth = 200.0\n"
        "  End\n"
        "End\n"

        "Object P53B_Shrub\n"
        "  KindOf = IMMOBILE SHRUBBERY\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 100.0\n"
        "    InitialHealth = 100.0\n"
        "  End\n"
        "End\n"

        "Object P53B_BattleBus\n"
        "  KindOf = PRELOAD SELECTABLE VEHICLE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200.0\n"
        "    InitialHealth = 200.0\n"
        "  End\n"
        "  Behavior = PhysicsBehavior ModuleTag_Phys\n"
        "    Mass = 10.0\n"
        "    ShockMaxYaw = 0.5\n"
        "    ShockMaxPitch = 0.4\n"
        "    ShockMaxRoll = 0.3\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "  Locomotor = SET_NORMAL InfLoco\n"
        "  Behavior = TransportContain ModuleTag_Contain\n"
        "    Slots = 4\n"
        "    DamagePercentToUnits = 10%\n"
        "    AllowInsideKindOf = INFANTRY\n"
        "  End\n"
        "  Behavior = BattleBusSlowDeathBehavior ModuleTag_SlowDeath\n"
        "    ThrowForce = 12.0\n"
        "    PercentDamageToPassengers = 25%\n"
        "    EmptyHulkDestructionDelay = 300\n"
        "    FXStartUndeath = FX_BB_Start\n"
        "    OCLStartUndeath = OCL_BB_Start\n"
        "    FXHitGround = FX_BB_Ground\n"
        "    OCLHitGround = OCL_BB_Ground\n"
        "    DestructionDelay = 200\n"
        "    SinkRate = 0.5\n"
        "    ProbabilityModifier = 10\n"
        "  End\n"
        "End\n"

        "Object P53B_BattleBusNoDelay\n"
        "  KindOf = PRELOAD SELECTABLE VEHICLE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200.0\n"
        "    InitialHealth = 200.0\n"
        "  End\n"
        "  Behavior = PhysicsBehavior ModuleTag_Phys\n"
        "    Mass = 10.0\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "  Locomotor = SET_NORMAL InfLoco\n"
        "  Behavior = TransportContain ModuleTag_Contain\n"
        "    Slots = 2\n"
        "    AllowInsideKindOf = INFANTRY\n"
        "  End\n"
        "  Behavior = BattleBusSlowDeathBehavior ModuleTag_SlowDeath\n"
        "    ThrowForce = 8.0\n"
        "    PercentDamageToPassengers = 10%\n"
        "    EmptyHulkDestructionDelay = 0\n"
        "    DestructionDelay = 150\n"
        "    SinkRate = 0.25\n"
        "    ProbabilityModifier = 10\n"
        "  End\n"
        "End\n"

        "Object P53B_Jet\n"
        "  KindOf = PRELOAD SELECTABLE VEHICLE AIRCRAFT\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 100.0\n"
        "    InitialHealth = 100.0\n"
        "  End\n"
        "  Behavior = PhysicsBehavior ModuleTag_Phys\n"
        "    Mass = 10.0\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "  Locomotor = SET_NORMAL JetLoco\n"
        "  Behavior = JetSlowDeathBehavior ModuleTag_JetSD\n"
        "    FXOnGroundDeath = FX_Jet_Ground\n"
        "    OCLOnGroundDeath = OCL_Jet_Ground\n"
        "    FXInitialDeath = FX_Jet_Init\n"
        "    OCLInitialDeath = OCL_Jet_Init\n"
        "    DelaySecondaryFromInitialDeath = 100\n"
        "    FXSecondary = FX_Jet_Sec\n"
        "    OCLSecondary = OCL_Jet_Sec\n"
        "    FXHitGround = FX_Jet_Hit\n"
        "    OCLHitGround = OCL_Jet_Hit\n"
        "    DelayFinalBlowUpFromHitGround = 150\n"
        "    FXFinalBlowUp = FX_Jet_Final\n"
        "    OCLFinalBlowUp = OCL_Jet_Final\n"
        "    RollRate = 0.25\n"
        "    RollRateDelta = 95%\n"
        "    PitchRate = 0.15\n"
        "    FallHowFast = 40%\n"
        "    DestructionDelay = 10000\n"
        "    SinkRate = 0.0\n"
        "    ProbabilityModifier = 10\n"
        "  End\n"
        "End\n"

        "Object P53B_Transport\n"
        "  KindOf = PRELOAD SELECTABLE VEHICLE TRANSPORT\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 300.0\n"
        "    InitialHealth = 300.0\n"
        "  End\n"
        "  Behavior = PhysicsBehavior ModuleTag_Phys\n"
        "    Mass = 20.0\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "  Locomotor = SET_NORMAL InfLoco\n"
        "  Behavior = TransportContain ModuleTag_Contain\n"
        "    Slots = 4\n"
        "    InitialPayload = P53B_InfArmed 1\n"
        "    HealthRegen%PerSec = 30.0\n"
        "    ExitDelay = 100\n"
        "    ArmedRidersUpgradeMyWeaponSet = Yes\n"
        "    GoAggressiveOnExit = Yes\n"
        "    OrientLikeContainerOnExit = Yes\n"
        "    KeepContainerVelocityOnExit = Yes\n"
        "    ScatterNearbyOnExit = No\n"
        "    ResetMoodCheckTimeOnExit = Yes\n"
        "    DestroyRidersWhoAreNotFreeToExit = No\n"
        "    BurnedDeathToUnits = Yes\n"
        "    PassengersAllowedToFire = Yes\n"
        "    AllowInsideKindOf = INFANTRY\n"
        "  End\n"
        "End\n"

        "Object P53B_TransportAirDestroy\n"
        "  KindOf = PRELOAD SELECTABLE VEHICLE TRANSPORT\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 300.0\n"
        "    InitialHealth = 300.0\n"
        "  End\n"
        "  Behavior = PhysicsBehavior ModuleTag_Phys\n"
        "    Mass = 20.0\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "  Locomotor = SET_NORMAL InfLoco\n"
        "  Behavior = TransportContain ModuleTag_Contain\n"
        "    Slots = 3\n"
        "    DelayExitInAir = Yes\n"
        "    DestroyRidersWhoAreNotFreeToExit = Yes\n"
        "    ExitPitchRate = 0.5\n"
        "    AllowInsideKindOf = INFANTRY\n"
        "  End\n"
        "End\n"

        "Object P53B_GarrisonEncl\n"
        "  KindOf = PRELOAD SELECTABLE STRUCTURE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 500.0\n"
        "    InitialHealth = 500.0\n"
        "  End\n"
        "  Behavior = GarrisonContain ModuleTag_Garr\n"
        "    ContainMax = 4\n"
        "    InitialRoster = P53B_InfArmed 1\n"
        "    HealObjects = Yes\n"
        "    TimeForFullHeal = 300\n"
        "    ImmuneToClearBuildingAttacks = No\n"
        "    IsEnclosingContainer = Yes\n"
        "    MobileGarrison = No\n"
        "  End\n"
        "End\n"

        "Object P53B_GarrisonMob\n"
        "  KindOf = PRELOAD SELECTABLE STRUCTURE GARRISONABLE_UNTIL_DESTROYED\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 500.0\n"
        "    InitialHealth = 500.0\n"
        "  End\n"
        "  Behavior = GarrisonContain ModuleTag_Garr\n"
        "    ContainMax = 4\n"
        "    HealObjects = No\n"
        "    ImmuneToClearBuildingAttacks = Yes\n"
        "    IsEnclosingContainer = No\n"
        "    MobileGarrison = Yes\n"
        "  End\n"
        "End\n"

        "Object P53B_HealCont\n"
        "  KindOf = PRELOAD SELECTABLE STRUCTURE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 400.0\n"
        "    InitialHealth = 400.0\n"
        "  End\n"
        "  Behavior = HealContain ModuleTag_Heal\n"
        "    ContainMax = 3\n"
        "    TimeForFullHeal = 200\n"
        "    AllowInsideKindOf = INFANTRY\n"
        "  End\n"
        "End\n"

        "Object P53B_NetHack\n"
        "  KindOf = PRELOAD SELECTABLE STRUCTURE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 400.0\n"
        "    InitialHealth = 400.0\n"
        "  End\n"
        "  Behavior = InternetHackContain ModuleTag_Hack\n"
        "    Slots = 3\n"
        "    InitialPayload = P53B_InfUnarmed 1\n"
        "    AllowInsideKindOf = INFANTRY\n"
        "  End\n"
        "End\n"

        "Object P53B_MobNexus\n"
        "  KindOf = PRELOAD SELECTABLE VEHICLE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 300.0\n"
        "    InitialHealth = 300.0\n"
        "  End\n"
        "  Behavior = PhysicsBehavior ModuleTag_Phys\n"
        "    Mass = 15.0\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "  Locomotor = SET_NORMAL InfLoco\n"
        "  Behavior = MobNexusContain ModuleTag_Mob\n"
        "    Slots = 4\n"
        "    InitialPayload = P53B_InfArmed 1\n"
        "    HealthRegen%PerSec = 30.0\n"
        "    ScatterNearbyOnExit = Yes\n"
        "    OrientLikeContainerOnExit = Yes\n"
        "    KeepContainerVelocityOnExit = Yes\n"
        "    ExitPitchRate = 0.4\n"
        "    AllowInsideKindOf = INFANTRY\n"
        "  End\n"
        "End\n"

        "Object P53B_Parachute\n"
        "  KindOf = PRELOAD PARACHUTE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 50.0\n"
        "    InitialHealth = 50.0\n"
        "  End\n"
        "  Behavior = PhysicsBehavior ModuleTag_Phys\n"
        "    Mass = 2.0\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "  Locomotor = SET_FREEFALL ParaFreefallLoco\n"
        "  Locomotor = SET_NORMAL ParaNormalLoco\n"
        "  Behavior = ParachuteContain ModuleTag_Para\n"
        "    PitchRateMax = 30.0\n"
        "    RollRateMax = 30.0\n"
        "    LowAltitudeDamping = 0.2\n"
        "    ParachuteOpenDist = 15.0\n"
        "    KillWhenLandingInWaterSlop = 10.0\n"
        "    FreeFallDamagePercent = 50%\n"
        "    AllowInsideKindOf = INFANTRY\n"
        "  End\n"
        "End\n"

        "Object P53B_ParachuteZero\n"
        "  KindOf = PRELOAD PARACHUTE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 50.0\n"
        "    InitialHealth = 50.0\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n"
        "  End\n"
        "  Locomotor = SET_FREEFALL ParaFreefallLoco\n"
        "  Locomotor = SET_NORMAL ParaNormalLoco\n"
        "  Behavior = ParachuteContain ModuleTag_Para\n"
        "    PitchRateMax = 0.0\n"
        "    RollRateMax = 0.0\n"
        "    LowAltitudeDamping = 0.2\n"
        "    ParachuteOpenDist = 0.0\n"
        "    KillWhenLandingInWaterSlop = 10.0\n"
        "    FreeFallDamagePercent = 0%\n"
        "    AllowInsideKindOf = INFANTRY\n"
        "  End\n"
        "End\n"
    );

    const uint32_t team_neutral_id = w.team_neutral ? static_cast<uint32_t>(w.team_neutral->getID()) : 0u;
    const uint32_t team_a_id = w.team_a ? static_cast<uint32_t>(w.team_a->getID()) : 1u;
    const uint32_t team_b_id = w.team_b ? static_cast<uint32_t>(w.team_b->getID()) : 2u;

    std::ostringstream vecs;
    std::string s0_diag = "{}";

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint64_t rng = 0x53B00001ULL ^ (static_cast<uint64_t>(seed) * 0x9e3779b97f4a7c15ULL);
        const UnsignedInt frame0 = 20u + static_cast<UnsignedInt>(oracle::splitmix64(rng) % 200u);
        const UnsignedInt logic_seed = static_cast<UnsignedInt>(oracle::splitmix64(rng));
        const bool force_hex = (seed <= 8u);

        const uint32_t bb_mode = static_cast<uint32_t>(seed % 3u); // 0=hulk w/ delay, 1=empty delay=0, 2=direct real death
        const uint32_t jet_mode = static_cast<uint32_t>((seed / 3u) % 4u); // 0=ground, 1=deck_offset, 2=air->land->final, 3=air->shrub
        const uint32_t tc_mode = static_cast<uint32_t>((seed / 5u) % 4u);
        const uint32_t gc_mode = static_cast<uint32_t>((seed / 7u) % 4u);
        const uint32_t hc_delay_sel = static_cast<uint32_t>((seed / 11u) % 2u);
        const uint32_t mn_expose = static_cast<uint32_t>((seed / 13u) % 2u);
        const uint32_t pc_mode = static_cast<uint32_t>((seed / 17u) % 4u);

        // =========================================================================
        // 1) BattleBusSlowDeathBehavior (PKT-258)
        // =========================================================================
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame0);
        InitRandom(logic_seed);

        const char* bb_tmpl = (bb_mode == 1u) ? "P53B_BattleBusNoDelay" : "P53B_BattleBus";
        Object* bb_obj = w.spawn_object(bb_tmpl, w.team_a, {100.0f, 100.0f, 10.0f}, 0.0f);
        Object* bb_rider = w.spawn_object("P53B_InfArmed", w.team_a, {100.0f, 100.0f, 10.0f}, 0.0f);
        bb_obj->getContain()->addToContain(bb_rider);
        BattleBusSlowDeathBehavior* bb_mod = find_behavior<BattleBusSlowDeathBehavior>(bb_obj);

        if (bb_mode == 2u) {
            DamageInfo d;
            d.in.m_damageType = DAMAGE_EXPLOSION;
            d.in.m_deathType = DEATH_NORMAL;
            d.in.m_amount = 500.0f;
            bb_obj->attemptDamage(&d);
            step_single_update(w, bb_mod, frame0 + 1u);
        } else {
            DamageInfo d;
            d.in.m_damageType = DAMAGE_EXPLOSION;
            d.in.m_deathType = DEATH_NORMAL;
            d.in.m_amount = 200.0f;
            bb_mod->beginSlowDeath(&d);
            step_single_update(w, bb_mod, frame0 + 5u);  // still airborne before groundCheckFrame
            step_single_update(w, bb_mod, frame0 + 10u); // exactly == groundCheckFrame (strict <)
            Coord3D landed_pos{100.0f, 100.0f, 0.0f};
            bb_obj->setPosition(&landed_pos);
            step_single_update(w, bb_mod, frame0 + 11u); // lands now!
        }

        const bool bb_mid_second_life = (bb_obj->getDrawable() && bb_obj->getDrawable()->getModelConditionFlags().test(MODELCONDITION_SECOND_LIFE) != FALSE);
        const bool bb_mid_held = (bb_obj->isDisabledByType(DISABLED_HELD) != FALSE);
        const Real bb_rider_hp = bb_rider->getBodyModule()->getHealth();
        const uint32_t bb_mid_pri = bb_mod->friend_getPriority();
        const uint32_t bb_mid_rng = GetGameLogicRandomSeedCRC();
        const XferCapture bb_mid_cap = capture_snapshot(bb_mod);

        if (bb_mode == 0u) {
            // Check refill resets penalty frame, then empty -> penalty kill -> real death
            step_single_update(w, bb_mod, frame0 + 12u); // still occupied
            bb_obj->getContain()->removeFromContain(bb_rider, FALSE);
            step_single_update(w, bb_mod, frame0 + 27u); // empty -> sets m_penaltyDeathFrame = frame0+27+9 = frame0+36
            step_single_update(w, bb_mod, frame0 + 42u); // frame > penaltyDeathFrame -> kills self (real death!)
        }
        const bool bb_post_real = (bb_mod->*access_member(BbRealDeathTag()) != FALSE);
        const bool bb_post_first = (bb_mod->*access_member(BbFirstDeathTag()) != FALSE);
        const UnsignedInt bb_post_gframe = bb_mod->*access_member(BbGroundFrameTag());
        const UnsignedInt bb_post_pframe = bb_mod->*access_member(BbPenaltyFrameTag());
        const uint32_t bb_post_pri = bb_mod->friend_getPriority();
        const XferCapture bb_post_cap = capture_snapshot(bb_mod);

        destroy_and_flush_all(w);

        // =========================================================================
        // 2) JetSlowDeathBehavior (PKT-278)
        // =========================================================================
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame0);
        InitRandom(logic_seed);

        const Real jet_z = (jet_mode == 0u) ? 0.0f : 80.0f;
        Object* jet_obj = w.spawn_object("P53B_Jet", w.team_a, {200.0f, 200.0f, jet_z}, 0.0f);
        if (jet_mode == 1u) {
            jet_obj->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_DECK_HEIGHT_OFFSET));
        }
        Object* jet_shrub = nullptr;
        if (jet_mode == 3u) {
            jet_shrub = w.spawn_object("P53B_Shrub", w.team_neutral, {200.0f, 200.0f, 0.0f}, 0.0f);
        }
        JetSlowDeathBehavior* jet_mod = find_behavior<JetSlowDeathBehavior>(jet_obj);
        DamageInfo jd;
        jd.in.m_damageType = DAMAGE_EXPLOSION;
        jd.in.m_deathType = DEATH_NORMAL;
        jd.in.m_amount = 300.0f;
        jet_obj->attemptDamage(&jd);

        const bool jet_dest_on_die = (jet_obj->isDestroyed() != FALSE);
        const bool jet_deck_cleared = (jet_obj->getStatusBits().test(OBJECT_STATUS_DECK_HEIGHT_OFFSET) == FALSE);
        const Real jet_roll0 = jet_mod->*access_member(JetRollRateTag());
        const UnsignedInt jet_df0 = jet_mod->*access_member(JetDeathFrameTag());
        const XferCapture jet_mid_cap = capture_snapshot(jet_mod);

        if (!jet_dest_on_die) {
            step_single_update(w, jet_mod, frame0 + 1u);
            step_single_update(w, jet_mod, frame0 + 4u); // secondary trigger (Delay=100ms = 3 frames)
            if (jet_mode == 3u && jet_shrub && jet_obj->getPhysics()) {
                // Simulate shrubbery collision on the way down
                jet_obj->getPhysics()->onCollide(jet_shrub, jet_obj->getPosition(), nullptr);
                step_single_update(w, jet_mod, frame0 + 5u);
            } else {
                Coord3D j_ground{200.0f, 200.0f, 0.0f};
                jet_obj->setPosition(&j_ground);
                step_single_update(w, jet_mod, frame0 + 5u); // hits ground
            }
        }
        const UnsignedInt jet_gf_mid = jet_mod->*access_member(JetGroundFrameTag());
        const UnsignedInt jet_df_mid = jet_mod->*access_member(JetDeathFrameTag());
        const Real jet_roll_mid = jet_mod->*access_member(JetRollRateTag());
        const XferCapture jet_post_cap = capture_snapshot(jet_mod);
        if (!jet_obj->isDestroyed()) {
            step_single_update(w, jet_mod, frame0 + 12u); // final blow up (Delay=150ms = 5 frames)
        }
        const bool jet_dest_final = (jet_obj->isDestroyed() != FALSE);
        const uint32_t jet_rng = GetGameLogicRandomSeedCRC();

        destroy_and_flush_all(w);

        // =========================================================================
        // 3) TransportContain (PKT-299)
        // =========================================================================
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame0);
        InitRandom(logic_seed);

        const Real tc_z = (tc_mode == 2u) ? 20.0f : 0.0f;
        const char* tc_tmpl = (tc_mode == 3u) ? "P53B_TransportAirDestroy" : "P53B_Transport";
        Object* tc_obj = w.spawn_object(tc_tmpl, w.team_a, {300.0f, 300.0f, tc_z}, 0.75f);
        TransportContain* tc_mod = find_behavior<TransportContain>(tc_obj);
        if (tc_obj->getPhysics()) {
            Coord3D vforce{20.0f, 0.0f, 0.0f};
            tc_obj->getPhysics()->applyForce(&vforce);
        }

        // First update creates InitialPayload on P53B_Transport
        step_single_update(w, tc_mod, frame0 + 1u);
        const bool tc_payload_created = (tc_mod->*access_member(TcPayloadCreatedTag()) != FALSE);
        const UnsignedInt tc_init_count = tc_mod->getContainCount();
        const bool tc_init_loaded = (tc_obj->getDrawable() && tc_obj->getDrawable()->getModelConditionFlags().test(MODELCONDITION_LOADED) != FALSE);
        const bool tc_init_wpn_upg = tc_obj->testWeaponSetFlag(WEAPONSET_PLAYER_UPGRADE);

        Object* tc_enemy_inf = w.spawn_object("P53B_InfArmed", w.team_b, {310.0f, 300.0f, 0.0f}, 0.0f);
        Object* tc_zero_inf = w.spawn_object("P53B_InfZeroSlot", w.team_a, {312.0f, 300.0f, 0.0f}, 0.0f);
        Object* tc_veh = w.spawn_object("P53B_VehNoTrans", w.team_a, {314.0f, 300.0f, 0.0f}, 0.0f);
        Object* tc_heavy = w.spawn_object("P53B_InfHeavy", w.team_a, {316.0f, 300.0f, 0.0f}, 0.0f);
        Object* tc_melee = w.spawn_object("P53B_InfMelee", w.team_a, {318.0f, 300.0f, 0.0f}, 0.0f);
        Object* tc_para = w.spawn_object("P53B_Parachute", w.team_a, {320.0f, 300.0f, 30.0f}, 0.0f);
        Object* tc_para_rider = w.spawn_object("P53B_InfUnarmed", w.team_a, {320.0f, 300.0f, 30.0f}, 0.0f);
        tc_para->getContain()->addToContain(tc_para_rider);

        const uint32_t tc_valid_mask =
            (tc_mod->isValidContainerFor(tc_enemy_inf, TRUE) ? 1u : 0u) |
            (tc_mod->isValidContainerFor(tc_zero_inf, TRUE) ? 2u : 0u) |
            (tc_mod->isValidContainerFor(tc_veh, TRUE) ? 4u : 0u) |
            (tc_mod->isValidContainerFor(tc_heavy, TRUE) ? 8u : 0u) |
            (tc_mod->isValidContainerFor(tc_para, TRUE) ? 16u : 0u);

        // Injure heavy before loading so HealthRegen%PerSec runs
        DamageInfo tdmg;
        tdmg.in.m_damageType = DAMAGE_SMALL_ARMS;
        tdmg.in.m_amount = 40.0f;
        tc_heavy->attemptDamage(&tdmg);

        if (tc_mod->isValidContainerFor(tc_heavy, TRUE)) tc_mod->addToContain(tc_heavy);
        if (tc_mod->isValidContainerFor(tc_melee, TRUE)) tc_mod->addToContain(tc_melee);
        const Int tc_extra_slots = tc_mod->*access_member(TcExtraSlotsTag());
        const ContainModuleInterface* tc_cmi = tc_mod;
        const bool tc_heavy_can_fire = (tc_cmi->isPassengerAllowedToFire(tc_heavy->getID()) != FALSE);
        const bool tc_veh_can_fire = (tc_cmi->isPassengerAllowedToFire(tc_veh->getID()) != FALSE);

        step_single_update(w, tc_mod, frame0 + 2u);
        const Real tc_heavy_hp_regen = tc_heavy->getBodyModule()->getHealth();

        const XferCapture tc_mid_cap = capture_snapshot(tc_mod);
        const ExitDoorType tc_door = tc_mod->reserveDoorForExit(tc_heavy->getTemplate(), tc_heavy);
        if (tc_door != DOOR_NONE_AVAILABLE) {
            tc_mod->exitObjectViaDoor(tc_heavy, tc_door);
        }
        const bool tc_exit_busy_now = (tc_mod->isExitBusy() != FALSE);
        const UnsignedInt tc_not_busy_frame = tc_mod->*access_member(TcFrameNotBusyTag());
        const bool tc_heavy_fall = (tc_heavy->getPhysics() && tc_heavy->getPhysics()->getAllowToFall());

        if (tc_mode == 1u) {
            // Capture test
            tc_mod->onCapture(w.team_a->getControllingPlayer(), w.team_b->getControllingPlayer());
        } else if (tc_mode == 3u) {
            // DestroyRidersWhoAreNotFreeToExit on unmanned or water/off-map
            DamageInfo kd;
            kd.in.m_damageType = DAMAGE_EXPLOSION;
            kd.in.m_amount = 500.0f;
            tc_obj->attemptDamage(&kd);
        }
        const UnsignedInt tc_post_count = tc_mod->getContainCount();
        const uint32_t tc_rng = GetGameLogicRandomSeedCRC();

        destroy_and_flush_all(w);

        // =========================================================================
        // 4) GarrisonContain (PKT-271) with Neutral Player
        // =========================================================================
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame0);
        InitRandom(logic_seed);

        const char* gc_tmpl = (gc_mode == 2u) ? "P53B_GarrisonMob" : "P53B_GarrisonEncl";
        Object* gc_obj = w.spawn_object(gc_tmpl, w.team_neutral, {400.0f, 400.0f, 0.0f}, 0.5f);
        GarrisonContain* gc_mod = find_behavior<GarrisonContain>(gc_obj);
        const UnsignedInt gc_roster_count = gc_mod->getContainCount();

        // Remove initial roster to test stealth garrison ownership + neutral transition
        gc_mod->removeAllContained(FALSE);
        Object* gc_stealth = w.spawn_object("P53B_InfStealth", w.team_a, {410.0f, 400.0f, 0.0f}, 0.0f);
        Object* gc_inf = w.spawn_object("P53B_InfArmed", w.team_a, {412.0f, 400.0f, 0.0f}, 0.0f);
        Object* gc_nogarr = w.spawn_object("P53B_InfNoGarrison", w.team_a, {414.0f, 400.0f, 0.0f}, 0.0f);
        const bool gc_nogarr_valid = (gc_mod->isValidContainerFor(gc_nogarr, TRUE) != FALSE);

        DamageInfo gd_inf;
        gd_inf.in.m_damageType = DAMAGE_SMALL_ARMS;
        gd_inf.in.m_amount = 40.0f;
        gc_stealth->attemptDamage(&gd_inf);

        gc_mod->addToContain(gc_stealth);
        const bool gc_hide_stealth_only = (gc_mod->*access_member(GcHideStealthTag()) != FALSE);
        const uint32_t gc_team_after_garr = static_cast<uint32_t>(gc_obj->getTeam()->getID());
        const Team* gc_orig_team_ptr = gc_mod->*access_member(GcOrigTeamTag());
        const uint32_t gc_orig_team_id = gc_orig_team_ptr ? static_cast<uint32_t>(gc_orig_team_ptr->getID()) : 0u;
        const bool gc_can_attack = (gc_obj->getStatusBits().test(OBJECT_STATUS_CAN_ATTACK) != FALSE);
        const bool gc_pts_init = (gc_mod->*access_member(GcPtsInitTag()) != FALSE);

        gc_mod->addToContain(gc_inf);
        const bool gc_hide_with_regular = (gc_mod->*access_member(GcHideStealthTag()) != FALSE);
        step_single_update(w, gc_mod, frame0 + 1u);
        const Real gc_stealth_hp = gc_stealth->getBodyModule()->getHealth();

        const XferCapture gc_mid_cap = capture_snapshot(gc_mod);

        if (gc_mode == 0u) {
            gc_mod->setEvacDisposition(EVAC_BURST_FROM_CENTER);
            gc_mod->exitObjectViaDoor(gc_inf, DOOR_1);
        } else if (gc_mode == 1u) {
            gc_mod->setEvacDisposition(EVAC_TO_LEFT);
            gc_mod->exitObjectViaDoor(gc_inf, DOOR_1);
        } else {
            gc_mod->setEvacDisposition(EVAC_TO_RIGHT);
            gc_mod->removeAllContained(TRUE);
        }

        const bool gc_rally_valid = (gc_mod->*access_member(GcRallyValidTag()) != FALSE);
        const UnsignedInt gc_post_count = gc_mod->getContainCount();
        // Damage structure to ReallyDamaged
        DamageInfo gd_struct;
        gd_struct.in.m_damageType = DAMAGE_EXPLOSION;
        gd_struct.in.m_amount = 360.0f;
        gc_obj->attemptDamage(&gd_struct);
        const bool gc_valid_when_rd = (gc_mod->isValidContainerFor(gc_nogarr, TRUE) != FALSE ||
                                       gc_mod->isValidContainerFor(gc_inf, TRUE) != FALSE);
        const XferCapture gc_post_cap = capture_snapshot(gc_mod);
        const uint32_t gc_rng = GetGameLogicRandomSeedCRC();

        destroy_and_flush_all(w);

        // =========================================================================
        // 5) HealContain (PKT-274)
        // =========================================================================
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame0);
        InitRandom(logic_seed);

        Object* hc_obj = w.spawn_object("P53B_HealCont", w.team_a, {500.0f, 500.0f, 0.0f}, 0.0f);
        HealContain* hc_mod = find_behavior<HealContain>(hc_obj);
        Object* hc_inf = w.spawn_object("P53B_InfArmed", w.team_a, {510.0f, 500.0f, 0.0f}, 0.0f);
        DamageInfo hd;
        hd.in.m_damageType = DAMAGE_SMALL_ARMS;
        hd.in.m_amount = 50.0f;
        hc_inf->attemptDamage(&hd);
        hc_mod->addToContain(hc_inf);

        step_single_update(w, hc_mod, frame0 + 2u);
        const Real hc_mid_hp = hc_inf->getBodyModule()->getHealth();
        const UnsignedInt hc_mid_cnt = hc_mod->getContainCount();
        const XferCapture hc_mid_cap = capture_snapshot(hc_mod);

        const UnsignedInt hc_exit_f = (hc_delay_sel == 0u) ? (frame0 + 5u) : (frame0 + 6u);
        for (UnsignedInt f = frame0 + 3u; f <= hc_exit_f; ++f) {
            step_single_update(w, hc_mod, f);
        }
        const Real hc_post_hp = hc_inf->getBodyModule()->getHealth();
        const UnsignedInt hc_post_cnt = hc_mod->getContainCount();

        destroy_and_flush_all(w);

        // =========================================================================
        // 6) InternetHackContain (PKT-277)
        // =========================================================================
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame0);
        InitRandom(logic_seed);

        Object* ih_obj = w.spawn_object("P53B_NetHack", w.team_a, {600.0f, 600.0f, 0.0f}, 0.0f);
        InternetHackContain* ih_mod = find_behavior<InternetHackContain>(ih_obj);
        step_single_update(w, ih_mod, frame0 + 1u);
        Object* ih_hacker = w.spawn_object("P53B_InfUnarmed", w.team_a, {610.0f, 600.0f, 0.0f}, 0.0f);
        ih_mod->addToContain(ih_hacker);
        const UnsignedInt ih_cnt = ih_mod->getContainCount();
        const XferCapture ih_cap = capture_snapshot(ih_mod);

        destroy_and_flush_all(w);

        // =========================================================================
        // 7) MobNexusContain (PKT-280)
        // =========================================================================
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame0);
        InitRandom(logic_seed);

        Object* mn_obj = w.spawn_object("P53B_MobNexus", w.team_a, {700.0f, 700.0f, 0.0f}, 0.25f);
        MobNexusContain* mn_mod = find_behavior<MobNexusContain>(mn_obj);
        if (mn_obj->getPhysics()) {
            Coord3D mf{15.0f, 0.0f, 0.0f};
            mn_obj->getPhysics()->applyForce(&mf);
        }
        const UnsignedInt mn_init_cnt = mn_mod->getContainCount();
        Object* mn_ally = w.spawn_object("P53B_InfStealth", w.team_a, {710.0f, 700.0f, 0.0f}, 0.0f);
        Object* mn_enemy = w.spawn_object("P53B_InfArmed", w.team_b, {712.0f, 700.0f, 0.0f}, 0.0f);
        Object* mn_neut = w.spawn_object("P53B_InfArmed", w.team_neutral, {714.0f, 700.0f, 0.0f}, 0.0f);
        Object* mn_heavy = w.spawn_object("P53B_InfHeavy", w.team_a, {716.0f, 700.0f, 0.0f}, 0.0f);
        Object* mn_para = w.spawn_object("P53B_Parachute", w.team_a, {718.0f, 700.0f, 30.0f}, 0.0f);
        Object* mn_prider = w.spawn_object("P53B_InfUnarmed", w.team_a, {718.0f, 700.0f, 30.0f}, 0.0f);
        mn_para->getContain()->addToContain(mn_prider);

        const uint32_t mn_valid_mask =
            (mn_mod->isValidContainerFor(mn_ally, TRUE) ? 1u : 0u) |
            (mn_mod->isValidContainerFor(mn_enemy, TRUE) ? 2u : 0u) |
            (mn_mod->isValidContainerFor(mn_neut, TRUE) ? 4u : 0u) |
            (mn_mod->isValidContainerFor(mn_heavy, TRUE) ? 8u : 0u) |
            (mn_mod->isValidContainerFor(mn_para, TRUE) ? 16u : 0u);

        DamageInfo md;
        md.in.m_damageType = DAMAGE_SMALL_ARMS;
        md.in.m_amount = 30.0f;
        mn_ally->attemptDamage(&md);
        mn_mod->addToContain(mn_ally);
        mn_mod->addToContain(mn_heavy);
        step_single_update(w, mn_mod, frame0 + 1u);
        const Real mn_ally_hp = mn_ally->getBodyModule()->getHealth();
        const Int mn_extra_mid = mn_mod->*access_member(MnExtraSlotsTag());
        const XferCapture mn_mid_cap = capture_snapshot(mn_mod);

        const Bool mn_evac_ok = mn_mod->tryToEvacuate(mn_expose != 0u);
        const UnsignedInt mn_post_cnt = mn_mod->getContainCount();
        const Int mn_extra_post = mn_mod->*access_member(MnExtraSlotsTag());
        const bool mn_stealth_detected = (mn_ally->getStatusBits().test(OBJECT_STATUS_DETECTED) != FALSE);
        const uint32_t mn_rng = GetGameLogicRandomSeedCRC();

        destroy_and_flush_all(w);

        // =========================================================================
        // 8) ParachuteContain (PKT-285)
        // =========================================================================
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame0);
        InitRandom(logic_seed);

        const char* pc_tmpl = (pc_mode == 3u) ? "P53B_ParachuteZero" : "P53B_Parachute";
        const Real pc_z0 = (pc_mode == 2u) ? 10.0f : 40.0f;
        Object* pc_obj = w.spawn_object(pc_tmpl, w.team_a, {800.0f, 800.0f, pc_z0}, 0.25f);
        ParachuteContain* pc_mod = find_behavior<ParachuteContain>(pc_obj);
        Object* pc_rider = w.spawn_object("P53B_InfArmed", w.team_a, {800.0f, 800.0f, pc_z0}, 0.0f);
        Object* pc_rider2 = w.spawn_object("P53B_InfUnarmed", w.team_a, {800.0f, 800.0f, pc_z0}, 0.0f);
        Object* pc_zero = w.spawn_object("P53B_InfZeroSlot", w.team_a, {800.0f, 800.0f, pc_z0}, 0.0f);

        const bool pc_valid_zero_inf = (pc_mod->isValidContainerFor(pc_zero, TRUE) != FALSE);
        pc_mod->addToContain(pc_rider);
        const bool pc_valid_second = (pc_mod->isValidContainerFor(pc_rider2, TRUE) != FALSE);

        if (pc_mode == 1u) {
            Coord3D dest{825.0f, 815.0f, 0.0f};
            pc_mod->setOverrideDestination(&dest);
        }

        // First update at initial altitude -> m_startZ set
        step_single_update(w, pc_mod, frame0 + 1u);
        // Drop altitude to trigger opening!
        Coord3D pc_drop{800.0f, 800.0f, (pc_mode == 2u) ? 5.0f : 15.0f};
        pc_obj->setPosition(&pc_drop);
        step_single_update(w, pc_mod, frame0 + 2u);
        step_single_update(w, pc_mod, frame0 + 3u);

        const bool pc_opened = (pc_mod->*access_member(PcOpenedTag()) != FALSE);
        const bool pc_override = (pc_mod->*access_member(PcOverrideTag()) != FALSE);
        const Real pc_pitch = pc_mod->*access_member(PcPitchTag());
        const Real pc_roll = pc_mod->*access_member(PcRollTag());
        const Real pc_pitch_rate = pc_mod->*access_member(PcPitchRateTag());
        const Real pc_roll_rate = pc_mod->*access_member(PcRollRateTag());
        const Real pc_start_z = pc_mod->*access_member(PcStartZTag());
        const bool pc_no_coll = (pc_obj->getStatusBits().test(OBJECT_STATUS_NO_COLLISIONS) != FALSE);
        const XferCapture pc_mid_cap = capture_snapshot(pc_mod);

        if (pc_mode == 2u) {
            // Die while significantly above terrain (test FreeFallDamagePercent)
            Coord3D pc_high{800.0f, 800.0f, 60.0f};
            pc_obj->setPosition(&pc_high);
            DamageInfo pd;
            pd.in.m_damageType = DAMAGE_EXPLOSION;
            pd.in.m_amount = 200.0f;
            pc_obj->attemptDamage(&pd);
        } else {
            // Ground collide -> removeAllContained + kill parachute
            Coord3D pc_gnd{800.0f, 800.0f, 0.0f};
            pc_obj->setPosition(&pc_gnd);
            Coord3D norm{0.0f, 0.0f, 1.0f};
            pc_mod->onCollide(nullptr, &pc_gnd, &norm);
        }
        const UnsignedInt pc_post_cnt = pc_mod->getContainCount();
        const Real pc_rider_post_hp = pc_rider->getBodyModule()->getHealth();
        const bool pc_rider_fall = (pc_rider->getPhysics() && pc_rider->getPhysics()->getAllowToFall());
        const bool pc_para_dead = (pc_obj->isEffectivelyDead() != FALSE);
        const uint32_t pc_rng = GetGameLogicRandomSeedCRC();

        destroy_and_flush_all(w);

        if (seed == 0u) {
            std::ostringstream d;
            d << "{"
              << "\"team_neutral_id\":" << team_neutral_id
              << ",\"team_a_id\":" << team_a_id
              << ",\"team_b_id\":" << team_b_id
              << ",\"bb_mid_second_life\":" << (bb_mid_second_life ? 1 : 0)
              << ",\"bb_mid_held\":" << (bb_mid_held ? 1 : 0)
              << ",\"bb_rider_hp_bits\":" << oracle::fbits(bb_rider_hp)
              << ",\"bb_post_real\":" << (bb_post_real ? 1 : 0)
              << ",\"jet_dest_on_die\":" << (jet_dest_on_die ? 1 : 0)
              << ",\"jet_dest_final\":" << (jet_dest_final ? 1 : 0)
              << ",\"tc_init_count\":" << tc_init_count
              << ",\"tc_valid_mask\":" << tc_valid_mask
              << ",\"gc_team_after_garr\":" << gc_team_after_garr
              << ",\"gc_orig_team_id\":" << gc_orig_team_id
              << ",\"hc_mid_hp_bits\":" << oracle::fbits(hc_mid_hp)
              << ",\"hc_post_cnt\":" << hc_post_cnt
              << ",\"mn_valid_mask\":" << mn_valid_mask
              << ",\"pc_opened\":" << (pc_opened ? 1 : 0)
              << ",\"pc_post_cnt\":" << pc_post_cnt
              << "}";
            s0_diag = d.str();
        }

        if (seed > 0u) vecs << ",\n";
        vecs << "    {"
             << "\"seed\":" << seed
             << ",\"in\":{"
             << "\"frame\":" << frame0
             << ",\"logic_seed\":" << logic_seed
             << ",\"bb_mode\":" << bb_mode
             << ",\"jet_mode\":" << jet_mode
             << ",\"tc_mode\":" << tc_mode
             << ",\"gc_mode\":" << gc_mode
             << ",\"hc_sel\":" << hc_delay_sel
             << ",\"mn_exp\":" << mn_expose
             << ",\"pc_mode\":" << pc_mode
             << ",\"team_n\":" << team_neutral_id
             << ",\"team_a\":" << team_a_id
             << ",\"team_b\":" << team_b_id
             << "}"
             << ",\"out\":{"
             << "\"bb_mid_sl\":" << (bb_mid_second_life ? 1 : 0)
             << ",\"bb_mid_held\":" << (bb_mid_held ? 1 : 0)
             << ",\"bb_rider_hp\":" << oracle::fbits(bb_rider_hp)
             << ",\"bb_mid_pri\":" << bb_mid_pri
             << ",\"bb_mid_rng\":" << bb_mid_rng;
        append_capture(vecs, "bb_mid", bb_mid_cap, force_hex);
        vecs << ",\"bb_post_real\":" << (bb_post_real ? 1 : 0)
             << ",\"bb_post_first\":" << (bb_post_first ? 1 : 0)
             << ",\"bb_post_gf\":" << bb_post_gframe
             << ",\"bb_post_pf\":" << bb_post_pframe
             << ",\"bb_post_pri\":" << bb_post_pri;
        append_capture(vecs, "bb_post", bb_post_cap, force_hex);

        vecs << ",\"jet_die\":" << (jet_dest_on_die ? 1 : 0)
             << ",\"jet_deck_clr\":" << (jet_deck_cleared ? 1 : 0)
             << ",\"jet_roll0\":" << oracle::fbits(jet_roll0)
             << ",\"jet_df0\":" << jet_df0;
        append_capture(vecs, "jet_mid", jet_mid_cap, force_hex);
        vecs << ",\"jet_gf_mid\":" << jet_gf_mid
             << ",\"jet_df_mid\":" << jet_df_mid
             << ",\"jet_roll_mid\":" << oracle::fbits(jet_roll_mid)
             << ",\"jet_dest_fin\":" << (jet_dest_final ? 1 : 0)
             << ",\"jet_rng\":" << jet_rng;
        append_capture(vecs, "jet_post", jet_post_cap, force_hex);

        vecs << ",\"tc_pay\":" << (tc_payload_created ? 1 : 0)
             << ",\"tc_init_cnt\":" << tc_init_count
             << ",\"tc_loaded\":" << (tc_init_loaded ? 1 : 0)
             << ",\"tc_wpn_upg\":" << (tc_init_wpn_upg ? 1 : 0)
             << ",\"tc_valid\":" << tc_valid_mask
             << ",\"tc_extra\":" << tc_extra_slots
             << ",\"tc_h_fire\":" << (tc_heavy_can_fire ? 1 : 0)
             << ",\"tc_v_fire\":" << (tc_veh_can_fire ? 1 : 0)
             << ",\"tc_h_hp\":" << oracle::fbits(tc_heavy_hp_regen)
             << ",\"tc_door\":" << static_cast<int>(tc_door)
             << ",\"tc_busy\":" << (tc_exit_busy_now ? 1 : 0)
             << ",\"tc_nb_f\":" << tc_not_busy_frame
             << ",\"tc_h_fall\":" << (tc_heavy_fall ? 1 : 0)
             << ",\"tc_post_cnt\":" << tc_post_count
             << ",\"tc_rng\":" << tc_rng;
        append_capture(vecs, "tc_mid", tc_mid_cap, force_hex);

        vecs << ",\"gc_ros\":" << gc_roster_count
             << ",\"gc_nogarr\":" << (gc_nogarr_valid ? 1 : 0)
             << ",\"gc_hide_s\":" << (gc_hide_stealth_only ? 1 : 0)
             << ",\"gc_hide_r\":" << (gc_hide_with_regular ? 1 : 0)
             << ",\"gc_team\":" << gc_team_after_garr
             << ",\"gc_orig_team\":" << gc_orig_team_id
             << ",\"gc_atk\":" << (gc_can_attack ? 1 : 0)
             << ",\"gc_pts_init\":" << (gc_pts_init ? 1 : 0)
             << ",\"gc_st_hp\":" << oracle::fbits(gc_stealth_hp)
             << ",\"gc_rally\":" << (gc_rally_valid ? 1 : 0)
             << ",\"gc_post_cnt\":" << gc_post_count
             << ",\"gc_valid_rd\":" << (gc_valid_when_rd ? 1 : 0)
             << ",\"gc_rng\":" << gc_rng;
        append_capture(vecs, "gc_mid", gc_mid_cap, force_hex);
        append_capture(vecs, "gc_post", gc_post_cap, force_hex);

        vecs << ",\"hc_mid_hp\":" << oracle::fbits(hc_mid_hp)
             << ",\"hc_mid_cnt\":" << hc_mid_cnt
             << ",\"hc_post_hp\":" << oracle::fbits(hc_post_hp)
             << ",\"hc_post_cnt\":" << hc_post_cnt;
        append_capture(vecs, "hc_mid", hc_mid_cap, force_hex);

        vecs << ",\"ih_cnt\":" << ih_cnt;
        append_capture(vecs, "ih", ih_cap, force_hex);

        vecs << ",\"mn_init_cnt\":" << mn_init_cnt
             << ",\"mn_valid\":" << mn_valid_mask
             << ",\"mn_ally_hp\":" << oracle::fbits(mn_ally_hp)
             << ",\"mn_extra_mid\":" << mn_extra_mid
             << ",\"mn_evac\":" << (mn_evac_ok ? 1 : 0)
             << ",\"mn_post_cnt\":" << mn_post_cnt
             << ",\"mn_extra_post\":" << mn_extra_post
             << ",\"mn_st_det\":" << (mn_stealth_detected ? 1 : 0)
             << ",\"mn_rng\":" << mn_rng;
        append_capture(vecs, "mn_mid", mn_mid_cap, force_hex);

        vecs << ",\"pc_v_zero\":" << (pc_valid_zero_inf ? 1 : 0)
             << ",\"pc_v_sec\":" << (pc_valid_second ? 1 : 0)
             << ",\"pc_open\":" << (pc_opened ? 1 : 0)
             << ",\"pc_ovr\":" << (pc_override ? 1 : 0)
             << ",\"pc_pitch\":" << oracle::fbits(pc_pitch)
             << ",\"pc_roll\":" << oracle::fbits(pc_roll)
             << ",\"pc_prate\":" << oracle::fbits(pc_pitch_rate)
             << ",\"pc_rrate\":" << oracle::fbits(pc_roll_rate)
             << ",\"pc_sz\":" << oracle::fbits(pc_start_z)
             << ",\"pc_nocoll\":" << (pc_no_coll ? 1 : 0)
             << ",\"pc_post_cnt\":" << pc_post_cnt
             << ",\"pc_rhp\":" << oracle::fbits(pc_rider_post_hp)
             << ",\"pc_rfall\":" << (pc_rider_fall ? 1 : 0)
             << ",\"pc_dead\":" << (pc_para_dead ? 1 : 0)
             << ",\"pc_rng\":" << pc_rng;
        append_capture(vecs, "pc_mid", pc_mid_cap, force_hex);

        vecs << "}"
             << "}";
    }

    std::ostringstream doc;
    doc << "{\n"
        << "  \"packet\": \"WP-5.3-B\",\n"
        << "  \"modules\": [\"BattleBusSlowDeathBehavior\",\"JetSlowDeathBehavior\",\"TransportContain\",\"GarrisonContain\",\"HealContain\",\"InternetHackContain\",\"MobNexusContain\",\"ParachuteContain\"],\n"
        << "  \"vector_count\": 1000,\n"
        << "  \"source_sha256\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Contain/TransportContain.cpp") << "\",\n"
        << "  \"seed0_diagnostics\": " << s0_diag << ",\n"
        << "  \"vectors\": [\n" << vecs.str() << "\n  ]\n}\n";

    oracle::write_file_atomic(out_path, doc.str());
    return 0;
}
