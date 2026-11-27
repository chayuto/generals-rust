// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for MissileLauncherBuildingUpdate (PKT-341), PointDefenseLaserUpdate
// (PKT-350) and BattlePlanUpdate (PKT-309), run on the unmodified reference modules in a
// small real world.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt341_350_309_support.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT341_350_309_Support.json
//
// Missile launcher: the real `SpecialAbility` special power modules drive the update
// through `doSpecialPower`; FX lists carry a recording nugget.
// Point defense: shots are real `Weapon` objects; only the template-level
// `fireWeaponTemplate` is replaced by a recorder (as in gen_wp55_fire.cpp).
// Battle plan: one or two strategy centers (with the real StealthDetectorUpdate and
// special power module, and none / a turretless / a turreted AI) among units of two
// players, so `Player::changeBattlePlan`, `Player::becomingTeamMember` and
// `Object::initObject` run as shipped. `aiDoCommand` is a recorder.
//
// A paralyze time of zero is only given to a layout that cannot paralyze a center: the
// reference pauses special powers even for a disable that is already over, and the port's
// disable handling (outside these modules) does not.
//
// Not covered: a point defense without a weapon template, a battle plan update whose
// power has no module (both dereference null), the vision object (its creator is never
// called), client sounds, messages, radar events and animation durations, loading a save,
// frames near the scheduler's clamp, and projectiles receiving battle plan bonuses at
// launch (`Weapon.cpp`, unported).
//
// In a row, a space-separated field equal to the previous row's is written as `~`.

#define FWT_FIRST_const ~, FWT_AS_DECL,
#define FWT_FIRST_sourceObj ~, FWT_AS_CALL,
#define FWT_SECOND_(a, b, ...) b
#define FWT_SECOND(...) FWT_SECOND_(__VA_ARGS__)
#define fireWeaponTemplate(first, ...) FWT_SECOND(FWT_FIRST_##first)(first, __VA_ARGS__)
#define FWT_AS_DECL(...) fwt_unused(__VA_ARGS__)
#define FWT_AS_CALL(...) getName(), oracle_record_shot(__VA_ARGS__)

#define DEFINE_DEATH_NAMES
#define DEFINE_WEAPONBONUSCONDITION_NAMES
#define DEFINE_WEAPONBONUSFIELD_NAMES
#define DEFINE_WEAPONCOLLIDEMASK_NAMES
#define DEFINE_WEAPONAFFECTSMASK_NAMES
#define DEFINE_WEAPONRELOAD_NAMES
#define DEFINE_WEAPONPREFIRE_NAMES
#include "PreRTS.h"
#include "Common/GameCommon.h"
#include "Common/Snapshot.h"
#include "Common/GameMemory.h"
#include "Common/AsciiString.h"
#include "GameLogic/Damage.h"
#include <vector>
#define private public
#define protected public
#include "GameLogic/Weapon.h"
#undef protected
#undef private

static void oracle_record_shot(const Object* sourceObj, WeaponSlotType wslot, Int barrel,
                               Object* victimObj, const Coord3D* victimPos, const WeaponBonus& bonus,
                               Bool isDetonation, Bool ignoreRanges, const Weapon* firingWeapon,
                               ObjectID* projectileID, Bool inflictDamage);

#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Weapon.cpp"
#undef fireWeaponTemplate

#include "oracle_wp51_world.h"

#include "Common/AcademyStats.h"
#include "Common/Module.h"
#include "Common/ModuleFactory.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/RandomValue.h"
#include "Common/SpecialPower.h"
#include "Common/Team.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BattlePlanUpdate.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/MissileLauncherBuildingUpdate.h"
#include "GameLogic/Module/PointDefenseLaserUpdate.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Module/StealthDetectorUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "Common/StateMachine.h"
#include "GameLogic/TurretAI.h"
#include "GameLogic/WeaponSet.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <string>

struct ExposedFactory : public ModuleFactory { using ModuleFactory::addModuleInternal; };

struct TurretsTag { using type = TurretAI* (AIUpdateInterface::*)[MAX_TURRETS]; };
template struct MemberAccess<TurretsTag, &AIUpdateInterface::m_turretAI>;
TurretsTag::type access_member(TurretsTag);
struct TurretAngleTag { using type = Real TurretAI::*; };
template struct MemberAccess<TurretAngleTag, &TurretAI::m_angle>;
TurretAngleTag::type access_member(TurretAngleTag);
struct TurretMachineTag { using type = TurretStateMachine* TurretAI::*; };
template struct MemberAccess<TurretMachineTag, &TurretAI::m_turretStateMachine>;
TurretMachineTag::type access_member(TurretMachineTag);
struct BombPlansTag { using type = Int Player::*; };
template struct MemberAccess<BombPlansTag, &Player::m_bombardBattlePlans>;
BombPlansTag::type access_member(BombPlansTag);
struct HoldPlansTag { using type = Int Player::*; };
template struct MemberAccess<HoldPlansTag, &Player::m_holdTheLineBattlePlans>;
HoldPlansTag::type access_member(HoldPlansTag);
struct SearchPlansTag { using type = Int Player::*; };
template struct MemberAccess<SearchPlansTag, &Player::m_searchAndDestroyBattlePlans>;
SearchPlansTag::type access_member(SearchPlansTag);
struct PlanBonusTag { using type = BattlePlanBonuses* Player::*; };
template struct MemberAccess<PlanBonusTag, &Player::m_battlePlanBonuses>;
PlanBonusTag::type access_member(PlanBonusTag);
struct AcademyTag { using type = AcademyStats Player::*; };
template struct MemberAccess<AcademyTag, &Player::m_academyStats>;
AcademyTag::type access_member(AcademyTag);
struct ChoseTag { using type = Bool AcademyStats::*; };
template struct MemberAccess<ChoseTag, &AcademyStats::m_choseAStrategyForCenter>;
ChoseTag::type access_member(ChoseTag);

static void die(const char* what) {
    std::fprintf(stderr, "oracle: %s\n", what);
    std::exit(1);
}

static uint32_t bits(float f) {
    uint32_t u;
    std::memcpy(&u, &f, 4);
    return u;
}

static uint32_t g_state = 0x34135030u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}

// --- recorders ---------------------------------------------------------------

static std::string g_shots, g_fx, g_orders;

static void oracle_record_shot(const Object* sourceObj, WeaponSlotType wslot, Int, Object* victimObj,
                               const Coord3D*, const WeaponBonus&, Bool, Bool, const Weapon*,
                               ObjectID* projectileID, Bool) {
    std::ostringstream s;
    s << '{' << sourceObj->getID() << ':' << static_cast<int>(wslot) << ':'
      << (victimObj ? victimObj->getID() : INVALID_ID) << '}';
    g_shots += s.str();
    if (projectileID) *projectileID = INVALID_ID;
}

class RecNugget : public FXNugget {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(RecNugget, "OracleSupportNugget", 16, 16)
public:
    const char* name = "";
    void doFXPos(const Coord3D* primary, const Matrix3D*, const Real, const Coord3D*, const Real) const override {
        std::ostringstream s;
        s << '{' << name << ':' << bits(primary->x) << ':' << bits(primary->y) << ':' << bits(primary->z) << '}';
        g_fx += s.str();
    }
};
RecNugget::~RecNugget() {}

class RecAI : public AIUpdateInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(RecAI, "OracleSupportRecAI", 64, 64)
public:
    RecAI(Thing* thing, const ModuleData* data) : AIUpdateInterface(thing, data) {}
    void aiDoCommand(const AICommandParms* parms) override {
        std::ostringstream s;
        s << '{' << getObject()->getID() << ':'
          << (parms->m_cmd == AICMD_IDLE ? 3 : 100 + static_cast<int>(parms->m_cmd)) << ':'
          << static_cast<int>(parms->m_cmdSource) << '}';
        g_orders += s.str();
    }
};
RecAI::~RecAI() {}
static Module* make_rec_ai(Thing* t, const ModuleData* d) { return newInstance(RecAI)(t, d); }

// --- fixtures ----------------------------------------------------------------

static const char* const kFxNames[5] = {"P341_FxOpening", "P341_FxOpen", "P341_FxWaiting", "P341_FxClosing",
                                        "P341_FxClosed"};

// door open, wait open, close times (ms); reload (ms); FX mask
struct Silo { int open, wait, close, reload, fx; };
static const Silo kSilos[] = {
    {1000, 500, 1000, 3000, 31}, {2000, 0, 3000, 6000, 31}, {5000, 1000, 100, 2000, 0},
    {0, 0, 0, 1000, 21},         {600, 2000, 4000, 2500, 10}, {100, 100, 100, 0, 31},
};
static const int kSiloCount = 6;

static const char* const kPdWeapons[4] = {
    "  AttackRange = 60\n  DelayBetweenShots = 100\n",
    "  AttackRange = 100\n  DelayBetweenShots = Min:33 Max:300\n  AntiGround = No\n  AntiAirborneVehicle = Yes\n",
    "  AttackRange = 40\n  DelayBetweenShots = 0\n  AntiSmallMissile = Yes\n",
    "  AttackRange = 80\n  DelayBetweenShots = Min:0 Max:100\n  AntiGround = Yes\n  AntiBallisticMissile = Yes\n",
};
// weapon, scan rate (ms), primary, secondary, velocity factor
struct Pd { int weapon; int rate; const char* primary; const char* secondary; const char* factor; };
static const Pd kPds[] = {
    {0, 0, "INFANTRY", "VEHICLE", "0"},
    {1, 100, "BALLISTIC_MISSILE SMALL_MISSILE", "VEHICLE AIRCRAFT", "2.5"},
    {2, 500, "SMALL_MISSILE", "INFANTRY", "0"},
    {3, 200, "VEHICLE", "INFANTRY SMALL_MISSILE", "10"},
    {0, 1000, "INFANTRY VEHICLE", "", "1"},
};
static const int kPdCount = 5;
static const char* const kTargets[] = {
    "INFANTRY", "VEHICLE", "INFANTRY VEHICLE", "PROJECTILE SMALL_MISSILE", "AIRCRAFT VEHICLE",
    "STRUCTURE IMMOBILE VEHICLE", "BALLISTIC_MISSILE PROJECTILE", "STRUCTURE",
};
static const int kTargetCount = 8;

// health scalar, change type, strategy sight scalar, detects, armor scalar, member sight
// scalar, paralyze (ms), idle (ms), anim (ms), valid, invalid, AI (0 none, 1 plain,
// 2 turret), stealth detector
struct Center {
    const char* health; const char* change; const char* sight; int detects; const char* armor;
    const char* member_sight; int paralyze; int idle; int anim; const char* valid; const char* invalid;
    int ai; int detector;
};
static const Center kCenters[] = {
    {"2.0", "PRESERVE_RATIO", "2.0", 1, "0.9", "1.2", 500, 0, 200, "INFANTRY VEHICLE", "AIRCRAFT", 2, 1},
    {"1.5", "ADD_CURRENT_HEALTH_TOO", "1.5", 0, "0.5", "1.0", 300, 100, 100, "INFANTRY VEHICLE STRUCTURE", "", 2, 1},
    {"1.0", "FULLY_HEAL", "1.0", 1, "1.0", "0.0", 1000, 200, 0, "VEHICLE STRUCTURE", "", 0, 1},
    {"3.0", "FULLY_HEAL", "0.7", 1, "0.0", "2.0", 0, 0, 66, "INFANTRY", "", 1, 0},
    {"0.0", "SAME_CURRENTHEALTH", "3.0", 0, "1.3", "1.5", 100, 66, 133, "INFANTRY VEHICLE AIRCRAFT", "STRUCTURE", 2, 0},
};
static const int kCenterCount = 5;
static const char* const kMembers[] = {"INFANTRY", "VEHICLE", "AIRCRAFT VEHICLE", "STRUCTURE", "PROJECTILE"};
static const int kMemberCount = 5;

static std::string fixture_ini() {
    std::ostringstream s;
    for (const char* fx : kFxNames) s << "FXList " << fx << "\nEnd\n";
    for (int i = 0; i < kSiloCount; ++i) {
        s << "SpecialPower P341_A" << i << "\n  Enum = SPECIAL_SCUD_STORM\n  ReloadTime = " << kSilos[i].reload << "\nEnd\n";
        s << "SpecialPower P341_B" << i << "\n  Enum = SPECIAL_NEUTRON_MISSILE\n  ReloadTime = 100\nEnd\n";
    }
    s << "SpecialPower P309_Power\n  Enum = SPECIAL_CHANGE_BATTLE_PLANS\n  ReloadTime = 0\nEnd\n";
    for (int i = 0; i < 4; ++i)
        s << "Weapon P350_W" << i << "\n  PrimaryDamage = 10\n" << kPdWeapons[i] << "End\n";
    s << "Weapon P309_Gun\n  PrimaryDamage = 10\n  AttackRange = 100\n  DelayBetweenShots = 100\nEnd\n";

    for (int i = 0; i < kSiloCount; ++i) {
        const Silo& v = kSilos[i];
        s << "Object P341_Silo" << i << "\n  KindOf = PRELOAD STRUCTURE\n"
          << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n"
          << "  Behavior = SpecialAbility ModuleTag_SpA\n    SpecialPowerTemplate = P341_A" << i << "\n  End\n"
          << "  Behavior = SpecialAbility ModuleTag_SpB\n    SpecialPowerTemplate = P341_B" << i << "\n  End\n"
          << "  Behavior = MissileLauncherBuildingUpdate ModuleTag_M\n    SpecialPowerTemplate = P341_A" << i
          << "\n    DoorOpenTime = " << v.open << "\n    DoorWaitOpenTime = " << v.wait
          << "\n    DoorCloseTime = " << v.close << "\n";
        static const char* const kFields[5] = {"DoorOpeningFX", "DoorOpenFX", "DoorWaitingToCloseFX",
                                               "DoorClosingFX", "DoorClosedFX"};
        for (int f = 0; f < 5; ++f)
            if (v.fx & (1 << f)) s << "    " << kFields[f] << " = " << kFxNames[f] << "\n";
        s << "  End\nEnd\n";
    }
    for (int i = 0; i < kPdCount; ++i) {
        const Pd& v = kPds[i];
        s << "Object P350_Laser" << i << "\n  KindOf = PRELOAD STRUCTURE\n"
          << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
          << "  Behavior = PointDefenseLaserUpdate ModuleTag_L\n    WeaponTemplate = P350_W" << v.weapon
          << "\n    ScanRate = " << v.rate << "\n    ScanRange = 150\n    PrimaryTargetTypes = " << v.primary << "\n";
        if (v.secondary[0]) s << "    SecondaryTargetTypes = " << v.secondary << "\n";
        s << "    PredictTargetVelocityFactor = " << v.factor << "\n  End\nEnd\n";
    }
    for (int i = 0; i < kTargetCount; ++i)
        s << "Object P350_Target" << i << "\n  KindOf = PRELOAD " << kTargets[i] << "\n"
          << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\nEnd\n";

    for (int i = 0; i < kCenterCount; ++i) {
        const Center& v = kCenters[i];
        s << "Object P309_Center" << i << "\n  KindOf = PRELOAD STRUCTURE\n  VisionRange = 100\n"
          << "  ShroudClearingRange = 130\n"
          << "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY P309_Gun\n  End\n"
          << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 700\n  End\n";
        if (v.ai == 1) s << "  Behavior = OracleSupportAI ModuleTag_AI\n  End\n";
        if (v.ai == 2)
            s << "  Behavior = OracleSupportAI ModuleTag_AI\n    Turret\n      ControlledWeaponSlots = PRIMARY\n    End\n  End\n";
        if (v.detector) s << "  Behavior = StealthDetectorUpdate ModuleTag_Det\n    DetectionRate = 500\n  End\n";
        s << "  Behavior = SpecialAbility ModuleTag_Sp\n    SpecialPowerTemplate = P309_Power\n  End\n"
          << "  Behavior = BattlePlanUpdate ModuleTag_Plan\n    SpecialPowerTemplate = P309_Power\n"
          << "    BombardmentPlanAnimationTime = " << v.anim << "\n    HoldTheLinePlanAnimationTime = " << v.anim + 100
          << "\n    SearchAndDestroyPlanAnimationTime = " << v.anim * 2
          << "\n    TransitionIdleTime = " << v.idle << "\n    BattlePlanChangeParalyzeTime = " << v.paralyze
          << "\n    ValidMemberKindOf = " << v.valid << "\n";
        if (v.invalid[0]) s << "    InvalidMemberKindOf = " << v.invalid << "\n";
        s << "    HoldTheLinePlanArmorDamageScalar = " << v.armor
          << "\n    SearchAndDestroyPlanSightRangeScalar = " << v.member_sight
          << "\n    StrategyCenterSearchAndDestroySightRangeScalar = " << v.sight
          << "\n    StrategyCenterSearchAndDestroyDetectsStealth = " << (v.detects ? "Yes" : "No")
          << "\n    StrategyCenterHoldTheLineMaxHealthScalar = " << v.health
          << "\n    StrategyCenterHoldTheLineMaxHealthChangeType = " << v.change << "\n  End\nEnd\n";
    }
    for (int i = 0; i < kMemberCount; ++i)
        s << "Object P309_Member" << i << "\n  KindOf = PRELOAD " << kMembers[i] << "\n  VisionRange = 80\n"
          << "  ShroudClearingRange = 90\n"
          << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 150\n  End\nEnd\n";
    return s.str();
}

template <typename T>
static T* find_behavior(Object* obj, int which = 0) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (T* typed = dynamic_cast<T*>(*m)) {
            if (which-- == 0) return typed;
        }
    }
    return nullptr;
}

static std::string delta(std::vector<std::string>& last, const std::string& full) {
    std::vector<std::string> now;
    std::istringstream in(full);
    for (std::string f; in >> f;) now.push_back(f);
    std::string out;
    for (size_t i = 0; i < now.size(); ++i) {
        if (i) out += ' ';
        out += (i < last.size() && last[i] == now[i]) ? std::string("~") : now[i];
    }
    last = now;
    return out;
}

static std::string save_crc(Snapshot* s) {
    std::ostringstream out;
    out << oracle_wp51::World::snapshot_save_hex(s) << ':' << oracle_wp51::World::snapshot_crc(s);
    return out.str();
}

static std::string take(std::string& log) {
    std::string out = log.empty() ? "-" : log;
    log.clear();
    return out;
}

static void destroy_all(oracle_wp51::World& w, const std::vector<ObjectID>& ids) {
    for (ObjectID id : ids)
        if (Object* o = TheGameLogic->findObjectByID(id)) TheGameLogic->destroyObject(o);
    w.flush_destroy_list();
}

static void emit(FILE* out, const std::vector<std::string>& ops, const std::vector<std::string>& rows) {
    std::fprintf(out, "\"ops\":[");
    for (size_t i = 0; i < ops.size(); ++i) std::fprintf(out, "%s%s", i ? "," : "", ops[i].c_str());
    std::fprintf(out, "],\"rows\":[");
    for (size_t i = 0; i < rows.size(); ++i) std::fprintf(out, "%s\"%s\"", i ? "," : "", rows[i].c_str());
    std::fprintf(out, "]}");
}

static std::string op_text(int code, long a, long b, long c) {
    std::ostringstream s;
    s << '[' << code << ',' << a << ',' << b << ',' << c << ']';
    return s.str();
}

// --- missile launcher -------------------------------------------------------

static std::string silo_row(Object* o, MissileLauncherBuildingUpdate* m) {
    std::ostringstream out;
    const ModelConditionFlags& f = o->getDrawable()->getModelConditionFlags();
    out << save_crc(m) << ' ' << save_crc(find_behavior<SpecialPowerModule>(o, 0)) << ' '
        << save_crc(find_behavior<SpecialPowerModule>(o, 1)) << ' '
        << f.test(MODELCONDITION_DOOR_1_OPENING) << f.test(MODELCONDITION_DOOR_1_WAITING_OPEN)
        << f.test(MODELCONDITION_DOOR_1_WAITING_TO_CLOSE) << f.test(MODELCONDITION_DOOR_1_CLOSING) << ' '
        << m->isActive() << m->isPowerCurrentlyInUse() << ' ' << take(g_fx);
    return out.str();
}

static void silo_vector(oracle_wp51::World& w, FILE* out, int v) {
    const int variant = v % kSiloCount;
    UnsignedInt frame = 1 + pick(5000);
    w.set_frame(frame);
    const Coord3D at = {static_cast<float>(100 + pick(500)), static_cast<float>(100 + pick(500)), 0.0f};
    const std::string name = "P341_Silo" + std::to_string(variant);
    Object* o = w.spawn_object(name.c_str(), w.team_a, at);
    MissileLauncherBuildingUpdate* m = find_behavior<MissileLauncherBuildingUpdate>(o);
    if (!m) die("missile module");
    const ObjectID id = o->getID();
    const int construction = pick(5) == 0 ? 1 : 0;
    if (construction) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
    g_fx.clear();
    std::vector<std::string> last, ops, rows;
    std::fprintf(out, "{\"kind\":0,\"variant\":%d,\"frame\":%u,\"id\":%u,\"x\":%u,\"y\":%u,\"construction\":%d,",
                 variant, frame, id, bits(at.x), bits(at.y), construction);
    const std::string spawn = delta(last, silo_row(o, m));
    std::fprintf(out, "\"spawn\":\"%s\",", spawn.c_str());
    const int count = 8 + static_cast<int>(pick(25));
    for (int k = 0; k < count; ++k) {
        static const int kCodes[] = {0, 0, 0, 0, 0, 0, 1, 1, 2, 3, 3, 4, 5, 6};
        const int code = kCodes[pick(14)];
        long a = 0;
        SpecialPowerModule* spa = find_behavior<SpecialPowerModule>(o, 0);
        SpecialPowerModule* spb = find_behavior<SpecialPowerModule>(o, 1);
        w.set_frame(frame);
        switch (code) {
        case 0:
            a = 1 + pick(pick(3) ? 12 : 120);
            for (long t = 0; t < a; ++t) {
                w.set_frame(++frame);
                if (m->update() != UPDATE_SLEEP_NONE) die("sleep");
            }
            break;
        case 1: spa->doSpecialPower(0); break;
        case 2: spb->doSpecialPower(0); break;
        case 3:  // the ready frame moves
            a = static_cast<long>(pick(400)) - 100;
            if (static_cast<long>(frame) + a < 0) a = 0;
            spa->setReadyFrame(static_cast<UnsignedInt>(static_cast<long>(frame) + a));
            break;
        case 4:
            a = pick(2);
            spa->pauseCountdown(a != 0);
            break;
        case 5:
            a = pick(2);
            if (a) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
            else o->clearStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
            break;
        case 6:  // the update asked directly
            a = pick(2);
            a = (a << 1) | static_cast<long>(m->initiateIntentToDoSpecialPower(
                               TheSpecialPowerStore->findSpecialPowerTemplate(
                                   AsciiString((std::string(a ? "P341_B" : "P341_A") + std::to_string(variant)).c_str())),
                               nullptr, nullptr, nullptr, 0));
            break;
        }
        ops.push_back(op_text(code, a, 0, 0));
        rows.push_back(delta(last, silo_row(o, m)));
    }
    emit(out, ops, rows);
    destroy_all(w, {id});
}

// --- point defense ----------------------------------------------------------

static std::string laser_row(Object* o, PointDefenseLaserUpdate* m, const std::vector<ObjectID>& targets,
                             uint32_t ret) {
    std::ostringstream out;
    out << save_crc(m) << ' ' << GetGameLogicRandomSeedCRC() << ' ' << take(g_shots) << ' ' << ret << ' ';
    for (ObjectID id : targets) {
        Object* t = TheGameLogic->findObjectByID(id);
        out << (!t ? 'x' : t->isEffectivelyDead() ? 'd' : 'a');
    }
    return out.str();
}

static void laser_vector(oracle_wp51::World& w, FILE* out, int v) {
    const int variant = v % kPdCount;
    InitRandom(0x350000u + v);
    UnsignedInt frame = 1 + pick(5000);
    w.set_frame(frame);
    const Coord3D centre = {500.0f, 500.0f, 0.0f};
    const std::string name = "P350_Laser" + std::to_string(variant);
    Object* o = w.spawn_object(name.c_str(), w.team_a, centre);
    PointDefenseLaserUpdate* m = find_behavior<PointDefenseLaserUpdate>(o);
    if (!m) die("laser module");
    const Real range = TheWeaponStore->findWeaponTemplate(
                           AsciiString(("P350_W" + std::to_string(kPds[variant].weapon)).c_str()))
                           ->getAttackRange(WeaponBonus());
    Team* teams[3] = {w.team_neutral, w.team_a, w.team_b};

    auto position = [&]() {
        Coord3D p = centre;
        const uint32_t how = pick(8);
        if (how == 0) p.x += range;  // exactly on the range
        else if (how == 1) p.y -= 150.0f;  // exactly on the scan range
        else if (how == 2) p.x += range + 0.25f;
        else if (how == 3) { p.x += 30.0f; }  // shared places, so distances tie
        else {
            p.x += static_cast<float>(static_cast<int>(pick(321)) - 160);
            p.y += static_cast<float>(static_cast<int>(pick(321)) - 160) * (pick(2) ? 0.25f : 1.0f);
        }
        p.z = pick(4) == 0 ? 25.0f : 0.0f;
        return p;
    };
    auto status_mask = [](uint32_t f) {
        ObjectStatusMaskType m_;
        if (f & 1) m_.set(OBJECT_STATUS_AIRBORNE_TARGET);
        if (f & 2) m_.set(OBJECT_STATUS_STEALTHED);
        if (f & 4) m_.set(OBJECT_STATUS_DETECTED);
        if (f & 8) m_.set(OBJECT_STATUS_DISGUISED);
        return m_;
    };
    auto random_status = [&]() -> uint32_t {
        uint32_t f = pick(2);
        if (pick(4) == 0) f |= 2;
        if (pick(5) == 0) f |= 4;
        if (pick(8) == 0) f |= 8;
        return f;
    };

    std::vector<ObjectID> targets;
    std::ostringstream text;
    const int count = 1 + static_cast<int>(pick(8));
    for (int i = 0; i < count; ++i) {
        const int kind = pick(kTargetCount);
        const int team = pick(6) == 0 ? static_cast<int>(pick(2)) : 2;
        const Coord3D p = position();
        const uint32_t flags = random_status();
        const std::string tname = "P350_Target" + std::to_string(kind);
        Object* t = w.spawn_object(tname.c_str(), teams[team], p);
        t->setStatus(status_mask(flags));
        targets.push_back(t->getID());
        text << (i ? ";" : "") << kind << ',' << team << ',' << bits(p.x) << ',' << bits(p.y) << ',' << bits(p.z)
             << ',' << flags;
    }
    ThePartitionManager->update();
    g_shots.clear();
    std::vector<std::string> last, ops, rows;
    std::fprintf(out, "{\"kind\":1,\"variant\":%d,\"frame\":%u,\"id\":%u,\"seed\":%u,\"targets\":\"%s\",",
                 variant, frame, o->getID(), 0x350000u + v, text.str().c_str());
    const std::string spawn = delta(last, laser_row(o, m, targets, 0));
    std::fprintf(out, "\"spawn\":\"%s\",", spawn.c_str());
    const int op_count = 8 + static_cast<int>(pick(25));
    for (int k = 0; k < op_count; ++k) {
        static const int kCodes[] = {0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 3, 4, 5, 1, 0, 6};
        int code = kCodes[pick(18)];
        if (code == 6 && pick(4) != 0) code = 0;
        long a = 0, b = 0, c = 0;
        uint32_t ret = 0;
        w.set_frame(frame);
        const int who = static_cast<int>(pick(targets.size()));
        Object* t = TheGameLogic->findObjectByID(targets[who]);
        if (code >= 1 && code <= 5 && !t) code = 0;
        switch (code) {
        case 0:
            a = 1 + pick(pick(3) ? 3 : 20);
            for (long s = 0; s < a; ++s) {
                w.set_frame(++frame);
                ret = static_cast<uint32_t>(m->update());
            }
            break;
        case 1: {
            const Coord3D p = position();
            a = who;
            b = bits(p.x);
            c = bits(p.y);
            Coord3D q = p;
            q.z = t->getPosition()->z;
            t->setPosition(&q);
            ThePartitionManager->update();
            break;
        }
        case 2:
            a = who;
            b = random_status();
            t->clearStatus(status_mask(15));
            t->setStatus(status_mask(static_cast<uint32_t>(b)));
            break;
        case 3: a = who; t->kill(); break;
        case 4:
            a = who;
            TheGameLogic->destroyObject(t);
            w.flush_destroy_list();
            ThePartitionManager->update();
            break;
        case 5:
            a = who;
            b = pick(3);
            t->setTeam(teams[b]);
            break;
        case 6: o->kill(); break;
        }
        ops.push_back(op_text(code, a, b, c));
        rows.push_back(delta(last, laser_row(o, m, targets, ret)));
    }
    emit(out, ops, rows);
    std::vector<ObjectID> all = targets;
    all.push_back(o->getID());
    destroy_all(w, all);
    ThePartitionManager->update();
}

// --- battle plan ------------------------------------------------------------

static void reset_player(Player* p) {
    p->*access_member(BombPlansTag()) = 0;
    p->*access_member(HoldPlansTag()) = 0;
    p->*access_member(SearchPlansTag()) = 0;
    if (p->*access_member(PlanBonusTag())) {
        deleteInstance(p->*access_member(PlanBonusTag()));
        p->*access_member(PlanBonusTag()) = nullptr;
    }
    (p->*access_member(AcademyTag())).*access_member(ChoseTag()) = FALSE;
}

static std::string player_text(Player* p) {
    std::ostringstream s;
    s << p->*access_member(BombPlansTag()) << ',' << p->*access_member(HoldPlansTag()) << ','
      << p->*access_member(SearchPlansTag()) << ','
      << static_cast<int>((p->*access_member(AcademyTag())).*access_member(ChoseTag()));
    const BattlePlanBonuses* b = p->*access_member(PlanBonusTag());
    if (b)
        s << ',' << bits(b->m_armorScalar) << ',' << bits(b->m_sightRangeScalar) << ',' << b->m_bombardment << ','
          << b->m_holdTheLine << ',' << b->m_searchAndDestroy;
    return s.str();
}

static std::string unit_text(Object* o) {
    std::ostringstream s;
    BodyModuleInterface* body = o->getBodyModule();
    s << bits(body->getHealth()) << ',' << bits(body->getMaxHealth()) << ',' << bits(body->getDamageScalar()) << ','
      << bits(o->getVisionRange()) << ',' << bits(o->getShroudClearingRange()) << ','
      << o->getWeaponBonusCondition() << ',' << o->getDisabledUntil(DISABLED_PARALYZED) << ','
      << o->getControllingPlayer()->getPlayerIndex();
    return s.str();
}

static TurretAI* turret_of(Object* o) {
    AIUpdateInterface* ai = o->getAI();
    return ai ? (ai->*access_member(TurretsTag()))[0] : nullptr;
}

struct Plans {
    std::vector<ObjectID> centers, members;
};

static std::string plan_row(const Plans& p, Player* a, Player* b) {
    std::ostringstream out;
    out << player_text(a) << ' ' << player_text(b) << ' ' << take(g_orders);
    for (ObjectID id : p.centers) {
        Object* o = TheGameLogic->findObjectByID(id);
        if (!o) {
            out << " x x x";
            continue;
        }
        BattlePlanUpdate* m = find_behavior<BattlePlanUpdate>(o);
        StealthDetectorUpdate* det = find_behavior<StealthDetectorUpdate>(o);
        const ModelConditionFlags& f = o->getDrawable()->getModelConditionFlags();
        std::string doors;
        for (ModelConditionFlagType flag :
             {MODELCONDITION_DOOR_1_OPENING, MODELCONDITION_DOOR_1_WAITING_TO_CLOSE, MODELCONDITION_DOOR_1_CLOSING,
              MODELCONDITION_DOOR_2_OPENING, MODELCONDITION_DOOR_2_WAITING_TO_CLOSE, MODELCONDITION_DOOR_2_CLOSING,
              MODELCONDITION_DOOR_3_OPENING, MODELCONDITION_DOOR_3_WAITING_TO_CLOSE, MODELCONDITION_DOOR_3_CLOSING})
            doors += f.test(flag) ? '1' : '0';
        out << ' ' << save_crc(m) << ' ' << (det ? save_crc(det) : std::string("-")) << ' ' << doors << ',' << unit_text(o) << ','
            << o->testWeaponSetFlag(WEAPONSET_VETERAN) << o->isCurWeaponLocked() << ','
            << static_cast<int>(m->getActiveBattlePlan()) << ',' << static_cast<int>(m->getCommandOption()) << ','
            << m->isActive();
        if (TurretAI* t = turret_of(o))
            out << ',' << t->isTurretEnabled()
                << ((t->*access_member(TurretMachineTag()))->getCurrentStateID() == TURRETAI_RECENTER);
    }
    for (ObjectID id : p.members) {
        Object* o = TheGameLogic->findObjectByID(id);
        out << ' ' << (o ? unit_text(o) : std::string("x"));
    }
    return out.str();
}

static void plan_vector(oracle_wp51::World& w, FILE* out, int v) {
    InitRandom(0x309000u + v);
    UnsignedInt frame = 1 + pick(5000);
    w.set_frame(frame);
    Player* pa = w.team_a->getControllingPlayer();
    Player* pb = w.team_b->getControllingPlayer();
    reset_player(pa);
    reset_player(pb);
    Team* teams[3] = {w.team_neutral, w.team_a, w.team_b};
    Plans p;
    std::ostringstream setup;
    ObjectID next = INVALID_ID;

    auto member = [&](int kind, int team, int producer) {
        const std::string name = "P309_Member" + std::to_string(kind);
        Object* o = w.spawn_object(name.c_str(), teams[team], {200.0f + 10.0f * p.members.size(), 200.0f, 0.0f});
        if (producer >= 0 && producer < static_cast<int>(p.members.size()))
            if (Object* src = TheGameLogic->findObjectByID(p.members[producer])) o->setProducer(src);
        p.members.push_back(o->getID());
        if (!next) next = o->getID();
    };
    auto center = [&](int variant, int team) {
        const std::string name = "P309_Center" + std::to_string(variant);
        Object* o = w.spawn_object(name.c_str(), teams[team], {300.0f + 40.0f * p.centers.size(), 300.0f, 0.0f});
        p.centers.push_back(o->getID());
        if (!next) next = o->getID();
    };

    const int early = static_cast<int>(pick(4));
    for (int i = 0; i < early; ++i) {
        const int kind = pick(kMemberCount), team = pick(5) == 0 ? 2 : 1,
                  producer = kind == 4 ? static_cast<int>(pick(p.members.size() + 1)) : -1;
        member(kind, team, producer);
        setup << 'm' << kind << ',' << team << ',' << producer << ';';
    }
    const int centers = 1 + (pick(3) == 0 ? 1 : 0);
    for (int i = 0; i < centers; ++i) {
        const int variant = (i == 0 || pick(2)) ? v % kCenterCount : static_cast<int>(pick(kCenterCount));
        const int team = (i > 0 && pick(5) == 0) ? 2 : 1;
        center(variant, team);
        setup << 'c' << variant << ',' << team << ";";
    }
    const int later = 1 + static_cast<int>(pick(4));
    for (int i = 0; i < later; ++i) {
        const int kind = pick(kMemberCount), team = pick(5) == 0 ? 2 : 1,
                  producer = kind == 4 ? static_cast<int>(pick(p.members.size() + 1)) : -1;
        member(kind, team, producer);
        setup << 'm' << kind << ',' << team << ',' << producer << ';';
    }
    g_orders.clear();
    std::vector<std::string> last, ops, rows;
    std::fprintf(out, "{\"kind\":2,\"frame\":%u,\"id\":%u,\"seed\":%u,\"setup\":\"%s\",", frame, next,
                 0x309000u + v, setup.str().c_str());
    const std::string spawn = delta(last, plan_row(p, pa, pb));
    std::fprintf(out, "\"spawn\":\"%s\",", spawn.c_str());
    const int op_count = 10 + static_cast<int>(pick(30));
    static const long kOptions[] = {0x2000, 0x4000, 0x8000, 0x6000, 0xC000, 0, 0x2000, 0x4000, 0x8000, 0x1000};
    for (int k = 0; k < op_count; ++k) {
        static const int kCodes[] = {0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 3, 4, 5, 6, 7, 8};
        int code = kCodes[pick(21)];
        if (code == 6 && pick(3) != 0) code = 0;
        long a = 0, b = 0, c = 0;
        w.set_frame(frame);
        const int which = static_cast<int>(pick(p.centers.size()));
        Object* o = TheGameLogic->findObjectByID(p.centers[which]);
        if ((code == 1 || code == 2 || code == 3 || code == 6) && !o) code = 0;
        switch (code) {
        case 0:  // scheduled frames of every center
            a = 1 + pick(pick(2) ? 6 : 15);
            for (long s = 0; s < a; ++s) {
                w.set_frame(++frame);
                for (ObjectID id : p.centers)
                    if (Object* x = TheGameLogic->findObjectByID(id))
                        if (find_behavior<BattlePlanUpdate>(x)->update() != UPDATE_SLEEP_NONE) die("sleep");
            }
            break;
        case 1:
            a = which;
            b = kOptions[pick(10)];
            find_behavior<SpecialPowerModule>(o)->doSpecialPower(static_cast<UnsignedInt>(b));
            break;
        case 2:  // where the turret points
            a = which;
            b = pick(2);
            if (TurretAI* t = turret_of(o)) t->*access_member(TurretAngleTag()) = b ? 0.0f : 1.0f;
            break;
        case 3:  // a capture
            a = which;
            b = o->getTeam() == w.team_a ? 2 : 1;
            o->setTeam(teams[b]);
            break;
        case 4:
            if (p.members.size() >= 9) { code = 0; a = 1; w.set_frame(++frame);
                for (ObjectID id : p.centers)
                    if (Object* x = TheGameLogic->findObjectByID(id)) find_behavior<BattlePlanUpdate>(x)->update();
                break;
            }
            a = pick(kMemberCount);
            b = pick(5) == 0 ? 2 : 1;
            c = a == 4 ? static_cast<long>(pick(p.members.size() + 1)) : -1;
            member(static_cast<int>(a), static_cast<int>(b), static_cast<int>(c));
            break;
        case 5: {  // a unit changes sides
            a = pick(p.members.size());
            b = pick(3) == 0 ? 0 : 1 + pick(2);
            if (Object* x = TheGameLogic->findObjectByID(p.members[a])) x->setTeam(teams[b]);
            break;
        }
        case 6:
            a = which;
            TheGameLogic->destroyObject(o);
            w.flush_destroy_list();
            break;
        case 7: {  // a unit leaves the world
            a = pick(p.members.size());
            if (Object* x = TheGameLogic->findObjectByID(p.members[a])) {
                TheGameLogic->destroyObject(x);
                w.flush_destroy_list();
            }
            break;
        }
        case 8:
            if (p.centers.size() >= 3) { code = 0; a = 0; break; }
            a = pick(kCenterCount);
            b = pick(5) == 0 ? 2 : 1;
            center(static_cast<int>(a), static_cast<int>(b));
            break;
        }
        ops.push_back(op_text(code, a, b, c));
        rows.push_back(delta(last, plan_row(p, pa, pb)));
    }
    emit(out, ops, rows);
    destroy_all(w, p.centers);
    destroy_all(w, p.members);
}

int main(int argc, char** argv) {
    if (argc < 2) die("usage: gen_pkt341_350_309_support <out.json>");
    oracle_wp51::World w;
    (TheModuleFactory->*(&ExposedFactory::addModuleInternal))(
        make_rec_ai, AIUpdateInterface::friend_newModuleData, MODULETYPE_BEHAVIOR,
        "OracleSupportAI", AIUpdateInterface::getInterfaceMask());
    w.load_ini(fixture_ini());
    w.init_with_neutral_player();
    // No display: an unused player observes, so shroud changes stay in the logic. FX lists
    // only play where the observer sees, so its map is revealed first.
    ThePartitionManager->revealMapForPlayerPermanently(5);
    ThePlayerList->setLocalPlayer(ThePlayerList->getNthPlayer(5));
    for (const char* name : kFxNames) {
        FXList* list = const_cast<FXList*>(TheFXListStore->findFXList(name));
        if (!list) die("fx list");
        RecNugget* n = newInstance(RecNugget);
        n->name = name;
        list->addFXNugget(n);
    }

    FILE* out = std::fopen(argv[1], "w");
    if (!out) return 1;
    const int kTotal = 3000;
    std::fprintf(out, "{\"vector_count\":%d,\"vectors\":[", kTotal);
    for (int v = 0; v < kTotal; ++v) {
        std::fprintf(out, "%s\n", v ? "," : "");
        const int n = v / 3;
        if (v % 3 == 0) silo_vector(w, out, n);
        else if (v % 3 == 1) laser_vector(w, out, n);
        else plan_vector(w, out, n);
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return 0;
}
