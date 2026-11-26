// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for ToppleUpdate (PKT-383), StructureToppleUpdate (PKT-377) and
// AutoDepositUpdate (PKT-305), run on the unmodified reference modules in a small
// real world (neutral player, players A and B, flat-normal sloped terrain).
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt383_377_305_topple.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT383_377_305_Topple.json
//
// Observed through: FX nuggets (list and position), OCL nuggets (list and primary
// position), the crushing weapon's historic damage list (shot positions),
// addFloatingText (position and amount), money, score keeper and the logic seed
// CRC. The client seed is probed once at the end of each vector.
//
// Not covered: SwayClientUpdate and shadows (client), the pathfind map removal of
// ToppleLeftOrRightOnly, an unknown StumpName (the reference throws), BoneFXUpdate
// and updateBodyParticleSystems (client), a contained AutoDeposit object, an
// inactive local player, loading a save, and the OCL/AI/weapon effects themselves.
// `getUpgradedSupplyBoost` caches its upgrade template for the whole process, so
// every vector shares the first lookup, as the vectors record.

#include "oracle_wp51_world.h"

#include "Common/Money.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/ScoreKeeper.h"
#include "Common/Team.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AutoDepositUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/StructureToppleUpdate.h"
#include "GameLogic/Module/ToppleUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

struct TopPhysVelTag { using type = Coord3D PhysicsBehavior::*; };
template struct MemberAccess<TopPhysVelTag, &PhysicsBehavior::m_vel>;
inline Coord3D PhysicsBehavior::* access_member(TopPhysVelTag);

struct TopHistoricTag { using type = HistoricWeaponDamageList WeaponTemplate::*; };
template struct MemberAccess<TopHistoricTag, &WeaponTemplate::m_historicDamage>;
inline HistoricWeaponDamageList WeaponTemplate::* access_member(TopHistoricTag);

static uint32_t g_state = 0x38337730u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
template <size_t N>
static const char* choose(const char* const (&list)[N]) { return list[pick(N)]; }

static uint32_t bits(Real v) {
    uint32_t u;
    std::memcpy(&u, &v, 4);
    return u;
}
static Real real_of(uint32_t u) {
    Real v;
    std::memcpy(&v, &u, 4);
    return v;
}
// A value in [lo, hi] on a 1/64 grid.
static Real grid(int lo, int hi) {
    return static_cast<Real>(lo * 64 + static_cast<int>(pick((hi - lo) * 64 + 1))) / 64.0f;
}
static void die(const char* why) {
    std::fprintf(stderr, "%s\n", why);
    std::exit(1);
}

// ---------------------------------------------------------------------------------------------
// Recorders
// ---------------------------------------------------------------------------------------------
static std::ostringstream g_fx, g_ocl, g_text;
static const int kFxLists = 9;
static const char* const kFxNames[kFxLists] = {"P383_Topple", "P383_Bounce", "P377_Start",
    "P377_Delay", "P377_Done", "P377_Crush", "P377_Toppling", "P377_Angle1", "P377_Angle2"};
static const int kOcls = 3;
static const char* const kOclNames[kOcls] = {"P377_O0", "P377_O1", "P377_O2"};

class RecordingFx : public FXNugget {
public:
    int list = 0;
    void doFXPos(const Coord3D* p, const Matrix3D*, const Real, const Coord3D*, const Real) const override {
        g_fx << list << ',' << bits(p->x) << ',' << bits(p->y) << ',' << bits(p->z) << ';';
    }
protected:
    MemoryPool* getObjectMemoryPool() override { return nullptr; }
};
static RecordingFx g_fx_nugget[kFxLists];

class RecordingOcl : public ObjectCreationNugget {
public:
    int list = 0;
    Object* create(const Object* primaryObj, const Coord3D* primary, const Coord3D* secondary, Real,
                   UnsignedInt lifetime) const override {
        if (secondary || lifetime) die("unexpected OCL arguments");
        g_ocl << list << ',' << bits(primary->x) << ',' << bits(primary->y) << ',' << bits(primary->z) << ';';
        return nullptr;
    }
    Object* create(const Object*, const Object*, UnsignedInt) const override {
        die("unexpected object OCL call");
        return nullptr;
    }
protected:
    MemoryPool* getObjectMemoryPool() override { return nullptr; }
};
static RecordingOcl g_ocl_nugget[kOcls];

class CashText : public oracle_wp51::NullGameText {
public:
    UnicodeString fetch(const Char* label, Bool* exists = nullptr) override {
        if (exists) *exists = TRUE;
        if (std::strcmp(label, "GUI:AddCash") != 0) return UnicodeString::TheEmptyString;
        return UnicodeString(L"%d");
    }
    UnicodeString fetch(AsciiString label, Bool* exists = nullptr) override { return fetch(label.str(), exists); }
};

class CashUI : public oracle_wp51::NullInGameUI {
public:
    void addFloatingText(const UnicodeString& text, const Coord3D* pos, Color) override {
        std::string digits;
        for (int i = 0; i < text.getLength(); ++i) digits += static_cast<char>(text.getCharAt(i));
        g_text << bits(pos->x) << ',' << bits(pos->y) << ',' << bits(pos->z) << ',' << digits << ';';
    }
};

class SlopeTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return x * 0.01f + y * 0.02f;
    }
};

// ---------------------------------------------------------------------------------------------
// Fixture
// ---------------------------------------------------------------------------------------------
static const int kToppleVariants = 24;
static const int kStructVariants = 32;
static const int kDepositVariants = 32;
static int g_topple_health[kToppleVariants];
static int g_struct_health[kStructVariants];

static std::string fx_line(const char* field, const char* name) {
    if (pick(5) == 0) return std::string();
    return std::string("    ") + field + " = " + (pick(8) == 0 ? "None" : name) + "\n";
}
static std::string opt(const char* field, const char* value) {
    return value ? std::string("    ") + field + " = " + value + "\n" : std::string();
}

static std::string fixture_ini() {
    std::string s;
    s += "Upgrade P305_U1\n  Type = PLAYER\nEnd\nUpgrade P305_U2\n  Type = PLAYER\nEnd\n";
    // HistoricBonusTime keeps the shots of one frame in the list.
    s += "Weapon P377_Crush\n  PrimaryDamage = 1.0\n  PrimaryDamageRadius = 0.0\n  AttackRange = 100000.0\n"
         "  HistoricBonusCount = 1000000\n  HistoricBonusTime = 100000\n  HistoricBonusRadius = 50.0\n"
         "  DelayBetweenShots = 1000\n  ClipSize = 0\nEnd\n";
    for (int i = 0; i < 2; ++i)
        s += "Object P383_Stump" + std::to_string(i) + "\n  KindOf = PRELOAD SHRUBBERY\n"
             "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = " + (i ? "5000000.0" : "50.0") +
             "\n    InitialHealth = " + (i ? "5000000.0" : "50.0") + "\n  End\nEnd\n";
    for (int i = 0; i < 8; ++i) {
        s += "Object P383_Crusher" + std::to_string(i) + "\n  KindOf = PRELOAD VEHICLE\n"
             "  CrusherLevel = " + std::to_string(i / 2) + "\n"
             "  Body = InactiveBody ModuleTag_Body\n  End\n";
        if (i % 2) s += "  Behavior = PhysicsBehavior ModuleTag_Physics\n  End\n";
        s += "End\n";
    }
    s += "Object P377_Attacker\n  KindOf = PRELOAD VEHICLE\n  Body = InactiveBody ModuleTag_Body\n  End\nEnd\n";

    static const char* const kPct[] = {nullptr, nullptr, "0%", "1%", "10%", "20%", "33%", "100%", "250%"};
    static const char* const kBounce[] = {nullptr, nullptr, "0%", "10%", "30%", "60%", "90%", "100%"};
    static const char* const kYesNo[] = {nullptr, "Yes", "No", "No"};
    static const char* const kMostlyNo[] = {nullptr, "No", "No", "Yes"};
    for (int i = 0; i < kToppleVariants; ++i) {
        const bool structure = pick(3) == 0;
        const int health = g_topple_health[i] = pick(4) == 0 ? 3000000 : 100;
        s += "Object P383_T" + std::to_string(i) + "\n  KindOf = PRELOAD " + (structure ? "STRUCTURE" : "SHRUBBERY") + "\n";
        s += "  Geometry = CYLINDER\n  GeometryMajorRadius = 6.0\n  GeometryHeight = " +
             std::to_string(10 + pick(60)) + ".0\n";
        if (structure && pick(2)) s += "  StructureRubbleHeight = " + std::to_string(1 + pick(9)) + "\n";
        s += "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = " + std::to_string(health) +
             ".0\n    InitialHealth = " + std::to_string(health) + ".0\n  End\n";
        s += "  Behavior = ToppleUpdate ModuleTag_Topple\n";
        s += fx_line("ToppleFX", "P383_Topple");
        s += fx_line("BounceFX", "P383_Bounce");
        if (pick(2)) s += "    StumpName = P383_Stump" + std::to_string(pick(2)) + "\n";
        s += opt("KillWhenStartToppling", i % 8 == 7 ? "Yes" : nullptr);
        s += opt("KillWhenFinishedToppling", choose(kYesNo));
        s += opt("KillStumpWhenToppled", choose(kYesNo));
        s += opt("ToppleLeftOrRightOnly", choose(kMostlyNo));
        s += opt("ReorientToppledRubble", choose(kYesNo));
        // Every sixth variant keeps its speed, so that a bounce can land exactly
        // on the 0.01 and 0.03 limits.
        const bool exact = i % 6 == 1;
        s += opt("InitialVelocityPercent", exact ? "100%" : choose(kPct));
        s += opt("InitialAccelPercent", exact ? "0%" : choose(kPct));
        s += opt("BounceVelocityPercent", exact ? "100%" : choose(kBounce));
        s += "  End\nEnd\n";
    }

    static const char* const kDelay[] = {nullptr, "0", "1", "100", "500", "1000", "2000"};
    static const char* const kIntegrity[] = {nullptr, "0.0", "0.1", "0.5", "0.9", "0.99", "1.0", "-0.5"};
    static const char* const kDecay[] = {nullptr, "0.0", "0.5", "0.9", "0.99", "1.0", "-1.0"};
    static const char* const kDamageTypes[] = {nullptr, nullptr, "ALL", "NONE", "ALL -EXPLOSION", "NONE +CRUSH",
        "ALL -UNRESISTABLE", "NONE +EXPLOSION +SMALL_ARMS", "ALL -HEALING"};
    static const char* const kDeathTypes[] = {nullptr, nullptr, nullptr, "ALL", "ALL -CRUSHED", "NONE +NORMAL"};
    static const char* const kVet[] = {nullptr, nullptr, nullptr, "ALL", "ALL -REGULAR"};
    static const char* const kRequired[] = {nullptr, nullptr, nullptr, nullptr, "SOLD"};
    static const char* const kExempt[] = {nullptr, nullptr, nullptr, "SOLD", "UNDER_CONSTRUCTION"};
    static const char* const kAngles[] = {"0.0", "1.0", "5.0", "20.0", "45.0", "70.0", "89.0", "90.0", "120.0"};
    for (int i = 0; i < kStructVariants; ++i) {
        const int health = g_struct_health[i] = 100;
        s += "Object P377_S" + std::to_string(i) + "\n  KindOf = PRELOAD STRUCTURE\n";
        s += std::string("  Geometry = ") + (pick(2) ? "BOX" : "CYLINDER") + "\n";
        s += "  GeometryMajorRadius = " + std::to_string(1 + pick(40)) + ".0\n";
        s += "  GeometryMinorRadius = " + std::to_string(1 + pick(40)) + ".0\n";
        s += "  GeometryHeight = " + std::to_string(pick(8) == 0 ? 0 : 5 + pick(110)) + ".0\n";
        if (pick(2)) s += "  StructureRubbleHeight = " + std::to_string(1 + pick(9)) + "\n";
        s += "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = " + std::to_string(health) +
             ".0\n    InitialHealth = " + std::to_string(health) + ".0\n  End\n";
        s += "  Behavior = StructureToppleUpdate ModuleTag_Fall\n";
        s += opt("MinToppleDelay", choose(kDelay));
        s += opt("MaxToppleDelay", choose(kDelay));
        s += opt("MinToppleBurstDelay", choose(kDelay));
        s += opt("MaxToppleBurstDelay", choose(kDelay));
        s += opt("StructuralIntegrity", choose(kIntegrity));
        s += opt("StructuralDecay", choose(kDecay));
        s += opt("DamageFXTypes", choose(kDamageTypes));
        s += fx_line("ToppleStartFX", "P377_Start");
        s += fx_line("ToppleDelayFX", "P377_Delay");
        s += fx_line("ToppleDoneFX", "P377_Done");
        s += fx_line("CrushingFX", "P377_Crush");
        s += fx_line("TopplingFX", "P377_Toppling");
        const uint32_t weapon = pick(6);
        if (weapon) s += std::string("    CrushingWeaponName = ") + (weapon == 1 ? "P377_NoSuchWeapon" : "P377_Crush") + "\n";
        for (int phase = 0; phase < 3; ++phase) {
            if (pick(3) == 0) continue;
            static const char* const kPhase[] = {"INITIAL", "DELAY", "FINAL"};
            s += std::string("    OCL = ") + kPhase[phase];
            const int n = 1 + static_cast<int>(pick(3));
            for (int k = 0; k < n; ++k) s += std::string(" ") + (pick(5) == 0 ? "None" : kOclNames[pick(kOcls)]);
            s += "\n";
        }
        const int angles = static_cast<int>(pick(4));
        for (int k = 0; k < angles; ++k)
            s += std::string("    AngleFX = ") + choose(kAngles) + " " + (pick(2) ? "P377_Angle1" : "P377_Angle2") + "\n";
        s += opt("DeathTypes", choose(kDeathTypes));
        s += opt("VeterancyLevels", choose(kVet));
        s += opt("RequiredStatus", choose(kRequired));
        s += opt("ExemptStatus", choose(kExempt));
        s += "  End\nEnd\n";
    }

    static const char* const kTiming[] = {"0", "1", "33", "100", "500", "1000", "4000", nullptr};
    static const char* const kAmount[] = {nullptr, "0", "-5", "1", "10", "10", "75", "100"};
    static const char* const kCapture[] = {nullptr, "0", "-3", "1", "50", "500", "1000"};
    static const char* const kActual[] = {nullptr, "Yes", "No", "Yes"};
    for (int i = 0; i < kDepositVariants; ++i) {
        const bool structure = pick(3) != 0;
        s += "Object P305_D" + std::to_string(i) + "\n  KindOf = PRELOAD " +
             (structure ? "STRUCTURE" : "VEHICLE") + (i % 9 == 8 ? " DISGUISER" : "") + "\n";
        static const char* const kRadius[] = {"1.0", "3.0", "4.0", "10.0", "33.0", "80.0"};
        s += "  Geometry = BOX\n  GeometryMajorRadius = " + std::string(choose(kRadius)) +
             "\n  GeometryMinorRadius = " + choose(kRadius) + "\n  GeometryHeight = 10.0\n";
        s += "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n    InitialHealth = 100.0\n  End\n";
        s += "  Behavior = AutoDepositUpdate ModuleTag_Cash\n";
        s += opt("DepositTiming", choose(kTiming));
        // Variant 1's boost cancels its deposit exactly.
        const char* amount = choose(kAmount);
        // Disguisers (every ninth) always deposit, so that their visibility shows.
        s += opt("DepositAmount", i == 1 || i % 9 == 8 ? "10" : amount);
        s += opt("InitialCaptureBonus", choose(kCapture));
        s += opt("ActualMoney", choose(kActual));
        // Variant 0 is looked up first by the generator: its first name is what
        // every later lookup tests.
        static const char* const kBoost[] = {"P305_U1", "P305_U2"};
        static const char* const kExtra[] = {"5", "7", "-20", "-200", "0", "1000", "-10", "-1"};
        const int boosts = i == 0 ? 2 : (i == 1 ? 0 : static_cast<int>(pick(4)) - 1);
        // The cached template is variant 0's first name, P305_U2.
        if (i == 1) s += "    UpgradedBoost = UpgradeType:P305_U2 Boost:-10\n";
        for (int k = 0; k < boosts; ++k)
            s += std::string("    UpgradedBoost = UpgradeType:") + (i == 0 ? kBoost[1 - k] : choose(kBoost)) +
                 " Boost:" + choose(kExtra) + "\n";
        s += "  End\nEnd\n";
    }
    return s;
}

// ---------------------------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------------------------
template <typename T>
static T* find_behavior(Object* obj) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (T* typed = dynamic_cast<T*>(*m)) return typed;
    }
    return nullptr;
}

static void step(oracle_wp51::World& w, UpdateModule* u, UnsignedInt frame) {
    w.set_frame(frame);
    if (u->friend_getNextCallFrame() > frame) return;
    UpdateSleepTime sleep_len = u->update();
    if (sleep_len < 1) sleep_len = UPDATE_SLEEP_NONE;
    UnsignedInt next = frame + static_cast<UnsignedInt>(sleep_len);
    if (next > UPDATE_SLEEP_FOREVER) next = UPDATE_SLEEP_FOREVER;
    u->friend_setNextCallFrame(next);
}

static std::string head(UpdateModule* m, uint32_t ret) {
    std::ostringstream out;
    out << oracle_wp51::World::snapshot_save_hex(m) << ' ' << oracle_wp51::World::snapshot_crc(m) << ' ' << ret << ' '
        << m->friend_getNextCallFrame();
    return out.str();
}

static std::string matrix_of(const Object* o) {
    std::ostringstream out;
    const Matrix3D* m = o->getTransformMatrix();
    for (int r = 0; r < 3; ++r)
        for (int c = 0; c < 4; ++c) out << (r || c ? "," : "") << bits((*m)[r][c]);
    return out.str();
}

static int flag(const Object* o, ModelConditionFlagType f) {
    const Drawable* d = o->getDrawable();
    if (!d) die("object without a drawable");
    return d->getModelConditionFlags().test(f) ? 1 : 0;
}

static std::string take(std::ostringstream& s) {
    std::string v = s.str();
    s.str("");
    return v.empty() ? std::string("-") : v;
}

static std::string take_fires() {
    const WeaponTemplate* t = TheWeaponStore->findWeaponTemplate("P377_Crush");
    HistoricWeaponDamageList& list = const_cast<WeaponTemplate*>(t)->*access_member(TopHistoricTag());
    std::ostringstream out;
    for (const HistoricWeaponDamageInfo& h : list)
        out << bits(h.location.x) << ',' << bits(h.location.y) << ',' << bits(h.location.z) << ';';
    list.clear();
    const std::string v = out.str();
    return v.empty() ? std::string("-") : v;
}

struct Op {
    uint32_t v[6] = {0, 0, 0, 0, 0, 0};
};

static void emit(FILE* out, const std::vector<Op>& ops, const std::vector<std::string>& rows) {
    std::fprintf(out, "\"ops\":[");
    for (size_t k = 0; k < ops.size(); ++k) {
        std::fprintf(out, "%s[%u,%u,%u,%u,%u,%u]", k ? "," : "", ops[k].v[0], ops[k].v[1], ops[k].v[2], ops[k].v[3],
                     ops[k].v[4], ops[k].v[5]);
    }
    std::fprintf(out, "],\"rows\":[");
    for (size_t r = 0; r < rows.size(); ++r) std::fprintf(out, "%s\"%s\"", r ? "," : "", rows[r].c_str());
    std::fprintf(out, "]");
}

static void set_name_and_direction(Object* o, const std::string& name, const Op& op) {
    o->setName(name.c_str());
    Coord3D d;
    d.x = real_of(op.v[1]);
    d.y = real_of(op.v[2]);
    d.z = real_of(op.v[3]);
    TheScriptEngine->setToppleDirection(name.c_str(), &d);
}

static Op script_op(uint32_t code) {
    Op op;
    op.v[0] = code;
    const bool zero = pick(6) == 0;
    op.v[1] = bits(zero ? 0.0f : grid(-4, 4));
    op.v[2] = bits(zero ? 0.0f : grid(-4, 4));
    op.v[3] = bits(pick(2) ? 0.0f : grid(-4, 4));
    return op;
}

static void destroy_all(oracle_wp51::World& w) {
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) TheGameLogic->destroyObject(o);
    w.flush_destroy_list();
    ThePartitionManager->update();
}

// ---------------------------------------------------------------------------------------------
// ToppleUpdate
// ---------------------------------------------------------------------------------------------
static std::string topple_row(Object* o, ToppleUpdate* m, uint32_t ret, ObjectID stump_id) {
    std::ostringstream out;
    out << head(m, ret) << ' ' << matrix_of(o) << ' ' << bits(o->getBodyModule()->getHealth()) << ' '
        << (o->isEffectivelyDead() ? 1 : 0) << flag(o, MODELCONDITION_TOPPLED) << flag(o, MODELCONDITION_BURNED) << ' '
        << GetGameLogicRandomSeedCRC() << ' ' << take(g_fx) << ' ';
    // The stump is the object created after this one.
    Object* stump = TheGameLogic->findObjectByID(stump_id);
    if (!stump) {
        out << '-';
    } else {
        out << matrix_of(stump) << ',' << bits(stump->getBodyModule()->getHealth()) << ','
            << (stump->isEffectivelyDead() ? 1 : 0) << ',' << flag(stump, MODELCONDITION_BURNED) << ','
            << (stump->getTeam() == ThePlayerList->getNeutralPlayer()->getDefaultTeam() ? 1 : 0);
    }
    return out.str();
}

static void run_topple(oracle_wp51::World& w, FILE* out, int index, uint32_t variant, uint32_t seed,
                       UnsignedInt frame) {
    const std::string tmpl = "P383_T" + std::to_string(variant);
    Coord3D pos;
    pos.x = grid(300, 700);
    pos.y = grid(300, 700);
    pos.z = pick(2) ? 0.0f : grid(0, 20);
    const Real angle = pick(8) == 0 ? 0.0f : (pick(4) == 0 ? grid(-12, 12) : grid(-3, 3));
    Object* o = w.spawn_object(tmpl.c_str(), w.team_a, pos, angle);
    ToppleUpdate* m = find_behavior<ToppleUpdate>(o);
    if (!m) die("missing ToppleUpdate");
    ThePartitionManager->update();
    // Only this object and its stump get the ids 1 and 2.
    const ObjectID stump_id = static_cast<ObjectID>(o->getID() + 1);
    // Crushers are made afterwards from a high id so that the stump's id holds.
    Object* crushers[8] = {};

    std::fprintf(out, "{\"kind\":0,\"variant\":%u,\"seed\":%u,\"frame\":%u,\"x\":%u,\"y\":%u,\"z\":%u,\"angle\":%u,",
                 variant, seed, frame, bits(pos.x), bits(pos.y), bits(pos.z), bits(angle));
    std::fprintf(out, "\"spawn\":\"%s\",", topple_row(o, m, 0, stump_id).c_str());

    std::vector<Op> ops;
    std::vector<std::string> rows;
    const int count = 4 + static_cast<int>(pick(14));
    for (int k = 0; k < count; ++k) {
        Op op;
        uint32_t code = pick(16);
        if (code >= 8) code = code < 14 ? 1 : (code == 14 ? 0 : 2);
        if (k == 0 && pick(2)) code = pick(4) == 0 ? 6 : (pick(5) == 0 ? 5 : (pick(3) == 0 ? 2 : 0));
        if (code == 7 && pick(2)) code = 1;
        op.v[0] = code;
        uint32_t ret = 0;
        w.set_frame(frame);
        if (code == 0) {
            const bool zero = pick(12) == 0;
            op.v[1] = bits(zero ? 0.0f : grid(-8, 8));
            op.v[2] = bits(zero ? 0.0f : grid(-8, 8));
            op.v[3] = bits(pick(3) ? 0.0f : grid(-8, 8));
            // Across the object's axis at angle 0, both sides are equally near.
            if (angle == 0.0f && pick(2)) {
                op.v[1] = bits(pick(2) ? 1.0f : -1.0f);
                op.v[2] = bits(0.0f);
                op.v[3] = bits(0.0f);
            }
            static const float kSpeed[] = {0.0f, 0.001f, 0.05f, 0.2f, 0.5f, 1.0f, 2.0f, 7.0f, 60.0f, -1.0f, 0.01f,
                                           0.03f};
            op.v[4] = bits(pick(3) ? kSpeed[pick(12)] : grid(0, 12));
            if (variant % 6 == 1 && pick(3)) op.v[4] = bits(pick(2) ? 0.01f : 0.03f);
            op.v[5] = pick(4);
            Coord3D dir;
            dir.x = real_of(op.v[1]);
            dir.y = real_of(op.v[2]);
            dir.z = real_of(op.v[3]);
            o->topple(&dir, real_of(op.v[4]), op.v[5]);
        } else if (code == 1) {
            op.v[1] = 1 + pick(pick(2) ? 5 : (pick(4) ? 80 : 200));
            // A constant 0.01 rad per frame needs 152 frames to reach the limit.
            if (variant % 6 == 1) op.v[1] = 1 + pick(250);
            for (uint32_t t = 0; t < op.v[1]; ++t) step(w, m, ++frame);
        } else if (code == 2) {
            const uint32_t kind = pick(3) == 0 ? pick(8) : 4 + pick(4);
            const Real dx = pick(8) == 0 ? 0.0f : grid(-20, 20);
            const Real dy = pick(8) == 0 ? 0.0f : grid(-20, 20);
            const Real vx = grid(-6, 6), vy = grid(-6, 6);
            op.v[1] = kind;
            op.v[2] = bits(dx);
            op.v[3] = bits(dy);
            op.v[4] = bits(vx);
            op.v[5] = bits(vy);
            if (!crushers[kind]) {
                const ObjectID saved = TheGameLogic->getObjectIDCounter();
                TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(100 + kind));
                crushers[kind] = w.spawn_object(("P383_Crusher" + std::to_string(kind)).c_str(), w.team_b, pos, 0.0f);
                TheGameLogic->setObjectIDCounter(saved);
            }
            Object* c = crushers[kind];
            Coord3D cpos;
            cpos.x = o->getPosition()->x + dx;
            cpos.y = o->getPosition()->y + dy;
            cpos.z = 3.0f;
            c->setPosition(&cpos);
            if (PhysicsBehavior* phys = c->getPhysics()) {
                Coord3D& vel = phys->*access_member(TopPhysVelTag());
                vel.x = vx;
                vel.y = vy;
                vel.z = vx * 0.5f;
            }
            Coord3D loc = cpos, normal;
            normal.x = 0.0f;
            normal.y = 0.0f;
            normal.z = 1.0f;
            o->onCollide(c, &loc, &normal);
        } else if (code == 3) {
            ret = static_cast<uint32_t>(m->update());
        } else if (code == 4) {
            o->onCollide(nullptr, o->getPosition(), o->getPosition());
        } else if (code == 5) {
            o->getDrawable()->setModelConditionState(MODELCONDITION_BURNED);
        } else if (code == 6) {
            op = script_op(6);
            set_name_and_direction(o, "P383_N" + std::to_string(index), op);
        } else {
            o->kill();
        }
        ops.push_back(op);
        rows.push_back(topple_row(o, m, ret, stump_id));
    }
    emit(out, ops, rows);
}

// ---------------------------------------------------------------------------------------------
// StructureToppleUpdate
// ---------------------------------------------------------------------------------------------
static std::string struct_row(Object* o, StructureToppleUpdate* m, uint32_t ret) {
    std::ostringstream out;
    out << head(m, ret) << ' ' << matrix_of(o) << ' ' << (o->isEffectivelyDead() ? 1 : 0)
        << flag(o, MODELCONDITION_RUBBLE) << flag(o, MODELCONDITION_POST_COLLAPSE) << ' ' << GetGameLogicRandomSeedCRC()
        << ' ' << take(g_fx) << ' ' << take(g_ocl) << ' ' << take_fires();
    return out.str();
}

static Op damage_op(uint32_t code) {
    static const uint32_t kDamage[] = {DAMAGE_EXPLOSION, DAMAGE_CRUSH, DAMAGE_SMALL_ARMS, DAMAGE_UNRESISTABLE,
                                       DAMAGE_FLAME, DAMAGE_ARMOR_PIERCING};
    static const uint32_t kDeath[] = {DEATH_NORMAL, DEATH_NORMAL, DEATH_CRUSHED, DEATH_BURNED, DEATH_TOPPLED};
    Op op;
    op.v[0] = code;
    op.v[1] = kDamage[pick(6)];
    op.v[2] = kDeath[pick(5)];
    op.v[3] = pick(6);  // source: 0 none, 1 unknown id, 2 attacker on top, 3.. attacker offset
    if (op.v[3] > 3) op.v[3] = 3;
    op.v[4] = bits(pick(5) == 0 ? 0.0f : grid(-40, 40));
    op.v[5] = bits(pick(5) == 0 ? 0.0f : grid(-40, 40));
    return op;
}

static void fill_damage(oracle_wp51::World& w, Object* o, const Op& op, Object*& attacker, DamageInfo& info) {
    info.in.m_damageType = static_cast<DamageType>(op.v[1]);
    info.in.m_deathType = static_cast<DeathType>(op.v[2]);
    info.in.m_sourceID = INVALID_ID;
    if (op.v[3] == 1) info.in.m_sourceID = static_cast<ObjectID>(9999);
    if (op.v[3] >= 2) {
        Coord3D p = *o->getPosition();
        if (op.v[3] == 3) {
            p.x += real_of(op.v[4]);
            p.y += real_of(op.v[5]);
        }
        if (!attacker) attacker = w.spawn_object("P377_Attacker", w.team_b, p, 0.0f);
        attacker->setPosition(&p);
        info.in.m_sourceID = attacker->getID();
    }
}

static void run_struct(oracle_wp51::World& w, FILE* out, int index, uint32_t variant, uint32_t seed,
                       UnsignedInt frame) {
    const std::string tmpl = "P377_S" + std::to_string(variant);
    Coord3D pos;
    pos.x = grid(400, 600);
    pos.y = grid(400, 600);
    pos.z = 0.0f;
    const Real angle = grid(-3, 3);
    Object* o = w.spawn_object(tmpl.c_str(), w.team_a, pos, angle);
    StructureToppleUpdate* m = find_behavior<StructureToppleUpdate>(o);
    if (!m) die("missing StructureToppleUpdate");
    ThePartitionManager->update();
    Object* attacker = nullptr;

    std::fprintf(out, "{\"kind\":1,\"variant\":%u,\"seed\":%u,\"frame\":%u,\"x\":%u,\"y\":%u,\"z\":%u,\"angle\":%u,",
                 variant, seed, frame, bits(pos.x), bits(pos.y), bits(pos.z), bits(angle));
    std::fprintf(out, "\"spawn\":\"%s\",", struct_row(o, m, 0).c_str());

    std::vector<Op> ops;
    std::vector<std::string> rows;
    const int count = 5 + static_cast<int>(pick(12));
    bool started = false;
    for (int k = 0; k < count; ++k) {
        uint32_t code = pick(16);
        if (code >= 7) code = 1;
        if (k == 0 && pick(3) == 0) code = pick(2) ? 4 : (pick(2) ? 2 : 6);
        if (k == 1) code = pick(6) == 0 ? 3 : 0;
        // A second death while the building is still falling restarts it.
        if (k > 1 && started && code == 3 && pick(3)) code = 1;
        Op op;
        op.v[0] = code;
        uint32_t ret = 0;
        w.set_frame(frame);
        if (code == 0) {
            op = damage_op(0);
            DamageInfo info;
            fill_damage(w, o, op, attacker, info);
            info.in.m_amount = 100000.0f;
            o->attemptDamage(&info);
            started = true;
        } else if (code == 1) {
            op.v[1] = 1 + pick(pick(2) ? 6 : 60);
            for (uint32_t t = 0; t < op.v[1]; ++t) step(w, m, ++frame);
        } else if (code == 2) {
            op = damage_op(2);
            if (pick(3) == 0) op.v[1] = DAMAGE_HEALING;
            DamageInfo info;
            fill_damage(w, o, op, attacker, info);
            const uint32_t amount = 1 + pick(10);
            const uint32_t bump = pick(2);
            info.in.m_amount = static_cast<Real>(amount);
            // The death type, amount and frame bump share one number.
            op.v[2] |= amount << 8 | bump << 16;
            if (op.v[1] == DAMAGE_HEALING) o->attemptHealing(info.in.m_amount, nullptr);
            else o->attemptDamage(&info);
            // Two damages on one frame exercise the priority rule of the body.
            frame += bump;
        } else if (code == 3) {
            op = damage_op(3);
            DamageInfo info;
            fill_damage(w, o, op, attacker, info);
            for (BehaviorModule** b = o->getBehaviorModules(); *b; ++b) {
                if (static_cast<void*>(dynamic_cast<StructureToppleUpdate*>(*b)) == static_cast<void*>(m))
                    (*b)->getDie()->onDie(&info);
            }
            started = true;
        } else if (code == 4) {
            op = script_op(4);
            set_name_and_direction(o, "P377_N" + std::to_string(index), op);
        } else if (code == 5) {
            // The reference asserts on a standing building; release builds sleep.
            ret = static_cast<uint32_t>(static_cast<UpdateModule*>(m)->update());
        } else {
            op.v[1] = pick(2) ? OBJECT_STATUS_SOLD : OBJECT_STATUS_UNDER_CONSTRUCTION;
            o->setStatus(MAKE_OBJECT_STATUS_MASK(static_cast<ObjectStatusTypes>(op.v[1])));
        }
        ops.push_back(op);
        rows.push_back(struct_row(o, m, ret));
    }
    emit(out, ops, rows);
}

// ---------------------------------------------------------------------------------------------
// AutoDepositUpdate
// ---------------------------------------------------------------------------------------------
static Team* g_second_team_a = nullptr;

static std::string deposit_row(AutoDepositUpdate* m, uint32_t ret) {
    std::ostringstream out;
    out << head(m, ret) << ' ';
    for (int p = 0; p < 3; ++p) out << (p ? "," : "") << ThePlayerList->getNthPlayer(p)->getMoney()->countMoney();
    out << ' ';
    for (int p = 0; p < 3; ++p)
        out << (p ? "," : "") << ThePlayerList->getNthPlayer(p)->getScoreKeeper()->getTotalMoneyEarned();
    out << ' ' << take(g_text) << ' ' << GetGameLogicRandomSeedCRC();
    return out.str();
}

static Team* team_of(oracle_wp51::World& w, uint32_t code) {
    switch (code) {
        case 0: return w.team_neutral;
        case 1: return w.team_a;
        case 2: return w.team_b;
        default: return g_second_team_a;
    }
}

static void run_deposit(oracle_wp51::World& w, FILE* out, uint32_t variant, uint32_t seed, UnsignedInt frame) {
    const std::string tmpl = "P305_D" + std::to_string(variant);
    Coord3D pos;
    pos.x = grid(300, 700);
    pos.y = grid(300, 700);
    pos.z = grid(0, 20);
    const Real angle = grid(-3, 3);
    uint32_t money[3];
    for (int p = 0; p < 3; ++p) {
        Player* player = ThePlayerList->getNthPlayer(p);
        player->getMoney()->withdraw(player->getMoney()->countMoney(), FALSE);
        money[p] = pick(4) == 0 ? pick(100) : 1000 + pick(100000);
        player->getMoney()->deposit(money[p], FALSE, FALSE);
        player->getScoreKeeper()->reset(p);
        for (const char* name : {"P305_U1", "P305_U2"}) {
            const UpgradeTemplate* u = TheUpgradeCenter->findUpgrade(name);
            if (player->hasUpgradeComplete(u)) player->removeUpgrade(u);
        }
    }
    uint32_t first_team = pick(6) < 4 ? 1 + pick(2) : (pick(2) ? 0 : 3);
    // A stealthed enemy disguiser stays visible to the local player.
    const bool disguised = variant % 9 == 8 && pick(2);
    if (disguised) first_team = 2;
    w.set_frame(frame);
    Object* o = w.spawn_object(tmpl.c_str(), team_of(w, first_team), pos, angle);
    AutoDepositUpdate* m = find_behavior<AutoDepositUpdate>(o);
    if (!m) die("missing AutoDepositUpdate");
    ThePartitionManager->update();

    std::fprintf(out, "{\"kind\":2,\"variant\":%u,\"seed\":%u,\"frame\":%u,\"x\":%u,\"y\":%u,\"z\":%u,\"angle\":%u,",
                 variant, seed, frame, bits(pos.x), bits(pos.y), bits(pos.z), bits(angle));
    std::fprintf(out, "\"team\":%u,\"m0\":%u,\"m1\":%u,\"m2\":%u,", first_team, money[0], money[1], money[2]);
    std::fprintf(out, "\"spawn\":\"%s\",", deposit_row(m, 0).c_str());

    std::vector<Op> ops;
    std::vector<std::string> rows;
    const int count = 6 + static_cast<int>(pick(14));
    for (int k = 0; k < count; ++k) {
        uint32_t code = pick(12);
        if (code >= 6) code = 0;
        if (disguised && k == 0) code = 4;
        Op op;
        op.v[0] = code;
        uint32_t ret = 0;
        w.set_frame(frame);
        if (code == 0) {
            op.v[1] = 1 + pick(pick(3) == 0 ? 90 : 12);
            for (uint32_t t = 0; t < op.v[1]; ++t) step(w, m, ++frame);
        } else if (code == 1) {
            op.v[1] = pick(4);
            o->setTeam(team_of(w, op.v[1]));
        } else if (code == 2) {
            op.v[1] = 1 + pick(2);
            op.v[2] = pick(2);
            op.v[3] = pick(4) != 0;
            Player* player = ThePlayerList->getNthPlayer(op.v[1]);
            const UpgradeTemplate* u = TheUpgradeCenter->findUpgrade(op.v[2] ? "P305_U2" : "P305_U1");
            if (op.v[3]) player->addUpgrade(u, UPGRADE_STATUS_COMPLETE);
            else if (player->hasUpgradeComplete(u)) player->removeUpgrade(u);
        } else if (code == 3) {
            static const float kPercent[] = {-1.0f, -1.0f, 0.0f, 50.0f, 100.0f, -0.5f};
            op.v[1] = bits(kPercent[pick(6)]);
            o->setConstructionPercent(real_of(op.v[1]));
        } else if (code == 4) {
            op.v[1] = pick(2) ? OBJECT_STATUS_DETECTED : OBJECT_STATUS_STEALTHED;
            op.v[2] = pick(3) != 0;
            if (disguised && k == 0) {
                op.v[1] = OBJECT_STATUS_STEALTHED;
                op.v[2] = 1;
            }
            const ObjectStatusMaskType mask = MAKE_OBJECT_STATUS_MASK(static_cast<ObjectStatusTypes>(op.v[1]));
            if (op.v[2]) o->setStatus(mask);
            else o->clearStatus(mask);
        } else {
            ret = static_cast<uint32_t>(static_cast<UpdateModule*>(m)->update());
        }
        ops.push_back(op);
        rows.push_back(deposit_row(m, ret));
    }
    emit(out, ops, rows);
}

int main(int argc, char** argv) {
    if (argc < 2) die("usage: gen_pkt383_377_305_topple <out.json>");
    oracle_wp51::World w;
    TheTerrainLogic = NEW SlopeTerrain;
    TheTerrainLogic->init();
    TheGameText = new CashText;
    TheInGameUI = new CashUI;
    for (int i = 0; i < kFxLists; ++i)
        w.load_ini(std::string("FXList ") + kFxNames[i] + "\nEnd\n");
    for (int i = 0; i < kOcls; ++i)
        w.load_ini(std::string("ObjectCreationList ") + kOclNames[i] + "\nEnd\n");
    for (int i = 0; i < kFxLists; ++i) {
        const FXList* list = TheFXListStore->findFXList(kFxNames[i]);
        if (!list) die("missing FXList");
        g_fx_nugget[i].list = i;
        const_cast<FXList*>(list)->addFXNugget(&g_fx_nugget[i]);
    }
    for (int i = 0; i < kOcls; ++i) {
        const ObjectCreationList* list = TheObjectCreationListStore->findObjectCreationList(kOclNames[i]);
        if (!list) die("missing OCL");
        g_ocl_nugget[i].list = i;
        const_cast<ObjectCreationList*>(list)->addObjectCreationNugget(&g_ocl_nugget[i]);
    }
    const std::string ini = fixture_ini();
    w.load_ini(ini);
    w.init_with_neutral_player();
    // Reveal while player 1 is not yet local: there is no display to update.
    ThePartitionManager->revealMapForPlayerPermanently(1);
    ThePlayerList->setLocalPlayer(ThePlayerList->getNthPlayer(1));
    {
        Dict dict;
        TheTeamFactory->initTeam("TeamA2", AsciiString::TheEmptyString, TRUE, &dict);
        g_second_team_a = TheTeamFactory->findTeam("TeamA2");
        g_second_team_a->setControllingPlayer(ThePlayerList->getNthPlayer(1));
    }

    FILE* out = std::fopen(argv[1], "w");
    if (!out) die("cannot open output");
    const int kTotal = 1800;
    std::string escaped;
    for (char c : ini) {
        if (c == '\n') escaped += "\\n";
        else escaped += c;
    }
    std::fprintf(out, "{\"vector_count\":%d,\n\"ini\":\"%s\",\n\"vectors\":[", kTotal, escaped.c_str());
    for (int i = 0; i < kTotal; ++i) {
        const int kind = i % 3;
        const uint32_t seed = 1 + pick(0x7fffffffu);
        const UnsignedInt frame = 10000 + pick(1000000);
        w.set_frame(frame);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        InitRandom(seed);
        take(g_fx);
        take(g_ocl);
        take(g_text);
        take_fires();
        std::fprintf(out, "%s\n", i ? "," : "");
        // The first deposit vector uses variant 0: it fixes the cached upgrade.
        if (kind == 0) run_topple(w, out, i, pick(kToppleVariants), seed, frame);
        else if (kind == 1) run_struct(w, out, i, pick(kStructVariants), seed, frame);
        else run_deposit(w, out, i == 2 ? 0 : pick(kDepositVariants), seed, frame);
        std::fprintf(out, ",\"client\":%d}", GameClientRandomValue(0, 1000000));
        destroy_all(w);
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return 0;
}
