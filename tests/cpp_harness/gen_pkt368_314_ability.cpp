// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for SpecialAbilityUpdate (PKT-368) and CommandButtonHuntUpdate
// (PKT-314), run on the unmodified reference modules in a small real world: one
// ability user (the hunter), targets, riders in a garrison, mines and booby traps,
// with the real SpecialAbility power modules, StealthUpdate, PhysicsBehavior,
// StickyBombUpdate, LaserUpdate and ActionManager.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt368_314_ability.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT368_314_Ability.json
//
// Test doubles: the AI update records the orders it is given and answers
// isIdle/isMoving/last command source from flags; terrain line of sight is a wall
// at x = 500; two booby traps kill their victim from a die module when detonated;
// the hunter's crate collides (for the ActionManager only) ignore contact.
// Player B is the local player and nothing clears the shroud. The hunter steps
// before its abilities, and the ActionManager answers are recorded just before it.
//
// Not covered: bone positions (no models), radar infiltration events and the
// drawable calls (animation time, capture flash colour, lasers: client only; the
// flash is observed by its tick sound), the pathfinder's view blocking, skill
// point scaling and rank ups, templates without a SpecialPowerTemplate (the
// reference dereferences null), commands with no special power while packing
// (same), ability updates of a disabled owner (the scheduler's job), and loading
// a save into an ability module (the hunt module is saved and loaded).

#include "oracle_wp51_world.h"

#include "Common/AcademyStats.h"
#include "Common/AudioSettings.h"
#include "Common/MiscAudio.h"
#include "Common/ActionManager.h"
#include "Common/Money.h"
#include "Common/PlayerList.h"
#include "Common/ScoreKeeper.h"
#include "GameClient/ControlBar.h"
#include "GameClient/Eva.h"
#include "GameClient/ParticleSys.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/CommandButtonHuntUpdate.h"
#include "GameLogic/Module/ConvertToCarBombCrateCollide.h"
#include "GameLogic/Module/ConvertToHijackedVehicleCrateCollide.h"
#include "GameLogic/Module/SabotageSupplyDropzoneCrateCollide.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/DestroyDie.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/SpecialAbilityUpdate.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Module/StickyBombUpdate.h"
#include "GameLogic/WeaponSet.h"

#include <algorithm>
#include <cstdio>
#include <map>
#include <string>
#include <vector>

#define PEEK(TAG, TYPE, CLASS, MEMBER)                                 \
    struct TAG { using type = TYPE CLASS::*; };                        \
    template struct MemberAccess<TAG, &CLASS::MEMBER>;                 \
    TYPE CLASS::* access_member(TAG);

PEEK(AbIndexTag, Int, UpdateModule, m_indexInLogic)
PEEK(AbMovingTag, Bool, AIUpdateInterface, m_isMoving)
PEEK(AbIgnoreTag, ObjectID, AIUpdateInterface, m_ignoreObstacleID)
PEEK(AbPitchTag, Real, PhysicsBehavior, m_pitchRate)
PEEK(AbPhysFlagsTag, Int, PhysicsBehavior, m_flags)
PEEK(AbSinkTag, ObjectID, ExperienceTracker, m_experienceSink)
PEEK(AbCapturedTag, UnsignedInt, AcademyStats, m_structuresCaptured)
PEEK(AbLifetimeTag, UnsignedInt, ParticleSystem, m_systemLifetimeLeft)
PEEK(AbModifierTag, Real, TAiData, m_attackPriorityDistanceModifier)
using AbEvaFlags = Bool[EVA_COUNT];
PEEK(AbEvaTag, AbEvaFlags, Eva, m_shouldPlay)
struct AbButtonTag { using type = const CommandButton* CommandButtonHuntUpdate::*; };
template struct MemberAccess<AbButtonTag, &CommandButtonHuntUpdate::m_commandButton>;
const CommandButton* CommandButtonHuntUpdate::* access_member(AbButtonTag);

auto ab_overrides_member();
template <auto M>
struct AbGrab {
    friend auto ab_overrides_member() { return M; }
};
template struct AbGrab<&GameLogic::m_controlBarOverrides>;

static uint32_t g_state = 0x36800314u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
static bool chance(uint32_t percent) { return pick(100) < percent; }
static const char* choose(std::initializer_list<const char*> v) {
    return *(v.begin() + pick(static_cast<uint32_t>(v.size())));
}

// ---- test doubles ----------------------------------------------------------

static std::vector<std::string> g_ai_log;
static std::vector<std::string> g_audio_log;
static ObjectID g_user_id = INVALID_ID;

static std::string bits(Real f) {
    uint32_t u;
    std::memcpy(&u, &f, 4);
    char b[16];
    std::snprintf(b, sizeof b, "%08x", u);
    return b;
}
static std::string pos_bits(const Coord3D& p) { return bits(p.x) + bits(p.y) + bits(p.z); }
static std::string num(uint64_t v) { return std::to_string(v); }

class OracleAI : public AIUpdateInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(OracleAI, "OracleAI", 16, 16)
public:
    Bool m_scriptedIdle;
    OracleAI(Thing* t, const ModuleData* d) : AIUpdateInterface(t, d), m_scriptedIdle(TRUE) {}
    Bool isIdle() const override { return m_scriptedIdle; }
    UpdateSleepTime update() override { return UPDATE_SLEEP_FOREVER; }
    void aiDoCommand(const AICommandParms* p) override {
        const char* name = nullptr;
        switch (p->m_cmd) {
            case AICMD_IDLE: name = "idle"; break;
            case AICMD_BUSY: name = "busy"; break;
            case AICMD_HUNT: name = "hunt"; break;
            case AICMD_ENTER: name = "enter"; break;
            case AICMD_MOVE_TO_OBJECT: name = "moveObj"; break;
            case AICMD_MOVE_TO_POSITION: name = "movePos"; break;
            case AICMD_ATTACK_OBJECT: name = "attack"; break;
            case AICMD_FACE_OBJECT: name = "faceObj"; break;
            case AICMD_FACE_POSITION: name = "facePos"; break;
            default: break;
        }
        std::string s = name ? name : ("cmd" + num(p->m_cmd));
        s += ":" + num(p->m_cmdSource);
        if (p->m_cmd == AICMD_MOVE_TO_OBJECT || p->m_cmd == AICMD_ATTACK_OBJECT ||
            p->m_cmd == AICMD_FACE_OBJECT || p->m_cmd == AICMD_ENTER)
            s += ":" + num(p->m_obj ? p->m_obj->getID() : 0);
        if (p->m_cmd == AICMD_MOVE_TO_POSITION || p->m_cmd == AICMD_FACE_POSITION)
            s += ":" + pos_bits(p->m_pos);
        if (p->m_cmd == AICMD_ATTACK_OBJECT) s += ":" + num(static_cast<uint32_t>(p->m_intValue));
        if (getObject()->getID() == g_user_id) g_ai_log.push_back(s);
        friend_setLastCommandSource(p->m_cmdSource);
    }
};
OracleAI::~OracleAI() {}
static Module* new_oracle_ai(Thing* t, const ModuleData* d) { return newInstance(OracleAI)(t, d); }

// A booby trap whose template says so kills its victim when it dies.
class OracleDestroyDie : public DestroyDie {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(OracleDestroyDie, "OracleDestroyDie", 64, 64)
public:
    OracleDestroyDie(Thing* t, const ModuleData* d) : DestroyDie(t, d) {}
    void onDie(const DamageInfo* damage) override {
        const AsciiString& name = getObject()->getTemplate()->getName();
        Object* victim = nullptr;
        if (name == "AB_TrapKill") victim = TheGameLogic->findObjectByID(getObject()->getProducerID());
        if (name == "AB_TrapKillUser") victim = TheGameLogic->findObjectByID(g_user_id);
        if (victim) victim->setEffectivelyDead(TRUE);
        DestroyDie::onDie(damage);
    }
};
OracleDestroyDie::~OracleDestroyDie() {}
static Module* new_oracle_die(Thing* t, const ModuleData* d) { return newInstance(OracleDestroyDie)(t, d); }

// The hunter carries crate collides only so the ActionManager can answer; touching
// another object must not use them up.
#define IDLE_COLLIDE(CLASS, BASE)                                                      \
    class CLASS : public BASE {                                                        \
        MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(CLASS, #CLASS, 64, 64)                   \
    public:                                                                            \
        CLASS(Thing* t, const ModuleData* d) : BASE(t, d) {}                           \
        void onCollide(Object*, const Coord3D*, const Coord3D*) override {}            \
    };                                                                                 \
    CLASS::~CLASS() {}                                                                 \
    static Module* new_##CLASS(Thing* t, const ModuleData* d) { return newInstance(CLASS)(t, d); }
IDLE_COLLIDE(IdleHijack, ConvertToHijackedVehicleCrateCollide)
IDLE_COLLIDE(IdleCarBomb, ConvertToCarBombCrateCollide)
IDLE_COLLIDE(IdleSabotage, SabotageSupplyDropzoneCrateCollide)

struct FactoryAccess : public ModuleFactory {
    using ModuleFactory::addModuleInternal;
};

class RecordingAudio : public oracle_wp51::NullAudioManager {
    Bool m_defecting = FALSE;

public:
    AudioHandle addAudioEvent(const AudioEventRTS* e) override {
        const std::string name = e->getEventName().str();
        // `Object::defect` plays the same tick as a capture flash, right after its voice.
        if (name == "AS_Defect") {
            m_defecting = TRUE;
        } else if (name == "AS_Tick" && m_defecting) {
            m_defecting = FALSE;
        } else if (name.rfind("AS_", 0) == 0) {
            g_audio_log.push_back(name);
        }
        return 0;
    }
};

static std::vector<std::string> g_text_log;
class RecordingUI : public oracle_wp51::NullInGameUI {
public:
    void addFloatingText(const UnicodeString&, const Coord3D* pos, Color color) override {
        g_text_log.push_back(pos_bits(*pos) + ":" + num(static_cast<uint32_t>(color)));
    }
};

static Real g_wall = 0.0f;  // 0: no wall
class WallTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    Bool isClearLineOfSight(const Coord3D& a, const Coord3D& b) const override {
        if (g_wall > 0.0f && (a.x - 500.0f) * (b.x - 500.0f) < 0.0f && (a.z + b.z) * 0.5f < g_wall)
            return FALSE;
        return TRUE;
    }
};

// ---- fixtures ----------------------------------------------------------------

static const char* const kTypes[] = {
    "SPECIAL_MISSILE_DEFENDER_LASER_GUIDED_MISSILES", "SPECIAL_REMOTE_CHARGES",
    "SPECIAL_TIMED_CHARGES", "SPECIAL_HELIX_NAPALM_BOMB", "SPECIAL_HACKER_DISABLE_BUILDING",
    "SPECIAL_TANKHUNTER_TNT_ATTACK", "SPECIAL_BLACKLOTUS_CAPTURE_BUILDING",
    "SPECIAL_BLACKLOTUS_DISABLE_VEHICLE_HACK", "SPECIAL_BLACKLOTUS_STEAL_CASH_HACK",
    "SPECIAL_INFANTRY_CAPTURE_BUILDING", "SPECIAL_DISGUISE_AS_VEHICLE", "SPECIAL_BOOBY_TRAP",
    "SPECIAL_BATTLESHIP_BOMBARDMENT"};
static const int kTypeCount = 13;
static const int kPowers = kTypeCount * 2;  // reload 0 and reload > 0
static const int kUsers = 65;
static const int kMaxAbilities = 3;

struct UserVariant {
    std::vector<int> power;  // per ability: power index
    bool ai, physics, stealth;
    int weapons;
    bool hunt;
};
static UserVariant g_users[kUsers];
static std::string g_rust_users[kUsers];

static std::string power_name(int p) { return "AB_SP" + num(p); }
// 0 none, 1 a science player A has, 2 one it lacks.
static int science_of(int p) { return p % 7 == 3 ? 1 : (p % 7 == 5 ? 2 : 0); }

static std::string special_powers_ini() {
    std::string s;
    for (int p = 0; p < kPowers; ++p) {
        s += "SpecialPower " + power_name(p) + "\n  Enum = " + kTypes[p / 2] +
             "\n  ReloadTime = " + (p % 2 ? std::string(p % 4 == 1 ? "100" : "2000") : std::string("0")) + "\n" +
             (science_of(p) ? std::string("  RequiredScience = ") + (science_of(p) == 1 ? "SCIENCE_AB_HAVE" : "SCIENCE_AB_LACK") + "\n"
                            : std::string()) +
             "End\n";
    }
    return s;
}

static std::string body() {
    return "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n"
           "  Behavior = DestroyDie ModuleTag_Die\n  End\n";
}
static std::string cylinder(const char* radius, const char* height) {
    return std::string("  Geometry = CYLINDER\n  GeometryMajorRadius = ") + radius +
           "\n  GeometryHeight = " + height + "\n  GeometryIsSmall = No\n";
}
static std::string box(const char* major, const char* minor, const char* height) {
    return std::string("  Geometry = BOX\n  GeometryMajorRadius = ") + major +
           "\n  GeometryMinorRadius = " + minor + "\n  GeometryHeight = " + height +
           "\n  GeometryIsSmall = No\n";
}

struct Fixed {
    const char* name;
    std::string ini;
};
static std::vector<Fixed> fixed_objects() {
    std::vector<Fixed> v;
    auto add = [&](const char* name, const std::string& kinds, const std::string& rest) {
        v.push_back({name, std::string("Object ") + name + "\n  KindOf = PRELOAD " + kinds +
                               "\n  VisionRange = 100\n  ShroudClearingRange = 0\n" + rest + "End\n"});
    };
    add("AB_Big", "STRUCTURE IMMOBILE CAPTURABLE SELECTABLE", box("15", "15", "20") + body());
    add("AB_SmallCyl", "STRUCTURE IMMOBILE CAPTURABLE SELECTABLE", cylinder("5", "10") + body());
    add("AB_SmallBox", "STRUCTURE CAPTURABLE SELECTABLE FS_SUPPLY_DROPZONE", box("8", "8", "6") + body());
    add("AB_Vehicle", "VEHICLE SELECTABLE",
        cylinder("8", "6") + body() + "  IsTrainable = Yes\n  ExperienceRequired = 0 10 60 200\n" + "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n");
    add("AB_Garrison", "STRUCTURE IMMOBILE CAPTURABLE SELECTABLE",
        box("10", "10", "30") + body() +
            "  Behavior = GarrisonContain ModuleTag_Contain\n    ContainMax = 5\n"
            "    AllowInsideKindOf = INFANTRY\n  End\n");
    add("AB_Infantry", "INFANTRY SELECTABLE", cylinder("3", "10") + body() + "  TransportSlotCount = 1\n");
    add("AB_Truck", "VEHICLE DISGUISER SELECTABLE",
        cylinder("7", "8") + body() +
            "  Behavior = PhysicsBehavior ModuleTag_Phys\n  End\n"
            "  Behavior = StealthUpdate ModuleTag_Stealth\n    DisguisesAsTeam = Yes\n"
            "    DisguiseTransitionTime = 100\n    DisguiseRevealTransitionTime = 100\n  End\n");
    add("AB_Ball", "STRUCTURE IMMOBILE SELECTABLE",
        "  Geometry = SPHERE\n  GeometryMajorRadius = 6\n  GeometryIsSmall = No\n" + body() +
            "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 3\n"
            "    AllowInsideKindOf = INFANTRY\n  End\n");
    add("AB_Edge", "STRUCTURE CAPTURABLE SELECTABLE", box("15", "5", "6") + body());
    add("AB_Rider", "INFANTRY STEALTH_GARRISON SELECTABLE",
        cylinder("3", "10") + body() + "  TransportSlotCount = 1\n");
    add("AB_Mine", "MINE", cylinder("2", "2") + body());
    // Special objects.
    add("AB_Laser", "NO_SELECT",
        cylinder("1", "1") + body() + "  ClientUpdate = LaserUpdate ModuleTag_Laser\n  End\n");
    add("AB_Dud", "NO_SELECT", cylinder("1", "1") + body());
    const std::string sticky = "  Behavior = StickyBombUpdate ModuleTag_Sticky\n    OffsetZ = 10\n  End\n";
    add("AB_Bomb", "NO_SELECT", cylinder("1", "1") + body() + sticky);
    add("AB_Trap", "BOOBY_TRAP NO_SELECT", cylinder("1", "1") + body() + sticky);
    add("AB_TrapKill", "BOOBY_TRAP NO_SELECT", cylinder("1", "1") + body() + sticky);
    add("AB_TrapKillUser", "BOOBY_TRAP NO_SELECT", cylinder("1", "1") + body() + sticky);
    add("AB_PhysObj", "NO_SELECT",
        cylinder("1", "1") + body() +
            "  Behavior = PhysicsBehavior ModuleTag_Phys\n    CenterOfMassOffset = 3\n  End\n");
    return v;
}
static const int kTargetTemplates = 9;  // AB_Big .. AB_Edge

static const char* object_for(int type) {
    static const char* const kAny[] = {"AB_Laser", "AB_Dud",      "AB_Bomb",        "AB_Trap",
                                       "AB_TrapKill", "AB_PhysObj", "AB_Nothing", "AB_TrapKillUser"};
    if (chance(25)) return kAny[pick(8)];
    switch (type) {
        case 1: case 2: case 5: return chance(80) ? "AB_Bomb" : "AB_TrapKill";
        case 11: return choose({"AB_Trap", "AB_Trap", "AB_TrapKill"});
        case 3: return chance(70) ? "AB_PhysObj" : "AB_Dud";
        default: return "AB_Laser";
    }
}

static void field(std::string& s, const char* name, const char* value) {
    if (value) s += std::string("    ") + name + " = " + value + "\n";
}
static void flag(std::string& s, const char* name, uint32_t percent) {
    if (chance(percent)) s += std::string("    ") + name + " = Yes\n";
}

static std::string ability_ini(int user, int k, int power) {
    const int type = power / 2;
    std::string s = "  Behavior = SpecialAbilityUpdate ModuleTag_SAU" + num(k) +
                    "\n    SpecialPowerTemplate = " + power_name(power) + "\n";
    // A quarter of the variants act at once, from anywhere: the effects need no run-up.
    const bool quick = user % 4 == 0;
    field(s, "StartAbilityRange", quick ? nullptr : choose({"0", "2.5", "3", "5", "40", "150", "150", nullptr}));
    field(s, "AbilityAbortRange", choose({nullptr, nullptr, nullptr, "5", "60", "200"}));
    // A capture flashes only over a long preparation.
    if ((type == 6 || type == 9) && chance(60))
        field(s, "PreparationTime", "1000");
    else
        field(s, "PreparationTime", quick ? choose({"0", "0", "33", "100", nullptr})
                                      : choose({"0", "0", "33", "100", "200", "500"}));
    field(s, "PersistentPrepTime", choose({nullptr, nullptr, nullptr, "33", "100"}));
    flag(s, "PersistenceRequiresRecharge", 35);
    field(s, "PackTime", quick ? choose({"0", "33", "100"}) : choose({"0", "33", "100", "200", nullptr}));
    field(s, "UnpackTime", quick ? "0" : choose({"0", "33", "100", "200", nullptr}));
    field(s, "PackUnpackVariationFactor", choose({nullptr, "0", "0.2", "0.5", "1.5"}));
    field(s, "PreTriggerUnstealthTime", choose({nullptr, "0", "33", "100", "300"}));
    flag(s, "SkipPackingWithNoTarget", 30);
    field(s, "SpecialObject", chance(93) ? object_for(type) : nullptr);
    field(s, "MaxSpecialObjects", choose({nullptr, "0", "1", "2", "3"}));
    flag(s, "SpecialObjectsPersistent", 40);
    flag(s, "UniqueSpecialObjectTargets", 20);
    flag(s, "SpecialObjectsPersistWhenOwnerDies", 30);
    flag(s, "AlwaysValidateSpecialObjects", 25);
    flag(s, "FlipOwnerAfterPacking", 20);
    flag(s, "FlipOwnerAfterUnpacking", 20);
    field(s, "FleeRangeAfterCompletion", choose({nullptr, nullptr, "0", "20", "60"}));
    field(s, "DisableFXParticleSystem", choose({"AB_Fx", "AB_Fx", "AB_Fx", "AB_Missing", nullptr}));
    flag(s, "DoCaptureFX", 60);
    field(s, "EffectDuration", choose({nullptr, "33", "500", "3000"}));
    field(s, "EffectValue", choose({nullptr, "7", "2500"}));
    field(s, "PackSound", choose({nullptr, "AS_Pack"}));
    field(s, "UnpackSound", choose({nullptr, "AS_Unpack"}));
    field(s, "PrepSoundLoop", choose({nullptr, "AS_Prep"}));
    field(s, "TriggerSound", choose({nullptr, "AS_Trigger"}));
    // Charges that give the unit away need the most vectors.
    flag(s, "LoseStealthOnTrigger", type == 1 || type == 5 ? 85 : 50);
    field(s, "AwardXPForTriggering", choose({nullptr, "0", "5", "50"}));
    field(s, "SkillPointsForTriggering", choose({nullptr, nullptr, "-1", "0", "3"}));
    if (chance(30)) s += "    ApproachRequiresLOS = No\n";
    if (chance(35)) s += "    NeedToFaceTarget = No\n";
    s += "  End\n";
    return s;
}

// Command buttons: one per power, then the others.
struct ButtonDef {
    std::string name, command;
    int power;  // -1 none
    bool context;
    int slot;
};
static std::vector<ButtonDef> g_buttons;
static void make_buttons() {
    for (int p = 0; p < kPowers; ++p)
        g_buttons.push_back({"AB_BSp" + num(p), "SPECIAL_POWER", p, p == 3, 0});
    g_buttons.push_back({"AB_BSpNone", "SPECIAL_POWER", -1, false, 0});
    g_buttons.push_back({"AB_BFire", "FIRE_WEAPON", -1, false, 1});
    g_buttons.push_back({"AB_BSwitch", "SWITCH_WEAPON", -1, false, 0});
    g_buttons.push_back({"AB_BHijack", "HIJACK_VEHICLE", -1, false, 0});
    g_buttons.push_back({"AB_BCarBomb", "CONVERT_TO_CARBOMB", -1, false, 0});
    g_buttons.push_back({"AB_BSabotage", "SABOTAGE_BUILDING", -1, false, 0});
    g_buttons.push_back({"AB_BStop", "STOP", -1, false, 0});
}
static const int kExtraButtons = 7;

static std::string button_ini() {
    std::string s;
    for (const ButtonDef& b : g_buttons) {
        s += "CommandButton " + b.name + "\n  Command = " + b.command + "\n";
        if (b.power >= 0) s += "  SpecialPower = " + power_name(b.power) + "\n";
        if (b.context) s += "  Options = CONTEXTMODE_COMMAND\n";
        if (b.slot == 1) s += "  WeaponSlot = SECONDARY\n";
        s += "End\n";
    }
    return s;
}

// Command set of a user: slot -> button index.
static std::vector<std::pair<int, int>> g_sets[kUsers];
static std::string set_ini(int user) {
    std::string s = "CommandSet AB_Set" + num(user) + "\n";
    for (auto& e : g_sets[user]) s += "  " + num(e.first + 1) + " = " + g_buttons[e.second].name + "\n";
    s += "End\n";
    return s;
}

static std::string user_object(int i) {
    UserVariant& u = g_users[i];
    u.ai = chance(94);
    u.physics = chance(70);
    u.stealth = chance(i % kTypeCount == 1 || i % kTypeCount == 5 ? 95 : 65);
    u.weapons = chance(80) ? 3 : static_cast<int>(pick(4));
    u.hunt = chance(75);
    const int count = 1 + static_cast<int>(chance(35)) + static_cast<int>(chance(15));
    std::vector<int> types = {i % kTypeCount};
    static const int kExclusive[] = {7, 8, 6, 1, 2, 9, 11};
    while (static_cast<int>(types.size()) < count)
        types.push_back(chance(75) ? kExclusive[pick(7)] : static_cast<int>(pick(kTypeCount)));
    for (int t : types) u.power.push_back(t * 2 + static_cast<int>(chance(45)));

    std::string head = "Object AB_U" + num(i) + "\n  KindOf = PRELOAD INFANTRY SELECTABLE\n" +
                       "  VisionRange = 200\n  ShroudClearingRange = 0\n  CommandSet = AB_Set" + num(i) +
                       "\n  IsTrainable = Yes\n  ExperienceRequired = 0 10 60 200\n"
                       "  ExperienceValue = 1 2 3 4\n  VoiceTaskComplete = AS_VoiceTask\n"
                       "  UnitSpecificSounds\n    VoiceCaptureBuildingComplete = AS_VoiceCapture\n"
                       "    VoiceDisableVehicleComplete = AS_VoiceDisable\n"
                       "    VoiceStealCashComplete = AS_VoiceCash\n  End\n" +
                       cylinder("3", "10");
    std::string mods;
    if (u.ai) mods += "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n";
    if (u.physics) mods += "  Behavior = PhysicsBehavior ModuleTag_Phys\n  End\n";
    if (u.stealth) {
        mods += "  Behavior = StealthUpdate ModuleTag_Stealth\n    InnateStealth = Yes\n"
                "    StealthDelay = " + std::string(choose({"100", "500", "2000"})) + "\n";
        if (std::find(types.begin(), types.end(), 10) != types.end() || chance(10))
            mods += "    DisguisesAsTeam = Yes\n    DisguiseTransitionTime = 100\n"
                    "    DisguiseRevealTransitionTime = 100\n";
        mods += "  End\n";
    }
    for (size_t k = 0; k < u.power.size(); ++k) {
        if (chance(93)) {
            mods += "  Behavior = SpecialAbility ModuleTag_SP" + num(k) + "\n    SpecialPowerTemplate = " +
                    power_name(u.power[k]) + "\n    StartsReady = " + (chance(85) ? "Yes" : "No") + "\n";
            if (chance(75)) mods += "    UpdateModuleStartsAttack = Yes\n";
            mods += "  End\n";
        }
        mods += ability_ini(i, static_cast<int>(k), u.power[k]);
    }
    if (u.hunt) {
        mods += "  Behavior = CommandButtonHuntUpdate ModuleTag_Hunt\n";
        field(mods, "ScanRate", choose({nullptr, "0", "33", "100", "500"}));
        field(mods, "ScanRange", choose({nullptr, "60", "150", "400"}));
        mods += "  End\n";
    }
    mods += "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n";

    // The command set: the unit's own powers and the other commands.
    int slot = 0;
    for (int p : u.power) g_sets[i].push_back({slot++, p});
    g_sets[i].push_back({slot++, kPowers});  // no template
    for (int b = 1; b < kExtraButtons; ++b) g_sets[i].push_back({slot++, kPowers + b});
    if (chance(50)) g_sets[i].push_back({17, static_cast<int>(pick(kPowers))});

    g_rust_users[i] = head + mods + "End\n";
    // Only the reference's ActionManager reads these: the answers are recorded.
    const std::string collides =
        "  Behavior = ConvertToHijackedVehicleCrateCollide ModuleTag_C1\n    RequiredKindOf = VEHICLE\n  End\n"
        "  Behavior = ConvertToCarBombCrateCollide ModuleTag_C2\n    RequiredKindOf = VEHICLE\n  End\n"
        "  Behavior = SabotageSupplyDropzoneCrateCollide ModuleTag_C3\n    BuildingPickup = Yes\n  End\n";
    return head + mods + collides + "End\n";
}

// ---- helpers -----------------------------------------------------------------

template <typename T>
static std::vector<T*> behaviors(Object* o) {
    std::vector<T*> v;
    for (BehaviorModule** m = o->getBehaviorModules(); m && *m; ++m)
        if (T* typed = dynamic_cast<T*>(*m)) v.push_back(typed);
    return v;
}
template <typename T>
static T* behavior(Object* o) {
    auto v = behaviors<T>(o);
    return v.empty() ? nullptr : v[0];
}

static std::string one_line(const std::string& s) {
    std::string out = s;
    for (char& c : out)
        if (c == '\n') c = '|';
    return out;
}

struct Scene {
    oracle_wp51::World* w;
    ObjectID user = INVALID_ID;
    std::vector<ObjectID> fixed;  // targets, riders, mines, traps (in spawn order)
    ObjectID highest_fixed = INVALID_ID;
    ObjectID sink = INVALID_ID;
    UnsignedInt frame = 1;
    Int money_earned0 = 0, skill0 = 0;
    UnsignedInt captured0 = 0;
    size_t particle_seen = 0;
    std::string previous[8];
    uint32_t rotate = 0;

    Object* u() const { return TheGameLogic->findObjectByID(user); }
    Player* player(int n) const { return ThePlayerList->getNthPlayer(n); }
};

static std::string object_state(Object* o) {
    std::string s = num(o->getControllingPlayer()->getPlayerIndex());
    uint64_t status = 0;
    for (int b = 0; b < 45; ++b)
        if (o->getStatusBits().test(b)) status |= 1ull << b;
    s += "," + num(status);
    s += "," + num(o->isEffectivelyDead()) + num(o->isDestroyed()) + num(o->isDisabledByType(DISABLED_HACKED));
    s += "," + num(o->getDisabledUntil(DISABLED_HACKED));
    s += "," + num(o->getContain() ? o->getContain()->getContainCount() : 0);
    return s;
}

static std::string observe(Scene& sc, const std::string& perms, uint32_t ret, int hunt_button_probe) {
    std::string sec[8];
    Object* u = sc.u();
    if (u) {
        for (SpecialAbilityUpdate* a : behaviors<SpecialAbilityUpdate>(u)) {
            const CommandButton* probe =
                TheControlBar->findCommandButton(g_buttons[hunt_button_probe].name.c_str());
            // The reference dereferences a command with no power while packing.
            const bool safe = probe && probe->getSpecialPowerTemplate();
            sec[0] += oracle_wp51::World::snapshot_save_hex(static_cast<UpdateModule*>(a)) + "," +
                      num(a->friend_getNextCallFrame()) + "," + num(a->isPowerCurrentlyInUse()) + "," +
                      num(safe ? a->isPowerCurrentlyInUse(probe) : 2) + "," + num(a->getSpecialObjectCount()) +
                      "," + num(a->getSpecialObjectMax()) + ",";
            std::string found;
            for (ObjectID id : sc.fixed) {
                Object* t = TheGameLogic->findObjectByID(id);
                Object* s = t ? a->findSpecialObjectWithProducerID(t) : nullptr;
                if (s) found += num(id) + ">" + num(s->getID()) + " ";
            }
            sec[0] += found + ";";
        }
        for (SpecialPowerModule* p : behaviors<SpecialPowerModule>(u))
            sec[0] += oracle_wp51::World::snapshot_save_hex(static_cast<BehaviorModule*>(p)) + "+";
        for (CommandButtonHuntUpdate* h : behaviors<CommandButtonHuntUpdate>(u))
        {
            const CommandButton* held = h->*access_member(AbButtonTag());
            sec[1] = oracle_wp51::World::snapshot_save_hex(static_cast<UpdateModule*>(h)) + "," +
                     num(h->friend_getNextCallFrame()) + "," + (held ? held->getName().str() : "-");
        }
        std::string& s = sec[2];
        s = pos_bits(*u->getPosition()) + "," + bits(u->getOrientation()) + ",";
        const ModelConditionFlags& f = u->getDrawable()->getModelConditionFlags();
        s += num(f.test(MODELCONDITION_UNPACKING)) + num(f.test(MODELCONDITION_PACKING)) +
             num(f.test(MODELCONDITION_FIRING_A)) + num(f.test(MODELCONDITION_RAISING_FLAG));
        s += "," + object_state(u);
        WeaponSlotType slot = PRIMARY_WEAPON;
        const Weapon* weapon = u->getCurrentWeapon(&slot);
        s += "," + num(weapon ? slot : 9) + num(u->isCurWeaponLocked());
        s += "," + num(u->getExperienceTracker()->getCurrentExperience()) + "," +
             num(u->getExperienceTracker()->getVeterancyLevel());
        PhysicsBehavior* phys = u->getPhysics();
        s += "," + (phys ? oracle_wp51::World::snapshot_save_hex(static_cast<UpdateModule*>(phys)) : "-");
        StealthUpdate* stealth = u->getStealth();
        s += "," + (stealth ? oracle_wp51::World::snapshot_save_hex(static_cast<UpdateModule*>(stealth)) : "-");
        AIUpdateInterface* ai = u->getAIUpdateInterface();
        s += "," + num(ai ? ai->*access_member(AbIgnoreTag()) : 0);
        Object* sink = TheGameLogic->findObjectByID(sc.sink);
        s += "," + num(sink ? sink->getExperienceTracker()->getCurrentExperience() : 0);
    }
    for (ObjectID id : sc.fixed) {
        Object* t = TheGameLogic->findObjectByID(id);
        sec[3] += (t ? object_state(t) : std::string("-")) + ";";
    }
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        const ObjectID id = o->getID();
        if (id <= sc.highest_fixed) continue;
        PhysicsBehavior* phys = o->getPhysics();
        sec[4] += num(id) + "," + o->getTemplate()->getName().str() + "," + num(o->getProducerID()) + "," +
                  num(o->getExperienceTracker()->*access_member(AbSinkTag())) + "," +
                  pos_bits(*o->getPosition()) + "," + bits(o->getOrientation()) + "," +
                  num(o->isDestroyed()) + "," +
                  (phys ? bits(phys->*access_member(AbPitchTag())) + num((phys->*access_member(AbPhysFlagsTag()) & 4) != 0)
                        : std::string("-")) +
                  ";";
    }
    {
        // Newest-first in the engine's list: sort by id.
        std::vector<std::string> parts;
        std::string all = sec[4];
        size_t start = 0, end;
        while ((end = all.find(';', start)) != std::string::npos) {
            parts.push_back(all.substr(start, end - start + 1));
            start = end + 1;
        }
        std::sort(parts.begin(), parts.end(),
                  [](const std::string& a, const std::string& b) { return std::stoul(a) < std::stoul(b); });
        sec[4].clear();
        for (auto& p : parts) sec[4] += p;
    }
    sec[5] = num(sc.player(1)->getMoney()->countMoney()) + "," + num(sc.player(2)->getMoney()->countMoney()) +
             "," + num(sc.player(1)->getScoreKeeper()->getTotalMoneyEarned() - sc.money_earned0) + "," +
             num(sc.player(1)->getSkillPoints() - sc.skill0) + "," +
             num(sc.player(1)->getAcademyStats()->*access_member(AbCapturedTag()) - sc.captured0);

    std::string events;
    for (const std::string& e : g_ai_log) events += e + " ";
    events += "/";
    for (const std::string& e : g_audio_log) events += e + " ";
    events += "/";
    for (const std::string& e : g_text_log) events += e + " ";
    g_text_log.clear();
    events += "/";
    Bool* eva = &(TheEva->*access_member(AbEvaTag()))[0];
    for (int e : {EVA_BuildingBeingStolen, EVA_BuildingStolen, EVA_CashStolen}) {
        events += num(eva[e] ? 1 : 0);
        eva[e] = FALSE;
    }
    events += "/";
    {
        auto& list = TheParticleSystemManager->getAllParticleSystems();
        size_t i = 0;
        for (ParticleSystem* sys : list) {
            if (i++ < sc.particle_seen) continue;
            Coord3D p;
            sys->getPosition(&p);
            events += pos_bits(p) + ":" + num(sys->*access_member(AbLifetimeTag())) + " ";
        }
        sc.particle_seen = i;
    }
    g_ai_log.clear();
    g_audio_log.clear();

    std::string row = perms + "|";
    for (int k = 0; k < 6; ++k) {
        row += (sec[k] == sc.previous[k] ? std::string("=") : sec[k]) + "|";
        sc.previous[k] = sec[k];
    }
    row += events + "|" + num(GetGameLogicRandomSeedCRC()) + "|" + num(ret);
    return row;
}

// What the ActionManager would answer the hunter for `button`, one id per yes.
static std::string permissions(Scene& sc, const CommandButton* button) {
    std::string s;
    Object* u = sc.u();
    if (!u || !button) return s;
    const SpecialPowerTemplate* sp = button->getSpecialPowerTemplate();
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        if (o == u) continue;
        const ObjectID id = o->getID();
        if (sp && TheActionManager->canDoSpecialPowerAtObject(u, o, CMD_FROM_AI, sp, 0))
            s += "p" + num(id) + " ";
        Bool yes = FALSE;
        switch (button->getCommandType()) {
            case GUICOMMANDMODE_HIJACK_VEHICLE: yes = TheActionManager->canHijackVehicle(u, o, CMD_FROM_AI); break;
            case GUICOMMANDMODE_SABOTAGE_BUILDING: yes = TheActionManager->canSabotageBuilding(u, o, CMD_FROM_AI); break;
            case GUICOMMANDMODE_CONVERT_TO_CARBOMB:
                yes = TheActionManager->canConvertObjectToCarBomb(u, o, CMD_FROM_AI);
                break;
            default: break;
        }
        if (yes) s += "e" + num(id) + " ";
    }
    return s;
}

static void step_module(UpdateModule* u, UnsignedInt frame) {
    if (u->friend_getNextCallFrame() > frame) return;
    UpdateSleepTime sleep_len = u->update();
    if (sleep_len < 1) sleep_len = UPDATE_SLEEP_NONE;
    UnsignedInt next = frame + static_cast<UnsignedInt>(sleep_len);
    if (next > UPDATE_SLEEP_FOREVER) next = UPDATE_SLEEP_FOREVER;
    u->friend_setNextCallFrame(next);
}

static std::string run_frame(Scene& sc) {
    sc.w->set_frame(++sc.frame);
    std::string perms;
    if (Object* u = sc.u()) {
        for (CommandButtonHuntUpdate* h : behaviors<CommandButtonHuntUpdate>(u)) {
            if (h->friend_getNextCallFrame() <= sc.frame) perms = permissions(sc, h->*access_member(AbButtonTag()));
            step_module(h, sc.frame);
        }
        if (!u->isDestroyed())
            for (SpecialAbilityUpdate* a : behaviors<SpecialAbilityUpdate>(u)) step_module(a, sc.frame);
    }
    ThePartitionManager->update();
    TheGameLogic->processDestroyList();
    return perms;
}

static const float kOffsets[][2] = {{0, 0},    {4, 0},    {6, 1},    {10, 0},  {0, 12},   {-18, 0},
                                    {25, 20},  {-30, -9}, {45, 0},   {0, -70}, {-100, 5}, {130, 40},
                                    {2, 2},    {-7, 0},   {200, 10}, {37, 0}};
static const int kOffsetCount = 16;

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_pkt368_314_ability <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheAudio = new RecordingAudio;
    TheAudio->init();
    TheInGameUI = new RecordingUI;
    TheAudio->friend_getMiscAudio()->m_defectorTimerTickSound.setEventName("AS_Tick");
    TheTerrainLogic = new WallTerrain;
    TheTerrainLogic->init();
    TheActionManager = NEW ActionManager;
    TheParticleSystemManager->newTemplate("AB_Fx");
    (TheModuleFactory->*&FactoryAccess::addModuleInternal)(
        new_oracle_ai, AIUpdateInterface::friend_newModuleData, MODULETYPE_BEHAVIOR, "AIUpdateInterface",
        AIUpdateInterface::getInterfaceMask());
    (TheModuleFactory->*&FactoryAccess::addModuleInternal)(
        new_oracle_die, DestroyDie::friend_newModuleData, MODULETYPE_BEHAVIOR, "DestroyDie",
        DestroyDie::getInterfaceMask());

#define ADD_IDLE(CLASS, BASE)                                                          \
    (TheModuleFactory->*&FactoryAccess::addModuleInternal)(                            \
        new_##CLASS, BASE::friend_newModuleData, MODULETYPE_BEHAVIOR, #BASE,           \
        BASE::getInterfaceMask());
    ADD_IDLE(IdleHijack, ConvertToHijackedVehicleCrateCollide)
    ADD_IDLE(IdleCarBomb, ConvertToCarBombCrateCollide)
    ADD_IDLE(IdleSabotage, SabotageSupplyDropzoneCrateCollide)

    make_buttons();
    const std::string sciences =
        "Science SCIENCE_AB_HAVE\n  PrerequisiteSciences = None\n  SciencePurchasePointCost = 1\n"
        "  IsGrantable = Yes\nEnd\n"
        "Science SCIENCE_AB_LACK\n  PrerequisiteSciences = None\n  SciencePurchasePointCost = 1\n"
        "  IsGrantable = Yes\nEnd\n";
    std::string ini = sciences + special_powers_ini();
    ini += "Rank 1\n  SkillPointsNeeded = 0\nEnd\nRank 2\n  SkillPointsNeeded = 100000000\nEnd\n";
    const std::vector<Fixed> fixtures = fixed_objects();
    std::string shared;
    for (const Fixed& f : fixtures) shared += f.ini;
    ini += "Weapon AB_Gun\n  PrimaryDamage = 1.0\n  AttackRange = 100.0\n  DamageType = EXPLOSION\n"
           "  DelayBetweenShots = 100\nEnd\n";
    std::string users;
    for (int i = 0; i < kUsers; ++i) {
        std::string u = user_object(i);
        std::string weapons;
        if (g_users[i].weapons) {
            weapons = "  WeaponSet\n    Conditions = None\n";
            if (g_users[i].weapons & 1) weapons += "    Weapon = PRIMARY AB_Gun\n";
            if (g_users[i].weapons & 2) weapons += "    Weapon = SECONDARY AB_Gun\n";
            weapons += "  End\n";
        }
        u.insert(u.rfind("End\n"), weapons);
        users += u;
    }
    auto parse = [&](const std::string& text) {
        w.mem_ini.parse_blocks(text, INI_LOAD_OVERWRITE, [](INI* in, const char* tok) {
            if (std::strcmp(tok, "SpecialPower") == 0) SpecialPowerStore::parseSpecialPowerDefinition(in);
            else if (std::strcmp(tok, "Science") == 0) INI::parseScienceDefinition(in);
            else if (std::strcmp(tok, "Weapon") == 0) WeaponStore::parseWeaponTemplateDefinition(in);
            else if (std::strcmp(tok, "Rank") == 0) INI::parseRankDefinition(in);
            else if (std::strcmp(tok, "CommandButton") == 0) INI::parseCommandButtonDefinition(in);
            else if (std::strcmp(tok, "CommandSet") == 0) INI::parseCommandSetDefinition(in);
            else if (std::strcmp(tok, "Object") == 0) {
                AsciiString name = in->getNextToken();
                ThingFactory::parseObjectDefinition(in, name, AsciiString::TheEmptyString);
            }
        });
    };
    parse(ini);
    parse(button_ini());
    std::string sets;
    for (int i = 0; i < kUsers; ++i) sets += set_ini(i);
    parse(sets);
    // Only the car bomb test of the reference's ActionManager reads the weapon set.
    std::string for_reference = shared;
    const std::string vehicle = "Object AB_Vehicle\n";
    for_reference.insert(for_reference.find(vehicle) + vehicle.size(),
                         "  WeaponSet\n    Conditions = CARBOMB\n    Weapon = PRIMARY AB_Gun\n  End\n");
    for (const Fixed& f : fixtures) {
        const std::string head = std::string("Object ") + f.name + "\n";
        for_reference.insert(for_reference.find(head) + head.size(), "  VoiceDefect = AS_Defect\n");
    }
    parse(for_reference + users);
    w.init_with_neutral_player();
    // Player B is local, so its objects are "locally viewed". Nothing clears the shroud
    // (ShroudClearingRange 0): a change of the local player's shroud needs a display.
    ThePlayerList->setLocalPlayer(ThePlayerList->getNthPlayer(2));
    TheEva->reset();
    if (!ThePlayerList->getNthPlayer(1)->grantScience(TheScienceStore->getScienceFromInternalName("SCIENCE_AB_HAVE")))
        return 6;

    FILE* out = std::fopen(argv[1], "w");
    if (!out) return 1;
    const int kTotal = 1300;
    std::fprintf(out, "{\"vector_count\":%d,\n\"shared\":\"%s\",\n", kTotal, one_line(shared).c_str());
    std::fprintf(out, "\"powers\":[");
    for (int p = 0; p < kPowers; ++p) {
        const SpecialPowerTemplate* t = TheSpecialPowerStore->findSpecialPowerTemplate(power_name(p).c_str());
        if (!t) return 3;
        std::fprintf(out, "%s{\"type\":\"%s\",\"reload\":%u,\"science\":%d}", p ? "," : "", kTypes[p / 2],
                     t->getReloadTime(), science_of(p));
    }
    std::fprintf(out, "],\n\"buttons\":[");
    for (size_t b = 0; b < g_buttons.size(); ++b)
        std::fprintf(out, "%s{\"name\":\"%s\",\"command\":\"%s\",\"power\":%d,\"context\":%d,\"slot\":%d}",
                     b ? "," : "", g_buttons[b].name.c_str(), g_buttons[b].command.c_str(), g_buttons[b].power,
                     g_buttons[b].context ? 1 : 0, g_buttons[b].slot);
    std::fprintf(out, "],\n\"users\":[");
    for (int i = 0; i < kUsers; ++i) {
        std::fprintf(out, "%s\n{\"ini\":\"%s\",\"weapons\":%d,\"set\":[", i ? "," : "",
                     one_line(g_rust_users[i]).c_str(), g_users[i].weapons);
        for (size_t k = 0; k < g_sets[i].size(); ++k)
            std::fprintf(out, "%s[%d,%d]", k ? "," : "", g_sets[i][k].first, g_sets[i][k].second);
        std::fprintf(out, "]}");
    }
    std::fprintf(out, "],\n\"vectors\":[");

    static AttackPriorityInfo priority;
    for (int i = 0; i < kTotal; ++i) {
        // Clear the world and restart the object ids.
        std::vector<Object*> old;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) old.push_back(o);
        for (Object* o : old) TheGameLogic->destroyObject(o);
        w.flush_destroy_list();
        ThePartitionManager->update();
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        TheParticleSystemManager->reset();
        TheParticleSystemManager->newTemplate("AB_Fx");

        Scene sc;
        sc.w = &w;
        sc.particle_seen = TheParticleSystemManager->getAllParticleSystems().size();
        sc.frame = 100 + pick(5000);
        w.set_frame(sc.frame);
        const int user = i < kUsers * 4 ? i % kUsers : static_cast<int>(pick(kUsers));
        const UserVariant& uv = g_users[user];
        const uint32_t seed = 1 + pick(1000000);
        const uint32_t rel = chance(78) ? 0 : (chance(50) ? 1 : 2);  // enemies, neutral, allies
        static const Relationship kRel[] = {ENEMIES, NEUTRAL, ALLIES};
        sc.player(1)->setPlayerRelationship(sc.player(2), kRel[rel]);
        sc.player(2)->setPlayerRelationship(sc.player(1), kRel[rel]);
        static const UnsignedInt kMoney[] = {0, 1, 999, 1000, 5000};
        const uint32_t money = kMoney[pick(5)];
        for (int p = 1; p <= 2; ++p) {
            Money* m = sc.player(p)->getMoney();
            m->withdraw(m->countMoney(), FALSE);
        }
        sc.player(2)->getMoney()->deposit(money, FALSE, FALSE);
        sc.player(1)->getMoney()->deposit(100, FALSE, FALSE);
        sc.money_earned0 = sc.player(1)->getScoreKeeper()->getTotalMoneyEarned();
        sc.skill0 = sc.player(1)->getSkillPoints();
        sc.captured0 = sc.player(1)->getAcademyStats()->*access_member(AbCapturedTag());
        g_wall = chance(70) ? 0.0f : (chance(50) ? 1000.0f : 22.0f);
        const Real modifier = static_cast<Real>(chance(40) ? 0 : (chance(50) ? 10 : 100));
        const_cast<TAiData*>(TheAI->getAiData())->*access_member(AbModifierTag()) = modifier;
        InitRandom(seed);

        std::string setup;
        Team* teams[3] = {w.team_neutral, w.team_a, w.team_b};
        auto spawn = [&](const std::string& name, int team, Coord3D pos) -> Object* {
            Object* o = w.spawn_object(name.c_str(), teams[team], pos);
            if (!o) {
                std::fprintf(stderr, "spawn failed %s\n", name.c_str());
                std::exit(4);
            }
            return o;
        };
        const Coord3D home = {static_cast<Real>(470 + static_cast<int>(pick(61))),
                              static_cast<Real>(470 + static_cast<int>(pick(61))), 0.0f};
        g_user_id = static_cast<ObjectID>(1);
        Object* u = spawn("AB_U" + num(user), 1, home);
        sc.user = u->getID();
        setup += "[\"AB_U" + num(user) + "\",1," + num(static_cast<int>(home.x)) + "," +
                 num(static_cast<int>(home.y)) + "]";
        // Targets around the hunter. The first is of a kind the first ability likes.
        const int type0 = uv.power[0] / 2;
        auto liked = [&](int type) -> int {
            if (chance(25)) return static_cast<int>(pick(kTargetTemplates));
            switch (type) {
                case 7: case 10: return chance(75) ? 3 : 6;
                case 0: return chance(75) ? 5 : 3;
                case 1: case 2: case 5: return static_cast<int>(pick(5));
                default: {
                    static const int kStructures[] = {0, 1, 2, 4, 7, 8};
                    return kStructures[pick(6)];
                }
            }
        };
        const int targets = 2 + static_cast<int>(pick(4));
        std::vector<int> garrisons;
        for (int t = 0; t < targets; ++t) {
            const int kind = uv.hunt && chance(12) ? 6 : liked(uv.power[t % uv.power.size()] / 2);
            const int team = chance(75) ? 2 : (chance(60) ? 1 : 0);
            const int off = t == 0 ? static_cast<int>(pick(9)) : static_cast<int>(pick(kOffsetCount));
            Coord3D p = {home.x + kOffsets[off][0] + static_cast<Real>(t), home.y + kOffsets[off][1], 0.0f};
            Object* o = spawn(fixtures[kind].name, team, p);
            sc.fixed.push_back(o->getID());
            setup += std::string(",[\"") + fixtures[kind].name + "\"," + num(team) + "," +
                     num(static_cast<int>(p.x)) + "," + num(static_cast<int>(p.y)) + "]";
            if (kind == 4 || kind == 7) garrisons.push_back(t);
        }
        // Mines of the hunter's side or the other.
        const int mines = chance(35) ? 1 + static_cast<int>(pick(3)) : 0;
        for (int m = 0; m < mines; ++m) {
            const int team = chance(75) ? 1 : 2;
            const int dx = static_cast<int>(pick(121)) - 60, dy = static_cast<int>(pick(41)) - 20;
            Coord3D p = {home.x + dx, home.y + dy, 0.0f};
            sc.fixed.push_back(spawn("AB_Mine", team, p)->getID());
            setup += ",[\"AB_Mine\"," + num(team) + "," + num(static_cast<int>(p.x)) + "," +
                     num(static_cast<int>(p.y)) + "]";
        }
        // Riders in the garrisons.
        std::string riders;
        for (int g : garrisons) {
            if (!chance(70)) continue;
            Object* house = TheGameLogic->findObjectByID(sc.fixed[g]);
            const int count = 1 + static_cast<int>(pick(2));
            const int team = chance(80) ? 2 : 1;
            for (int r = 0; r < count; ++r) {
                const char* name = chance(70) ? "AB_Rider" : "AB_Infantry";
                Object* rider = spawn(name, team, *house->getPosition());
                setup += std::string(",[\"") + name + "\"," + num(team) + "," +
                         num(static_cast<int>(house->getPosition()->x)) + "," +
                         num(static_cast<int>(house->getPosition()->y)) + "]";
                riders += (riders.empty() ? "" : ",") + std::string("[") + num(g) + "," +
                          num(sc.fixed.size()) + "]";
                sc.fixed.push_back(rider->getID());
                house->getContain()->addToContain(rider);
            }
        }
        // Booby traps already on targets.
        std::string traps;
        const bool capturer = uv.power[0] / 2 == 9;
        const int trap_count = chance(capturer ? 70 : 45) ? 1 : 0;
        for (int n = 0; n < trap_count; ++n) {
            const int t = chance(70) ? 0 : static_cast<int>(pick(targets));
            const char* name = capturer && chance(60) ? "AB_TrapKill"
                                                      : choose({"AB_Trap", "AB_Trap", "AB_TrapKill", "AB_TrapKillUser"});
            const int team = chance(60) ? 2 : 1;
            Object* target = TheGameLogic->findObjectByID(sc.fixed[t]);
            Object* trap = spawn(name, team, *target->getPosition());
            behavior<StickyBombUpdate>(trap)->initStickyBomb(target, nullptr);
            setup += std::string(",[\"") + name + "\"," + num(team) + "," +
                     num(static_cast<int>(target->getPosition()->x)) + "," +
                     num(static_cast<int>(target->getPosition()->y)) + "]";
            traps += "[" + num(t) + "," + num(sc.fixed.size()) + "]";
            sc.fixed.push_back(trap->getID());
        }
        sc.highest_fixed = sc.fixed.empty() ? sc.user : sc.fixed.back();
        const int sink = chance(25) ? static_cast<int>(pick(targets)) : -1;
        const int scalar = chance(30) ? 2 : 1;
        if (sink >= 0) u->getExperienceTracker()->setExperienceSink(sc.fixed[sink]);
        u->getExperienceTracker()->setExperienceScalar(static_cast<Real>(scalar));
        sc.sink = sink >= 0 ? sc.fixed[sink] : INVALID_ID;
        ThePartitionManager->update();
        g_ai_log.clear();
        g_audio_log.clear();
        InitRandom(seed + 1);

        auto abilities = behaviors<SpecialAbilityUpdate>(u);
        if (abilities.size() != uv.power.size()) return 5;
        const bool has_hunt = behavior<CommandButtonHuntUpdate>(u) != nullptr;
        auto probe_of = [&]() { return static_cast<int>(sc.rotate % g_buttons.size()); };

        std::fprintf(out,
                     "%s\n{\"user\":%d,\"frame\":%u,\"seed\":%u,\"rel\":%u,\"money\":%u,\"wall\":%d,"
                     "\"modifier\":%d,\"setup\":[%s],\"riders\":[%s],\"traps\":[%s],"
                     "\"sink\":%d,\"scalar\":%d,",
                     i ? "," : "", user, sc.frame, seed, rel, money, static_cast<int>(g_wall),
                     static_cast<int>(modifier), setup.c_str(), riders.c_str(), traps.c_str(), sink, scalar);

        std::vector<std::string> rows;
        std::string ops;
        rows.push_back(observe(sc, "", 0, probe_of()));
        auto op = [&](uint32_t code, int a, int b, int c) {
            ops += (ops.empty() ? "" : ",") + std::string("[") + num(code) + "," + std::to_string(a) + "," +
                   std::to_string(b) + "," + std::to_string(c) + "]";
        };
        auto record = [&](const std::string& perms, uint32_t ret) {
            ++sc.rotate;
            rows.push_back(observe(sc, perms, ret, probe_of()));
        };

        bool over = false;
        const int hunt_vector = has_hunt && chance(40);
        const int length = 5 + static_cast<int>(pick(9));
        if (hunt_vector) {
            if (chance(20)) {
                // The hunter itself off the map.
                const int y = static_cast<int>(u->getPosition()->y);
                op(21, -1, y, 0);
                Coord3D p = {-3.0f, static_cast<Real>(y), 0.0f};
                u->setPosition(&p);
                ThePartitionManager->update();
                record("", 0);
            }
            for (size_t n = 0; n < sc.fixed.size(); ++n) {
                Object* o = TheGameLogic->findObjectByID(sc.fixed[n]);
                if (!o) continue;
                auto mark = [&](ObjectStatusTypes status) {
                    op(7, static_cast<int>(n), status, 1);
                    o->setStatus(MAKE_OBJECT_STATUS_MASK(status), TRUE);
                    record("", 0);
                };
                if (o->getTemplate()->getName() == "AB_Truck") {
                    // Disguised, merely hidden, or flagged as disguised without a disguise.
                    const uint32_t how = pick(3);
                    if (how == 0) {
                        op(19, static_cast<int>(n), -1, 0);
                        o->getStealth()->disguiseAsObject(u);
                        o->setStatus(MAKE_OBJECT_STATUS_MASK2(OBJECT_STATUS_STEALTHED, OBJECT_STATUS_DISGUISED), TRUE);
                        record("", 0);
                    } else {
                        mark(OBJECT_STATUS_STEALTHED);
                        if (how == 2) mark(OBJECT_STATUS_DISGUISED);
                    }
                } else if (o->getTemplate()->getName() == "AB_Rider" && chance(30)) {
                    mark(OBJECT_STATUS_DETECTED);
                } else if (chance(25)) {
                    mark(OBJECT_STATUS_STEALTHED);
                }
                // Off the map: another map status than the hunter's.
                if (chance(8)) {
                    op(21, static_cast<int>(n), static_cast<int>(o->getPosition()->y), 0);
                    Coord3D p = {-3.0f, static_cast<Real>(static_cast<int>(o->getPosition()->y)), 0.0f};
                    o->setPosition(&p);
                    ThePartitionManager->update();
                    record("", 0);
                }
            }
        }
        for (int step = 0; step < length && !over && sc.u(); ++step) {
            u = sc.u();
            uint32_t roll = pick(100);
            uint32_t code;
            if (step == 0) code = hunt_vector ? 13 : (chance(75) ? 0 : 3);
            else if (roll < 34) code = 4;
            else if (roll < 42) code = 0;
            else if (roll < 45) code = 1;
            else if (roll < 48) code = 2;
            else if (roll < 52) code = 3;
            else if (roll < 63) code = 5;
            else if (roll < 70) code = 6;
            else if (roll < 76) code = 7;
            else if (roll < 79) code = 8;
            else if (roll < 81) code = 9;
            else if (roll < 84) code = 10;
            else if (roll < 85) code = 11;
            else if (roll < 87) code = 12;
            else if (roll < 91) code = has_hunt ? 13 : 4;
            else if (roll < 93) code = has_hunt ? 15 : 4;
            else if (roll < 94) code = 16;
            else if (roll < 95) code = has_hunt ? 17 : 4;
            else if (roll < 97) code = 18;
            else if (roll < 99) code = 19;
            else code = has_hunt && chance(50) ? 20 : 14;
            if (step == 1) code = 4;

            const int k = static_cast<int>(pick(static_cast<uint32_t>(uv.power.size())));
            const SpecialPowerTemplate* tmpl =
                TheSpecialPowerStore->findSpecialPowerTemplate(power_name(uv.power[k]).c_str());
            const int t = chance(65) ? (k < targets ? k : 0) : static_cast<int>(pick(sc.fixed.size()));
            Object* target = TheGameLogic->findObjectByID(sc.fixed[t]);
            const int off = static_cast<int>(pick(kOffsetCount));
            switch (code) {
                case 0:
                case 1:
                case 2: {
                    if (code == 0 && !target) continue;
                    SpecialPowerModuleInterface* spm = u->getSpecialPowerModule(tmpl);
                    if (!spm) {
                        code = 3;
                    } else {
                        Coord3D p = {u->getPosition()->x + kOffsets[off][0], u->getPosition()->y + kOffsets[off][1], 0.0f};
                        if (chance(8)) p.x = p.y = 0.0f;
                        const int px = static_cast<int>(p.x), py = static_cast<int>(p.y);
                        p.x = static_cast<Real>(px);
                        p.y = static_cast<Real>(py);
                        op(code, k, code == 0 ? t : px, py);
                        if (code == 0) spm->doSpecialPowerAtObject(target, 0);
                        else if (code == 1) spm->doSpecialPowerAtLocation(&p, 0.0f, 0);
                        else spm->doSpecialPower(0);
                        record("", 0);
                        break;
                    }
                }
                    [[fallthrough]];
                case 3: {
                    // Straight at the update module; sometimes with another power's template.
                    const int power = chance(85) ? uv.power[k] : static_cast<int>(pick(kPowers));
                    const SpecialPowerTemplate* given =
                        TheSpecialPowerStore->findSpecialPowerTemplate(power_name(power).c_str());
                    const int mode = chance(70) && target ? 0 : static_cast<int>(1 + pick(2));
                    Coord3D p = {u->getPosition()->x + kOffsets[off][0], u->getPosition()->y + kOffsets[off][1], 0.0f};
                    p.x = static_cast<Real>(static_cast<int>(p.x));
                    p.y = static_cast<Real>(static_cast<int>(p.y));
                    if (chance(8)) p.x = p.y = 0.0f;
                    ops += (ops.empty() ? "" : ",") + std::string("[3,") + std::to_string(k) + "," +
                           std::to_string(power) + "," + std::to_string(mode) + "," +
                           std::to_string(mode == 0 ? t : static_cast<int>(p.x)) + "," +
                           std::to_string(static_cast<int>(p.y)) + "]";
                    const Bool ret = abilities[k]->initiateIntentToDoSpecialPower(
                        given, mode == 0 ? target : nullptr, mode == 1 ? &p : nullptr, nullptr, 0);
                    record("", ret);
                    break;
                }
                case 4: {
                    const int n = 1 + static_cast<int>(pick(chance(70) ? 6 : 16));
                    op(4, n, 0, 0);
                    for (int f = 0; f < n; ++f) {
                        // A moving unit comes to rest now and then.
                        const std::string perms = run_frame(sc);
                        record(perms, 0);
                    }
                    break;
                }
                case 5: {
                    OracleAI* ai = dynamic_cast<OracleAI*>(u->getAIUpdateInterface());
                    if (!ai) continue;
                    const int idle = chance(70), moving = chance(40);
                    op(5, idle, moving, 0);
                    ai->m_scriptedIdle = idle;
                    ai->*access_member(AbMovingTag()) = moving;
                    record("", 0);
                    break;
                }
                case 6: {
                    // Move the hunter (or a target) next to the other.
                    if (!target) continue;
                    const int who = chance(80) ? 0 : 1;
                    const int near = chance(70) ? static_cast<int>(pick(6)) : off;
                    const int z = chance(85) ? 0 : (chance(50) ? 8 : 40);
                    op(6, t * 2 + who, near, z);
                    Object* mover = who ? target : u;
                    const Object* anchor = who ? u : target;
                    Coord3D p = {anchor->getPosition()->x + kOffsets[near][0],
                                 anchor->getPosition()->y + kOffsets[near][1], static_cast<Real>(z)};
                    mover->setPosition(&p);
                    ThePartitionManager->update();
                    record("", 0);
                    break;
                }
                case 7: {
                    if (!target) continue;
                    const int status = chance(60) ? OBJECT_STATUS_STEALTHED : OBJECT_STATUS_DETECTED;
                    const int on = chance(65);
                    op(7, t, status, on);
                    target->setStatus(MAKE_OBJECT_STATUS_MASK(static_cast<ObjectStatusTypes>(status)), on);
                    record("", 0);
                    break;
                }
                case 8: {
                    // Kill a target, destroy a target, or destroy a special object.
                    const int mode = static_cast<int>(pick(3));
                    if (mode < 2) {
                        if (!target) continue;
                        op(8, mode, t, 0);
                        if (mode == 0) target->setEffectivelyDead(TRUE);
                        else TheGameLogic->destroyObject(target);
                    } else {
                        Object* special = nullptr;
                        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject())
                            if (o->getID() > sc.highest_fixed && (!special || o->getID() < special->getID()))
                                special = o;
                        if (!special) continue;
                        op(8, 2, special->getID(), 0);
                        TheGameLogic->destroyObject(special);
                    }
                    TheGameLogic->processDestroyList();
                    record("", 0);
                    break;
                }
                case 9: {
                    AIUpdateInterface* ai = u->getAIUpdateInterface();
                    if (!ai) continue;
                    const int src = chance(60) ? CMD_FROM_PLAYER : (chance(50) ? CMD_FROM_SCRIPT : CMD_FROM_AI);
                    op(9, src, 0, 0);
                    ai->friend_setLastCommandSource(static_cast<CommandSourceType>(src));
                    record("", 0);
                    break;
                }
                case 10: {
                    if (!target) continue;
                    const int team = static_cast<int>(pick(3));
                    op(10, t, team, 0);
                    target->setTeam(teams[team]);
                    record("", 0);
                    break;
                }
                case 11: {
                    const int dead = chance(75);
                    op(11, dead, 0, 0);
                    u->setEffectivelyDead(dead);
                    record("", 0);
                    break;
                }
                case 12: {
                    op(12, 0, 0, 0);
                    TheGameLogic->destroyObject(u);
                    TheGameLogic->processDestroyList();
                    record("", 0);
                    over = true;
                    break;
                }
                case 13: {
                    CommandButtonHuntUpdate* hunt = behavior<CommandButtonHuntUpdate>(u);
                    if (!hunt) continue;
                    int b;
                    if (chance(8)) b = -1;
                    else if (chance(55)) {
                        b = g_sets[user][pick(static_cast<uint32_t>(uv.power.size()))].second;
                    } else b = g_sets[user][pick(static_cast<uint32_t>(g_sets[user].size()))].second;
                    // A button of another set is not found.
                    if (chance(5)) b = static_cast<int>(pick(static_cast<uint32_t>(g_buttons.size())));
                    op(13, b, 0, 0);
                    const AsciiString name = b < 0 ? "AB_Unknown" : g_buttons[b].name.c_str();
                    // `setCommandButton` idles the unit before its first scan.
                    const std::string perms = permissions(sc, TheControlBar->findCommandButton(name));
                    hunt->setCommandButton(name);
                    record(perms, 0);
                    break;
                }
                case 20: {
                    CommandButtonHuntUpdate* hunt = behavior<CommandButtonHuntUpdate>(u);
                    if (!hunt) continue;
                    op(20, 0, 0, 0);
                    oracle::MemoryXfer xs(XFER_SAVE);
                    xs.open("save");
                    oracle::call_snapshot_xfer(static_cast<UpdateModule*>(hunt), &xs);
                    xs.close();
                    // Loading re-registers the update; put the scheduler index back.
                    const Int index = static_cast<UpdateModule*>(hunt)->*access_member(AbIndexTag());
                    oracle_wp51::World::snapshot_load_bytes(static_cast<UpdateModule*>(hunt), xs.bytes);
                    static_cast<UpdateModule*>(hunt)->*access_member(AbIndexTag()) = index;
                    record("", 0);
                    break;
                }
                case 14: {
                    op(14, k, 0, 0);
                    const uint32_t ret = static_cast<uint32_t>(static_cast<UpdateModule*>(abilities[k])->update());
                    record("", ret);
                    break;
                }
                case 15: {
                    // Command bar override, or another command set for the unit.
                    const int mode = static_cast<int>(pick(2));
                    if (mode == 0) {
                        const int slot = static_cast<int>(pick(6));
                        const int b = chance(30) ? -1 : static_cast<int>(pick(static_cast<uint32_t>(g_buttons.size())));
                        op(15, 0, slot, b);
                        TheGameLogic->setControlBarOverride(
                            ("AB_Set" + num(user)).c_str(), slot,
                            b < 0 ? nullptr : TheControlBar->findCommandButton(g_buttons[b].name.c_str()));
                    } else {
                        const int other = chance(25) ? -1 : static_cast<int>(pick(kUsers));
                        op(15, 1, other, 0);
                        u->setCommandSetStringOverride(other < 0 ? AsciiString("AB_NoSet")
                                                                 : AsciiString(("AB_Set" + num(other)).c_str()));
                    }
                    record("", 0);
                    break;
                }
                case 16: {
                    const int frames = chance(50) ? 3 : 1000;
                    op(16, frames, 0, 0);
                    u->setDisabledUntil(DISABLED_HACKED, sc.frame + frames);
                    record("", 0);
                    break;
                }
                case 17: {
                    AIUpdateInterface* ai = u->getAIUpdateInterface();
                    if (!ai) continue;
                    const int on = chance(80);
                    const int fallback = static_cast<int>(pick(4));
                    std::vector<int> values;
                    priority.reset();
                    priority.friend_setDefaultPriority(fallback);
                    std::string list;
                    for (int n = 0; n < kTargetTemplates; ++n) {
                        const int v = chance(40) ? -1 : static_cast<int>(pick(12));
                        if (v >= 0) priority.setPriority(TheThingFactory->findTemplate(fixtures[n].name), v);
                        list += "," + std::to_string(v);
                    }
                    ops += (ops.empty() ? "" : ",") + std::string("[17,") + std::to_string(on) + "," +
                           std::to_string(fallback) + list + "]";
                    ai->setAttackInfo(on ? &priority : nullptr);
                    record("", 0);
                    break;
                }
                case 18: {
                    SpecialPowerModuleInterface* spm = u->getSpecialPowerModule(tmpl);
                    if (!spm) continue;
                    op(18, k, 0, 0);
                    spm->startPowerRecharge();
                    record("", 0);
                    break;
                }
                case 19: {
                    // Disguise a truck as the hunter or as another object.
                    int truck = -1;
                    for (size_t n = 0; n < sc.fixed.size(); ++n) {
                        Object* o = TheGameLogic->findObjectByID(sc.fixed[n]);
                        if (o && o->getTemplate()->getName() == "AB_Truck") truck = static_cast<int>(n);
                    }
                    if (truck < 0) continue;
                    const int as_self = chance(60);
                    Object* like = as_self ? u : target;
                    if (!like) continue;
                    op(19, truck, as_self ? -1 : t, 0);
                    Object* o = TheGameLogic->findObjectByID(sc.fixed[truck]);
                    o->getStealth()->disguiseAsObject(like);
                    o->setStatus(MAKE_OBJECT_STATUS_MASK2(OBJECT_STATUS_STEALTHED, OBJECT_STATUS_DISGUISED), TRUE);
                    record("", 0);
                    break;
                }
            }
        }
        std::fprintf(out, "\"ops\":[%s],\"rows\":[", ops.c_str());
        for (size_t r = 0; r < rows.size(); ++r) std::fprintf(out, "%s\n\"%s\"", r ? "," : "", rows[r].c_str());
        std::fprintf(out, "]}");
        (TheGameLogic->*ab_overrides_member()).clear();
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return 0;
}
