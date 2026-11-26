// WP-5.5 weapon firing state machine and PKT-329 FireWeaponUpdate.
//
// The unmodified vendor Weapon.cpp is compiled into this translation unit. Only the
// template-level `WeaponTemplate::fireWeaponTemplate` is intercepted: its declaration and
// definition are renamed to `fwt_unused`, and the calls in `Weapon::privateFireWeapon` are
// redirected to a recorder. The two forms are told apart by their first token (`const` for the
// declaration and definition, `sourceObj` for the calls).
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp55_fire.cpp --opt O0 -o <dir>/gen_wp55_fire_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp55_fire.cpp --opt O2 -o <dir>/gen_wp55_fire_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp55_fire.cpp --opt O2 --sanitize -o <dir>/gen_wp55_fire_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_wp55_fire_o2 docs/golden_vectors/WP55_FIRE.json

#define FWT_FIRST_const ~, FWT_AS_DECL,
#define FWT_FIRST_sourceObj ~, FWT_AS_CALL,
#define FWT_SECOND_(a, b, ...) b
#define FWT_SECOND(...) FWT_SECOND_(__VA_ARGS__)
#define fireWeaponTemplate(first, ...) FWT_SECOND(FWT_FIRST_##first)(first, __VA_ARGS__)
#define FWT_AS_DECL(...) fwt_unused(__VA_ARGS__)
#define FWT_AS_CALL(...) getName(), oracle_record_shot(__VA_ARGS__)

// Barrel counts come from draw modules, which a headless oracle has none of. A static accessor is
// added to Drawable (no layout change) and the single `getBarrelCount` call in Weapon.cpp is sent
// to it; the static call never touches the null drawable pointer.
// The name tables Weapon.cpp asks for; its own includes come too late once Weapon.h is in.
#define DEFINE_DEATH_NAMES
#define DEFINE_WEAPONBONUSCONDITION_NAMES
#define DEFINE_WEAPONBONUSFIELD_NAMES
#define DEFINE_WEAPONCOLLIDEMASK_NAMES
#define DEFINE_WEAPONAFFECTSMASK_NAMES
#define DEFINE_WEAPONRELOAD_NAMES
#define DEFINE_WEAPONPREFIRE_NAMES
#include "PreRTS.h"
#include "Common/GameCommon.h"
#define killStaticImages() killStaticImages(); static Int oracle_barrels(Int wslot)
#include "GameClient/Drawable.h"
#undef killStaticImages

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

static Int g_barrels[WEAPONSLOT_COUNT] = {0, 0, 0};
Int Drawable::oracle_barrels(Int wslot) { return g_barrels[wslot]; }

static void oracle_record_shot(const Object* sourceObj, WeaponSlotType wslot, Int barrel,
                               Object* victimObj, const Coord3D* victimPos, const WeaponBonus& bonus,
                               Bool isDetonation, Bool ignoreRanges, const Weapon* firingWeapon,
                               ObjectID* projectileID, Bool inflictDamage);

#define getBarrelCount(w) oracle_barrels(w)
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Weapon.cpp"
#undef getBarrelCount
#undef fireWeaponTemplate

#include "oracle_wp51_world.h"

#include "Common/RandomValue.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/WeaponSet.h"

#include <cstdio>
#include <cstring>
#include <string>

template <class Tag, typename Tag::type M>
struct Wp55Rob { friend typename Tag::type access_member(Tag) { return M; } };
struct Wp55BonusTag { using type = WeaponBonusConditionFlags Object::*; friend type access_member(Wp55BonusTag); };
template struct Wp55Rob<Wp55BonusTag, &Object::m_weaponBonusCondition>;
struct Wp55ContainerTag { using type = Object* Object::*; friend type access_member(Wp55ContainerTag); };
template struct Wp55Rob<Wp55ContainerTag, &Object::m_containedBy>;

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }
static Real real_of(uint32_t b) { Real v; std::memcpy(&v, &b, 4); return v; }

// Height depends on the position so a scatter shot that keeps the old z shows up.
class SlopeTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return x * 0.25f + y * 0.125f;
    }
};

// ----------------------------------------------------------------------------- shot recorder

enum { W_STATE = 0, W_SET0, W_SET1, W_SET2, W_MOD, W_CONT0, W_CONT1, W_REACT, W_COUNT };
static const char* kWeaponName[W_COUNT] = {"WF_STATE", "WF_SET0", "WF_SET1", "WF_SET2",
                                           "WF_MOD",   "WF_CONT0", "WF_CONT1", "WF_REACT"};

struct Shot { uint32_t weapon, victim, x, y, z; };
static std::vector<Shot> g_shots;

static void oracle_record_shot(const Object*, WeaponSlotType, Int, Object* victimObj,
                               const Coord3D* victimPos, const WeaponBonus&, Bool, Bool,
                               const Weapon* firingWeapon, ObjectID*, Bool) {
    uint32_t index = W_COUNT;
    for (int i = 0; i < W_COUNT; ++i) {
        if (firingWeapon->getTemplate()->getName() == AsciiString(kWeaponName[i])) index = i;
    }
    const Coord3D* p = victimPos ? victimPos : victimObj->getPosition();
    g_shots.push_back({index, victimObj ? 1u : 0u, bits(p->x), bits(p->y), bits(p->z)});
}

// ----------------------------------------------------------------------------- input generator

static uint32_t g_state = 0x55F12E01u;
static uint32_t next_u32() {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state;
}
static uint32_t pick(uint32_t n) { return next_u32() % n; }
static Real coord() { return static_cast<Real>(next_u32() % 400000u) / 997.0f + 20.0f; }

// Global weapon bonuses: (condition, field, percent).
struct BonusLine { uint32_t cond, field, pct; };
static const BonusLine kGlobalBonus[] = {
    {0, 3, 115},  // GARRISONED RATE_OF_FIRE
    {1, 3, 137},  // HORDE RATE_OF_FIRE
    {2, 3, 60},   // CONTINUOUS_FIRE_MEAN RATE_OF_FIRE
    {3, 3, 233},  // CONTINUOUS_FIRE_FAST RATE_OF_FIRE
    {4, 0, 125},  // NATIONALISM DAMAGE
    {5, 3, 91},   // PLAYER_UPGRADE RATE_OF_FIRE
};
static const uint32_t kConditionCount = 6;

static std::string bonus_line(const BonusLine& b) {
    return std::string(TheWeaponBonusNames[b.cond]) + " " + TheWeaponBonusFieldNames[b.field] + " " +
           std::to_string(b.pct) + "%";
}

struct Tmpl {
    uint32_t clip, reload, min_delay, max_delay, spb, reload_type, disarm, leech, scalar, suspend,
        pitch;
    std::vector<uint32_t> scatter;  // x bits, y bits pairs
    std::vector<BonusLine> extra;
};

static Tmpl random_tmpl(bool allow_disarm) {
    Tmpl t;
    const uint32_t clips[] = {0, 1, 1, 2, 3, 5};
    t.clip = clips[pick(6)];
    t.reload = pick(4) == 0 ? 0 : 1 + pick(200);
    t.min_delay = pick(3) == 0 ? 0 : 1 + pick(40);
    t.max_delay = pick(3) == 0 ? t.min_delay : t.min_delay + 1 + pick(60);
    if (pick(16) == 0) { t.max_delay = t.min_delay; t.min_delay += 3; }  // inverted range
    t.spb = 1 + pick(3);
    t.reload_type = pick(5) < 3 ? 0 : 1 + pick(2);
    t.disarm = allow_disarm && pick(12) == 0 ? 1 : 0;
    t.leech = pick(6) == 0 ? 1 : 0;
    t.scalar = bits(static_cast<Real>(pick(3000)) / 97.0f);
    t.suspend = pick(3) == 0 ? pick(90) : 0;
    t.pitch = pick(5) == 0 ? 1 : 0;
    if (pick(3) == 0) {
        const uint32_t n = 1 + pick(4);
        for (uint32_t i = 0; i < n * 2; ++i) {
            t.scatter.push_back(bits(static_cast<Real>(pick(2001)) / 101.0f - 9.9f));
        }
    }
    if (pick(4) == 0) {
        const uint32_t n = 1 + pick(2);
        for (uint32_t i = 0; i < n; ++i) {
            t.extra.push_back({pick(kConditionCount), 3, 40 + pick(300)});
        }
    }
    return t;
}

static void apply_tmpl(oracle_wp51::World& w, int index, const Tmpl& t) {
    WeaponTemplate* wt = const_cast<WeaponTemplate*>(
        TheWeaponStore->findWeaponTemplate(kWeaponName[index]));
    wt->m_clipSize = t.clip;
    wt->m_clipReloadTime = t.reload;
    wt->m_minDelayBetweenShots = t.min_delay;
    wt->m_maxDelayBetweenShots = t.max_delay;
    wt->m_shotsPerBarrel = t.spb;
    wt->m_reloadType = static_cast<WeaponReloadType>(t.reload_type);
    wt->m_damageType = t.disarm ? DAMAGE_DISARM : DAMAGE_EXPLOSION;
    wt->m_leechRangeWeapon = t.leech ? TRUE : FALSE;
    wt->m_scatterTargetScalar = real_of(t.scalar);
    wt->m_suspendFXDelay = t.suspend;
    wt->m_minTargetPitch = t.pitch ? -0.5f : -PI;
    wt->m_scatterTargets.clear();
    for (size_t i = 0; i < t.scatter.size(); i += 2) {
        Coord2D c;
        c.x = real_of(t.scatter[i]);
        c.y = real_of(t.scatter[i + 1]);
        wt->m_scatterTargets.push_back(c);
    }
    deleteInstance(wt->m_extraBonus);
    wt->m_extraBonus = nullptr;
    if (!t.extra.empty()) {
        wt->m_extraBonus = newInstance(WeaponBonusSet);
        for (const BonusLine& b : t.extra) {
            w.mem_ini.parse_blocks("WB " + bonus_line(b) + "\n", INI_LOAD_OVERWRITE,
                                   [wt](INI* ini, const char*) {
                                       wt->m_extraBonus->parseWeaponBonusSet(ini);
                                   });
        }
    }
}

// ----------------------------------------------------------------------------- fixture

enum { K_STATE = 0, K_SET, K_TEMP, K_UPDATE, K_DEAD, K_DAMAGED, K_COLLIDE, K_COUNT };
static const char* kKindName[K_COUNT] = {"state", "set", "temp", "update", "dead", "damaged", "collide"};

// FireWeaponUpdate variants: (InitialDelay msec, ExclusiveWeaponDelay msec) and their frames.
static const uint32_t kUpdMs[6][2] = {{0, 0}, {100, 0}, {500, 0}, {0, 200}, {100, 1000}, {0, 4000}};
static const uint32_t kUpdFrames[6][2] = {{0, 0}, {3, 0}, {15, 0}, {0, 6}, {3, 30}, {0, 120}};

static std::string fixture_ini() {
    std::string s;
    for (int i = 0; i < W_COUNT; ++i) s += std::string("Weapon ") + kWeaponName[i] + "\nEnd\n";
    auto unit = [&](const std::string& name, bool shared, const std::string& modules) {
        s += "Object " + name + "\n  KindOf = SELECTABLE\n"
             "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY WF_SET0\n"
             "    Weapon = SECONDARY WF_SET1\n    Weapon = TERTIARY WF_SET2\n";
        if (shared) s += "    ShareWeaponReloadTime = Yes\n";
        s += "  End\n"
             "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n    InitialHealth = 100.0\n  End\n";
        s += modules + "End\n";
    };
    for (int sh = 0; sh < 2; ++sh) {
        const std::string x = std::to_string(sh);
        unit("WF_Plain" + x, sh != 0, "");
        for (int v = 0; v < 6; ++v) {
            unit("WF_Update" + std::to_string(v) + "_" + x, sh != 0,
                 "  Behavior = FireWeaponUpdate ModuleTag_M\n    Weapon = WF_MOD\n    InitialDelay = " +
                     std::to_string(kUpdMs[v][0]) + "\n    ExclusiveWeaponDelay = " +
                     std::to_string(kUpdMs[v][1]) + "\n  End\n");
        }
        unit("WF_Dead" + x, sh != 0,
             "  Behavior = FireWeaponWhenDeadBehavior ModuleTag_M\n    DeathWeapon = WF_MOD\n"
             "    StartsActive = Yes\n  End\n");
        unit("WF_Damaged" + x, sh != 0,
             "  Behavior = FireWeaponWhenDamagedBehavior ModuleTag_M\n    StartsActive = Yes\n"
             "    ReactionWeaponPristine = WF_REACT\n    ReactionWeaponDamaged = WF_REACT\n"
             "    ContinuousWeaponPristine = WF_CONT0\n    ContinuousWeaponDamaged = WF_CONT1\n  End\n");
        unit("WF_Collide" + x, sh != 0,
             "  Behavior = FireWeaponCollide ModuleTag_M\n    CollideWeapon = WF_MOD\n"
             "    FireOnce = No\n  End\n");
    }
    s += "Object WF_Box\n  KindOf = SELECTABLE\n"
         "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n    InitialHealth = 100.0\n  End\n"
         "  Behavior = TransportContain ModuleTag_C\n    Slots = 4\n"
         "    WeaponBonusPassedToPassengers = Yes\n  End\nEnd\n";
    return s;
}

// ----------------------------------------------------------------------------- vectors

enum {
    OP_ADVANCE = 0, OP_LOAD_NOW, OP_RELOAD, OP_FIRE_POS, OP_FIRE_OBJ, OP_FORCE, OP_STATUS,
    OP_CLIP_PCT, OP_BONUS, OP_CONTAIN, OP_TEMP_POS, OP_TEMP_OBJ, OP_STEP, OP_DAMAGE, OP_KILL,
    OP_COLLIDE, OP_BUILDING
};

struct Op { uint32_t code, a, b, c; };
struct Snap { std::string xfer; uint32_t crc; };
struct Row { uint32_t ret, rng; std::vector<Snap> snaps; std::vector<Shot> shots; };
struct Vec {
    uint32_t kind, shared, variant, frame, seed, barrels[3], pos[3], victim[3];
    Tmpl tmpl[W_COUNT];
    std::vector<Op> ops;
    std::vector<Row> rows;
};

static Op weapon_op(uint32_t kind) {
    const uint32_t slot = kind == K_SET ? pick(3) : 0;
    const uint32_t r = pick(100);
    if (r < 24) return {OP_ADVANCE, pick(3) == 0 ? 300 + pick(200) : pick(50), 0, 0};
    if (r < 34) return {OP_LOAD_NOW, slot, 0, 0};
    if (r < 40) return {OP_RELOAD, slot, 0, 0};
    if (r < 58) return {OP_FIRE_POS, slot, bits(coord()), bits(coord())};
    if (r < 66) return {OP_FIRE_OBJ, slot, 0, 0};
    if (r < 76) return {OP_FORCE, slot, bits(coord()), bits(coord())};
    if (r < 81) return {OP_STATUS, slot, 0, 0};
    if (r < 87) {
        return {OP_CLIP_PCT, slot, bits(static_cast<Real>(static_cast<int>(pick(180)) - 20) / 100.0f), pick(2)};
    }
    if (r < 94) return {OP_BONUS, pick(1u << kConditionCount), 0, 0};
    return {OP_CONTAIN, pick(2), pick(1u << kConditionCount), 0};
}

static void gen(Vec& v, uint32_t index) {
    v.kind = index % K_COUNT;
    v.shared = pick(2);
    v.variant = pick(6);
    const uint32_t scale = pick(4);
    v.frame = scale == 0 ? 1 + pick(100)
            : scale == 1 ? 100 + pick(100000)
            : scale == 2 ? (1u << 24) + pick(1u << 20)
                         : (1u << 28) + pick(1u << 30);
    v.seed = next_u32() & 0x7fffffff;
    for (int i = 0; i < 3; ++i) v.barrels[i] = pick(4);
    for (int i = 0; i < 3; ++i) { v.pos[i] = bits(coord()); v.victim[i] = bits(coord()); }
    for (int i = 0; i < W_COUNT; ++i) v.tmpl[i] = random_tmpl(i <= W_SET2);

    const uint32_t n = 8 + pick(8);
    for (uint32_t k = 0; k < n; ++k) {
        if (v.kind == K_STATE || v.kind == K_SET) {
            v.ops.push_back(weapon_op(v.kind));
            continue;
        }
        const uint32_t r = pick(100);
        if (r < 25) v.ops.push_back({OP_ADVANCE, pick(4) == 0 ? 200 + pick(200) : pick(30), 0, 0});
        else if (r < 33) v.ops.push_back({OP_BONUS, pick(1u << kConditionCount), 0, 0});
        else if (r < 39) v.ops.push_back({OP_CONTAIN, pick(2), pick(1u << kConditionCount), 0});
        else if (v.kind == K_TEMP) {
            if (r < 70) v.ops.push_back({OP_TEMP_POS, pick(W_COUNT + 1), bits(coord()), bits(coord())});
            else if (r < 90) v.ops.push_back({OP_TEMP_OBJ, pick(W_COUNT + 1), 0, 0});
            else v.ops.push_back({OP_FIRE_POS, 0, bits(coord()), bits(coord())});
        } else if (v.kind == K_UPDATE) {
            if (r < 80) v.ops.push_back({OP_STEP, 1 + pick(20), 0, 0});
            else if (r < 92) v.ops.push_back({OP_FIRE_POS, pick(3), bits(coord()), bits(coord())});
            else v.ops.push_back({OP_BUILDING, pick(2), 0, 0});
        } else if (v.kind == K_DEAD) {
            if (r < 55) v.ops.push_back({OP_KILL, 0, 0, 0});
            else v.ops.push_back({OP_FIRE_POS, pick(3), bits(coord()), bits(coord())});
        } else if (v.kind == K_DAMAGED) {
            if (r < 65) v.ops.push_back({OP_STEP, 1 + pick(40), 0, 0});
            else if (r < 90) v.ops.push_back({OP_DAMAGE, bits(1.0f + static_cast<Real>(pick(20))), 0, 0});
            else v.ops.push_back({OP_FIRE_POS, pick(3), bits(coord()), bits(coord())});
        } else {
            if (r < 85) v.ops.push_back({OP_COLLIDE, 0, 0, 0});
            else v.ops.push_back({OP_FIRE_POS, pick(3), bits(coord()), bits(coord())});
        }
    }
}

static Snap snap(Snapshot* s) {
    Snap out;
    out.xfer = oracle_wp51::World::snapshot_save_hex(s);
    out.crc = oracle_wp51::World::snapshot_crc(s);
    return out;
}

static void run(oracle_wp51::World& w, Vec& v) {
    for (int i = 0; i < W_COUNT; ++i) apply_tmpl(w, i, v.tmpl[i]);
    for (int i = 0; i < 3; ++i) g_barrels[i] = static_cast<Int>(v.barrels[i]);

    const std::string sh = std::to_string(v.shared);
    std::string tname = "WF_Plain" + sh;
    if (v.kind == K_UPDATE) tname = "WF_Update" + std::to_string(v.variant) + "_" + sh;
    if (v.kind == K_DEAD) tname = "WF_Dead" + sh;
    if (v.kind == K_DAMAGED) tname = "WF_Damaged" + sh;
    if (v.kind == K_COLLIDE) tname = "WF_Collide" + sh;

    const Coord3D at = {real_of(v.pos[0]), real_of(v.pos[1]), real_of(v.pos[2])};
    const Coord3D there = {real_of(v.victim[0]), real_of(v.victim[1]), real_of(v.victim[2])};
    Object* obj = w.spawn_object(tname.c_str(), w.team_a, at);
    Object* victim = w.spawn_object("WF_Plain0", w.team_b, there);
    Object* box = w.spawn_object("WF_Box", w.team_a, at);
    if (!obj || !victim || !box) { std::fprintf(stderr, "spawn failed\n"); std::exit(1); }

    BehaviorModule* module = nullptr;
    if (v.kind >= K_UPDATE) {
        const char* names[] = {"FireWeaponUpdate", "FireWeaponWhenDeadBehavior",
                               "FireWeaponWhenDamagedBehavior", "FireWeaponCollide"};
        const NameKeyType key = TheNameKeyGenerator->nameToKey(names[v.kind - K_UPDATE]);
        for (BehaviorModule** m = obj->getBehaviorModules(); *m; ++m) {
            if ((*m)->getModuleNameKey() == key) module = *m;
        }
        if (!module) { std::fprintf(stderr, "missing module\n"); std::exit(1); }
    }

    Weapon* standalone = nullptr;
    if (v.kind == K_STATE) {
        standalone = TheWeaponStore->allocateNewWeapon(
            TheWeaponStore->findWeaponTemplate(kWeaponName[W_STATE]), PRIMARY_WEAPON);
    }

    InitRandom(v.seed);
    UnsignedInt frame = v.frame;

    auto record = [&](uint32_t ret) {
        Row row;
        row.ret = ret;
        row.rng = GetGameLogicRandomSeedCRC();
        if (v.kind == K_STATE) row.snaps.push_back(snap(standalone));
        const int set_slots = v.kind == K_SET ? 3 : 1;
        for (int s = 0; s < set_slots; ++s) {
            row.snaps.push_back(snap(obj->getWeaponInWeaponSlot(static_cast<WeaponSlotType>(s))));
        }
        if (module) row.snaps.push_back(snap(module));
        row.shots = g_shots;
        g_shots.clear();
        v.rows.push_back(row);
    };

    g_shots.clear();
    record(0);

    for (const Op& op : v.ops) {
        w.set_frame(frame);
        uint32_t ret = 0;
        auto weapon_at = [&](uint32_t slot) -> Weapon* {
            if (v.kind == K_STATE) return standalone;
            return obj->getWeaponInWeaponSlot(static_cast<WeaponSlotType>(slot));
        };
        const Coord3D target = {real_of(op.b), real_of(op.c), 7.5f};
        switch (op.code) {
        case OP_ADVANCE: frame += op.a; w.set_frame(frame); break;
        case OP_LOAD_NOW: weapon_at(op.a)->loadAmmoNow(obj); break;
        case OP_RELOAD: weapon_at(op.a)->reloadAmmo(obj); break;
        case OP_FIRE_POS: ret = weapon_at(op.a)->fireWeapon(obj, &target) ? 1 : 0; break;
        case OP_FIRE_OBJ: ret = weapon_at(op.a)->fireWeapon(obj, victim) ? 1 : 0; break;
        case OP_FORCE: weapon_at(op.a)->forceFireWeapon(obj, &target); break;
        case OP_STATUS: ret = weapon_at(op.a)->getStatus(); break;
        case OP_CLIP_PCT: weapon_at(op.a)->setClipPercentFull(real_of(op.b), op.c != 0); break;
        case OP_BONUS: obj->*access_member(Wp55BonusTag()) = op.a; break;
        case OP_CONTAIN:
            obj->*access_member(Wp55ContainerTag()) = op.a ? box : nullptr;
            box->*access_member(Wp55BonusTag()) = op.b;
            break;
        case OP_TEMP_POS:
            TheWeaponStore->createAndFireTempWeapon(
                TheWeaponStore->findWeaponTemplate(op.a < W_COUNT ? kWeaponName[op.a] : "WF_NONE"),
                obj, &target);
            break;
        case OP_TEMP_OBJ:
            TheWeaponStore->createAndFireTempWeapon(
                TheWeaponStore->findWeaponTemplate(op.a < W_COUNT ? kWeaponName[op.a] : "WF_NONE"),
                obj, victim);
            break;
        case OP_STEP: {
            frame += op.a;
            w.set_frame(frame);
            UpdateModule* u = static_cast<UpdateModule*>(module);
            const UpdateSleepTime sleep = u->update();
            ret = static_cast<uint32_t>(sleep);
            break;
        }
        case OP_DAMAGE: {
            DamageInfo d;
            d.in.m_damageType = DAMAGE_UNRESISTABLE;
            d.in.m_deathType = DEATH_NORMAL;
            d.in.m_amount = real_of(op.a);
            d.in.m_sourceID = INVALID_ID;
            obj->attemptDamage(&d);
            break;
        }
        case OP_KILL: obj->kill(); break;
        case OP_COLLIDE: {
            const Coord3D zero = {0.0f, 0.0f, 0.0f};
            obj->onCollide(victim, &zero, &zero);
            break;
        }
        case OP_BUILDING:
            if (op.a) obj->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
            else obj->clearStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
            break;
        }
        record(ret);
        if (obj->isDestroyed()) break;
    }
    // Ops after the object was destroyed are not run; drop them from the input.
    v.ops.resize(v.rows.size() - 1);

    deleteInstance(standalone);
    obj->*access_member(Wp55ContainerTag()) = nullptr;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        if (!o->isDestroyed()) TheGameLogic->destroyObject(o);
    }
    w.flush_destroy_list();
}

// ----------------------------------------------------------------------------- output

static void print_list(FILE* f, const std::vector<uint32_t>& xs) {
    for (size_t i = 0; i < xs.size(); ++i) std::fprintf(f, "%s%u", i ? "," : "", xs[i]);
}

static void print_tmpl(FILE* f, const Tmpl& t) {
    std::fprintf(f, "[%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%zu", t.clip, t.reload, t.min_delay,
                 t.max_delay, t.spb, t.reload_type, t.disarm, t.leech, t.scalar, t.suspend, t.pitch,
                 t.scatter.size() / 2);
    for (uint32_t x : t.scatter) std::fprintf(f, ",%u", x);
    std::fprintf(f, ",%zu", t.extra.size());
    for (const BonusLine& b : t.extra) std::fprintf(f, ",%u,%u,%u", b.cond, b.field, b.pct);
    std::fprintf(f, "]");
}

// The C++ constructor's values, in the order the Rust suite compares them.
static std::vector<uint32_t> default_template() {
    const WeaponTemplate* d = TheWeaponStore->newWeaponTemplate("WF_DEFAULTS");
    return {bits(d->m_primaryDamage), bits(d->m_primaryDamageRadius), bits(d->m_secondaryDamage),
            bits(d->m_secondaryDamageRadius), bits(d->m_attackRange), bits(d->m_minimumAttackRange),
            bits(d->m_requestAssistRange), bits(d->m_aimDelta), bits(d->m_scatterRadius),
            bits(d->m_scatterTargetScalar), bits(d->m_shockWaveAmount), bits(d->m_shockWaveRadius),
            bits(d->m_shockWaveTaperOff), (uint32_t)d->m_damageType, (uint32_t)d->m_deathType,
            bits(d->m_weaponSpeed), bits(d->m_minWeaponSpeed), (uint32_t)d->m_isScaleWeaponSpeed,
            bits(d->m_weaponRecoil), bits(d->m_minTargetPitch), bits(d->m_maxTargetPitch),
            bits(d->m_radiusDamageAngle), (uint32_t)d->m_clipSize, (uint32_t)d->m_clipReloadTime,
            (uint32_t)d->m_minDelayBetweenShots, (uint32_t)d->m_maxDelayBetweenShots,
            (uint32_t)d->m_continuousFireOneShotsNeeded, (uint32_t)d->m_continuousFireTwoShotsNeeded,
            (uint32_t)d->m_continuousFireCoastFrames, (uint32_t)d->m_autoReloadWhenIdleFrames,
            (uint32_t)d->m_shotsPerBarrel, (uint32_t)d->m_damageDealtAtSelfPosition,
            (uint32_t)d->m_reloadType, (uint32_t)d->m_prefireType, (uint32_t)d->m_preAttackDelay,
            bits(d->m_continueAttackRange), (uint32_t)d->m_affectsMask, (uint32_t)d->m_collideMask,
            (uint32_t)d->m_antiMask, (uint32_t)d->m_leechRangeWeapon,
            (uint32_t)d->m_capableOfFollowingWaypoint, (uint32_t)d->m_isShowsAmmoPips,
            (uint32_t)d->m_allowAttackGarrisonedBldgs, (uint32_t)d->m_playFXWhenStealthed,
            (uint32_t)d->m_dieOnDetonate, (uint32_t)d->m_suspendFXDelay,
            bits(d->m_infantryInaccuracyDist)};
}

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_wp55_fire <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    oracle_wp51::World w;
    w.load_ini(fixture_ini());
    w.init_with_neutral_player("FactionWF", nullptr);
    TheTerrainLogic = NEW SlopeTerrain;
    TheTerrainLogic->init();
    TheWritableGlobalData->m_unitDamagedThresh = 0.7f;
    TheWritableGlobalData->m_unitReallyDamagedThresh = 0.35f;
    TheWritableGlobalData->m_weaponBonusSet = newInstance(WeaponBonusSet);
    for (const BonusLine& b : kGlobalBonus) {
        w.mem_ini.parse_blocks("WB " + bonus_line(b) + "\n", INI_LOAD_OVERWRITE,
                               [](INI* ini, const char*) {
                                   TheWritableGlobalData->m_weaponBonusSet->parseWeaponBonusSet(ini);
                               });
    }
    const std::vector<uint32_t> defaults = default_template();

    const int kTotal = 1050;
    std::vector<Vec> vecs;
    uint32_t shots = 0;
    for (int i = 0; i < kTotal; ++i) {
        Vec v;
        gen(v, static_cast<uint32_t>(i));
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(v.frame);
        run(w, v);
        for (const Row& r : v.rows) shots += static_cast<uint32_t>(r.shots.size());
        vecs.push_back(std::move(v));
    }

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    std::fprintf(f, "{\n\"packet\": \"WP55_FIRE\",\n\"vector_count\": %d,\n\"shot_count\": %u,\n",
                 kTotal, shots);
    std::fprintf(f, "\"source_sha256\": \"%s\",\n",
                 oracle::sha256_file(
                     "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Weapon.cpp")
                     .c_str());
    std::fprintf(f, "\"default_template\": [");
    print_list(f, defaults);
    std::fprintf(f, "],\n\"global_bonus\": [");
    for (size_t i = 0; i < sizeof(kGlobalBonus) / sizeof(kGlobalBonus[0]); ++i) {
        std::fprintf(f, "%s[%u,%u,%u]", i ? "," : "", kGlobalBonus[i].cond, kGlobalBonus[i].field,
                     kGlobalBonus[i].pct);
    }
    std::fprintf(f, "],\n\"update_variants\": [");
    for (int i = 0; i < 6; ++i) {
        std::fprintf(f, "%s[%u,%u]", i ? "," : "", kUpdFrames[i][0], kUpdFrames[i][1]);
    }
    std::fprintf(f, "],\n\"vectors\": [\n");
    for (int i = 0; i < kTotal; ++i) {
        const Vec& v = vecs[i];
        std::fprintf(f, "{\"kind\":\"%s\",\"head\":[%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u],\"tmpl\":[",
                     kKindName[v.kind], v.kind, v.shared, v.variant, v.frame, v.seed, v.barrels[0],
                     v.barrels[1], v.barrels[2], v.pos[0], v.pos[1], v.pos[2], v.victim[0], v.victim[1]);
        for (int t = 0; t < W_COUNT; ++t) {
            if (t) std::fprintf(f, ",");
            print_tmpl(f, v.tmpl[t]);
        }
        std::fprintf(f, "],\"victim_z\":%u,\"ops\":[", v.victim[2]);
        for (size_t k = 0; k < v.ops.size(); ++k) {
            const Op& o = v.ops[k];
            std::fprintf(f, "%s[%u,%u,%u,%u]", k ? "," : "", o.code, o.a, o.b, o.c);
        }
        std::fprintf(f, "],\"rows\":[");
        for (size_t k = 0; k < v.rows.size(); ++k) {
            const Row& r = v.rows[k];
            std::fprintf(f, "%s{\"r\":[%u,%u],\"s\":[", k ? "," : "", r.ret, r.rng);
            for (size_t s = 0; s < r.snaps.size(); ++s) {
                std::fprintf(f, "%s\"%s\"", s ? "," : "", r.snaps[s].xfer.c_str());
            }
            std::fprintf(f, "],\"c\":[");
            for (size_t s = 0; s < r.snaps.size(); ++s) {
                std::fprintf(f, "%s%u", s ? "," : "", r.snaps[s].crc);
            }
            std::fprintf(f, "],\"f\":[");
            for (size_t s = 0; s < r.shots.size(); ++s) {
                const Shot& h = r.shots[s];
                std::fprintf(f, "%s[%u,%u,%u,%u,%u]", s ? "," : "", h.weapon, h.victim, h.x, h.y, h.z);
            }
            std::fprintf(f, "]}");
        }
        std::fprintf(f, "]}%s\n", i + 1 < kTotal ? "," : "");
    }
    std::fprintf(f, "]\n}\n");
    if (std::fclose(f) != 0 || std::rename(tmp.c_str(), out_path.c_str()) != 0) {
        std::perror("write");
        return 1;
    }
    std::fprintf(stderr, "wrote %d vectors, %u shots\n", kTotal, shots);
    return 0;
}
