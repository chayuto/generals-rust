// PKT-388 WeaponBonusUpdate.
//
// Build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp55_weapon_bonus.cpp --opt O0 -o <dir>/gen_pkt388_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp55_weapon_bonus.cpp --opt O2 -o <dir>/gen_pkt388_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp55_weapon_bonus.cpp --opt O2 --sanitize -o <dir>/gen_pkt388_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_pkt388_o2 docs/golden_vectors/PKT-388.json
//
// Real module, Objects, containers and PartitionManager. Each vector is a script of operations;
// after each one a row records the tower modules, every object's weapon bonus flags and bonus
// helper, and the layout of the whole sleepy update heap. The helpers are woken in scan order, so
// the layout of the heap is where that order shows.

#include "oracle_wp51_world.h"

#include "Common/ThingTemplate.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/TempWeaponBonusHelper.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/WeaponBonusUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

#include <cmath>
#include <cstdio>
#include <cstring>
#include <map>
#include <sstream>
#include <string>
#include <vector>

struct SleepyTag { using type = std::vector<UpdateModulePtr> GameLogic::*; };
template struct MemberAccess<SleepyTag, &GameLogic::m_sleepyUpdates>;
std::vector<UpdateModulePtr> GameLogic::* access_member(SleepyTag);

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }
static Real real_of(uint32_t b) { Real v; std::memcpy(&v, &b, 4); return v; }

static uint32_t g_state = 0x51C9A3F7u;
static uint32_t next_u32() {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state;
}
static uint32_t pick(uint32_t n) { return next_u32() % n; }
static Real unit_real() { return static_cast<Real>(next_u32() % 100000u) / 100000.0f; }

struct Fnv {
    uint32_t h = 0x811c9dc5u;
    void word(uint32_t w) {
        for (int i = 0; i < 4; ++i) {
            h ^= (w >> (8 * i)) & 0xffu;
            h *= 0x01000193u;
        }
    }
    void text(const std::string& s) {
        for (char c : s) {
            h ^= static_cast<unsigned char>(c);
            h *= 0x01000193u;
        }
    }
};

// ----------------------------------------------------------------------------- fixture

struct Variant {
    const char* required;
    const char* forbidden;
    const char* duration;   // ms, or nullptr to leave the field out
    const char* delay;
    const char* range;
    const char* condition;
    bool armed;
    Real gen_range;
};

static const Variant kVariant[] = {
    {nullptr, nullptr, "600", "300", "50", "ENTHUSIASTIC", true, 50.0f},
    {"INFANTRY", nullptr, "100", nullptr, "80.0", "SUBLIMINAL", true, 80.0f},
    {"VEHICLE", "STRUCTURE", nullptr, "100", "100", "FRENZY_ONE", true, 100.0f},
    {nullptr, "HERO", "1000", "500", "40", "FRENZY_TWO", false, 40.0f},
    {"INFANTRY", "HERO VEHICLE", "1", "33", "123.5", "FRENZY_THREE", true, 123.5f},
    {"STRUCTURE", nullptr, "900", "34", "0", "ENTHUSIASTIC", true, 10.0f},
    {nullptr, nullptr, "250", "200", "200.25", "SUBLIMINAL", false, 200.25f},
    {"INFANTRY HERO", nullptr, "1500", "67", "75", "frenzy_one", true, 75.0f},
    // Parsed only: running it would shift by -1.
    {"ALWAYS_SELECTABLE VEHICLE", "INFANTRY STRUCTURE HERO", "65535", "1001", "-3.75", nullptr, true, 1.0f},
};
static const int kVariants = 9;
static const int kRunnable = 8;

static const char* kUnit[] = {"WB_Inf",  "WB_Veh",    "WB_Unarmed", "WB_Hero",     "WB_Struct",
                              "WB_Trans", "WB_BareTrans", "WB_Garr",  "WB_OpenGarr"};
static const int kUnitKinds = 9;
static bool is_rider_kind(uint32_t k) { return k == 0 || k == 2 || k == 3; }
static bool is_box_kind(uint32_t k) { return k >= 5; }

static std::string fixture_ini() {
    std::ostringstream ss;
    const std::string body =
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n    InitialHealth = 100.0\n  End\n";
    const std::string weapons = "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY Wpn_WB\n  End\n";
    for (int i = 0; i < kVariants; ++i) {
        const Variant& v = kVariant[i];
        ss << "Object WB_Tower" << i << "\n  KindOf = PRELOAD STRUCTURE\n" << body
           << (v.armed ? weapons : "") << "  Behavior = WeaponBonusUpdate ModuleTag_WBU\n";
        if (v.required) ss << "    RequiredAffectKindOf = " << v.required << "\n";
        if (v.forbidden) ss << "    ForbiddenAffectKindOf = " << v.forbidden << "\n";
        if (v.duration) ss << "    BonusDuration = " << v.duration << "\n";
        if (v.delay) ss << "    BonusDelay = " << v.delay << "\n";
        if (v.range) ss << "    BonusRange = " << v.range << "\n";
        if (v.condition) ss << "    BonusConditionType = " << v.condition << "\n";
        ss << "  End\nEnd\n";
    }
    auto object = [&](const char* name, const char* kind, bool armed, const std::string& extra) {
        ss << "Object " << name << "\n  KindOf = PRELOAD " << kind << "\n" << body
           << (armed ? weapons : "") << extra << "End\n";
    };
    const std::string slot = "  TransportSlotCount = 1\n";
    const std::string trans =
        "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 8\n"
        "    AllowInsideKindOf = INFANTRY\n  End\n";
    object("WB_Inf", "INFANTRY", true, slot);
    object("WB_Veh", "VEHICLE", true, "");
    object("WB_Unarmed", "INFANTRY", false, slot);
    object("WB_Hero", "INFANTRY HERO", true, slot);
    object("WB_Struct", "STRUCTURE", true, "");
    object("WB_Trans", "VEHICLE", true, trans);
    object("WB_BareTrans", "VEHICLE", false, trans);
    object("WB_Garr", "STRUCTURE", false,
           "  Behavior = GarrisonContain ModuleTag_Contain\n    ContainMax = 10\n  End\n");
    object("WB_OpenGarr", "STRUCTURE", true,
           "  Behavior = GarrisonContain ModuleTag_Contain\n    ContainMax = 10\n"
           "    IsEnclosingContainer = No\n  End\n");
    return ss.str();
}

// ----------------------------------------------------------------------------- observation

template <typename T>
static T* find_behavior(Object* obj) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (T* typed = dynamic_cast<T*>(*m)) return typed;
    }
    return nullptr;
}

static const char* kTag[] = {
    "ModuleTag_SMCHelper",        "ModuleTag_StatusDamageHelper",  "ModuleTag_SubdualDamageHelper",
    "ModuleTag_RepulsorHelper",   "ModuleTag_DefectionHelper",     "ModuleTag_WeaponStatusHelper",
    "ModuleTag_FiringTrackerHelper", "ModuleTag_TempWeaponBonusHelper", "ModuleTag_WBU",
    "ModuleTag_Contain"};
static const int kTags = 10;

struct Heap { std::vector<uint32_t> code, priority; };

static Heap read_heap(const std::vector<ObjectID>& ids) {
    std::map<const UpdateModule*, uint32_t> known;
    for (size_t k = 0; k < ids.size(); ++k) {
        Object* o = TheGameLogic->findObjectByID(ids[k]);
        if (!o) continue;
        for (BehaviorModule** m = o->getBehaviorModules(); m && *m; ++m) {
            UpdateModuleInterface* ui = (*m)->getUpdate();
            if (!ui) continue;
            const UpdateModule* u = static_cast<const UpdateModule*>(ui);
            const AsciiString tag = TheNameKeyGenerator->keyToName((*m)->getModuleTagNameKey());
            int t = 0;
            while (t < kTags && std::strcmp(tag.str(), kTag[t]) != 0) ++t;
            if (t == kTags) { std::fprintf(stderr, "unknown update tag %s\n", tag.str()); std::exit(1); }
            known[u] = static_cast<uint32_t>(k * 16 + t);
        }
    }
    Heap h;
    for (UpdateModulePtr u : TheGameLogic->*access_member(SleepyTag())) {
        const auto it = known.find(u);
        if (it == known.end()) { std::fprintf(stderr, "stray update module in the heap\n"); std::exit(1); }
        h.code.push_back(it->second);
        h.priority.push_back(u->friend_getPriority());
    }
    return h;
}

// The harness's step: the frame is already set.
static uint32_t step_single_update(UpdateModule* u, UnsignedInt frame) {
    const Object* obj = u->friend_getObject();
    if (!obj || obj->isDestroyed()) return 0xffffffffu;
    if (u->friend_getNextCallFrame() > frame) return 0xffffffffu;
    UpdateSleepTime sleep_len = UPDATE_SLEEP_NONE;
    uint32_t raw = 0xfffffffeu;
    DisabledMaskType dis = obj->getDisabledFlags();
    if (u->getDisabledTypesToProcess().testForAll(dis)) {
        sleep_len = u->update();
        raw = static_cast<uint32_t>(sleep_len);
        if (sleep_len < 1) sleep_len = UPDATE_SLEEP_NONE;
    }
    UnsignedInt next = frame + static_cast<UnsignedInt>(sleep_len);
    if (next > UPDATE_SLEEP_FOREVER) next = UPDATE_SLEEP_FOREVER;
    u->friend_setNextCallFrame(next);
    return raw;
}

struct Unit { uint32_t kind, team, x, y, box; };
struct Op { uint32_t code, a, b, c; };
struct Vec {
    uint32_t frame, towers, variant[2], tower_team, tx, ty;
    std::vector<Unit> units;
    std::vector<Op> ops;
    Heap initial;
    std::vector<std::vector<uint32_t>> rows;
    std::vector<std::string> detail;  // per object, after the last operation
    std::string xfer;
    uint32_t crc = 0;
};

// Offsets that land on, just inside and just outside the range, the map edge, and scattered ones.
// An object is off the map at x <= 0, but only just past the edge is it still in a cell.
static void random_position(const Vec& v, uint32_t* x, uint32_t* y) {
    const Real range = kVariant[v.variant[pick(v.towers)]].gen_range;
    const Real cx = real_of(v.tx);
    Real px = cx, py = real_of(v.ty);
    switch (pick(16)) {
        case 0: px = cx + range; break;
        case 1: px = cx - range; break;
        case 2: px = cx + std::nextafterf(range, 1.0e9f); break;
        case 3: px += 0.6f * range; py += 0.8f * range; break;
        case 4: py += range - 0.001f; break;
        case 5: break;
        case 6:
        case 7:
            if (cx < 20.0f) {
                px = pick(2) ? 0.0f : -0.5f * unit_real();
                py += (unit_real() - 0.5f) * range;
                break;
            }
            [[fallthrough]];
        default:
            px += (unit_real() - 0.5f) * (range * 2.4f + 4.0f);
            py += (unit_real() - 0.5f) * (range * 2.4f + 4.0f);
            break;
    }
    *x = bits(px);
    *y = bits(py);
}

static void generate(Vec& v, int i) {
    v.frame = 100 + pick(5000);
    v.towers = 1 + pick(2);
    v.variant[0] = static_cast<uint32_t>(i % kRunnable);
    v.variant[1] = pick(kRunnable);
    v.tower_team = pick(8) == 0 ? 1 : 0;
    v.tx = bits(pick(4) == 0 ? (pick(5) == 0 ? 0.0f : 4.0f + unit_real() * 10.0f)
                             : 400.0f + unit_real() * 40.0f);
    v.ty = bits(400.0f + unit_real() * 40.0f);

    const uint32_t count = 3 + pick(5);
    std::vector<uint32_t> boxes;
    for (uint32_t k = 0; k < count; ++k) {
        Unit u;
        u.kind = k == 0 && pick(3) ? 5 + pick(4)
                 : boxes.empty()    ? pick(kUnitKinds)
                 : pick(2)          ? (pick(2) ? 0 : 3)
                                    : pick(kUnitKinds);
        u.team = pick(6) == 0 ? 1 : (pick(10) == 0 ? 2 : 0);
        random_position(v, &u.x, &u.y);
        u.box = 0;
        if (is_rider_kind(u.kind) && !boxes.empty() && pick(3) != 0) {
            const uint32_t b = pick(3) ? boxes[0] : boxes[pick(static_cast<uint32_t>(boxes.size()))];
            u.box = b + 1;
            u.team = v.units[b].team;
        }
        if (is_box_kind(u.kind)) boxes.push_back(k);
        v.units.push_back(u);
    }

    v.ops.push_back({0, 1 + pick(3), 0, 0});
    for (int n = 1; n < 10; ++n) {
        Op op = {0, 0, 0, 0};
        const uint32_t r = pick(100);
        const uint32_t k = pick(count);
        if (r < 32) {
            op = {0, pick(4) == 0 ? pick(50) : pick(8), 0, 0};
        } else if (r < 56) {
            op = {1, pick(4) == 0 ? 20 + pick(40) : pick(6), 0, 0};
        } else if (r < 64) {
            op = {2, k, 0, 0};
            random_position(v, &op.b, &op.c);
        } else if (r < 69) {
            op = {3, k, 0, 0};
        } else if (r < 77) {
            if (boxes.empty()) { op = {0, pick(8), 0, 0}; }
            else op = {4, k, boxes[pick(static_cast<uint32_t>(boxes.size()))], 0};
        } else if (r < 82) {
            op = {5, k, 0, 0};
        } else if (r < 87) {
            op = {6, pick(count + 1), pick(3) != 0, 0};  // 0 is the first tower
        } else if (r < 89) {
            op = {7, pick(2), 0, 0};
        } else if (r < 92) {
            op = {8, k, 0, 0};
        } else if (v.towers == 2) {
            op = {9, pick(6), 0, 0};
        } else {
            op = {0, 9 + pick(30), 0, 0};
        }
        v.ops.push_back(op);
    }
}

static Team* team_of(oracle_wp51::World& w, uint32_t idx) {
    return idx == 0 ? w.team_a : (idx == 1 ? w.team_b : w.team_neutral);
}

static void run(oracle_wp51::World& w, Vec& v) {
    std::vector<ObjectID> ids;  // two tower places, then the units
    std::vector<bool> box_unit;
    for (uint32_t t = 0; t < 2; ++t) {
        if (t >= v.towers) { ids.push_back(INVALID_ID); continue; }
        const std::string name = "WB_Tower" + std::to_string(v.variant[t]);
        Object* o = w.spawn_object(name.c_str(), team_of(w, v.tower_team));
        Coord3D pos = {real_of(v.tx), real_of(v.ty), 0.0f};
        o->setPosition(&pos);
        ids.push_back(o->getID());
    }
    for (const Unit& u : v.units) {
        Object* o = w.spawn_object(kUnit[u.kind], team_of(w, u.team));
        Coord3D pos = {real_of(u.x), real_of(u.y), 0.0f};
        o->setPosition(&pos);
        ids.push_back(o->getID());
    }
    for (size_t k = 0; k < v.units.size(); ++k) {
        if (!v.units[k].box) continue;
        Object* box = TheGameLogic->findObjectByID(ids[2 + v.units[k].box - 1]);
        Object* rider = TheGameLogic->findObjectByID(ids[2 + k]);
        box->getContain()->addToContain(rider);
    }

    auto live = [&](size_t k) -> Object* {
        return ids[k] == INVALID_ID ? nullptr : TheGameLogic->findObjectByID(ids[k]);
    };

    v.initial = read_heap(ids);
    UnsignedInt frame = v.frame;
    for (const Op& op : v.ops) {
        ThePartitionManager->update();
        uint32_t sleep = 0xffffffffu;
        Object* u = (op.code == 2 || op.code == 3 || op.code == 4 || op.code == 5 || op.code == 8)
                        ? live(2 + op.a) : nullptr;
        if (op.code == 0 || op.code == 9) {
            frame += op.a;
            w.set_frame(frame);
            Object* t = live(op.code == 0 ? 0 : 1);
            if (t) sleep = step_single_update(find_behavior<WeaponBonusUpdate>(t), frame);
        }
        if (op.code == 1) {
            frame += op.a;
            w.set_frame(frame);
            for (size_t k = 0; k < ids.size(); ++k) {
                Object* o = live(k);
                if (!o) continue;
                if (TempWeaponBonusHelper* h = find_behavior<TempWeaponBonusHelper>(o)) {
                    step_single_update(h, frame);
                }
            }
        }
        if (op.code == 2 && u) {
            Coord3D pos = {real_of(op.b), real_of(op.c), 0.0f};
            u->setPosition(&pos);
        }
        if (op.code == 3 && u && !u->isEffectivelyDead()) u->kill();
        if (op.code == 4 && u && !u->isEffectivelyDead() && is_rider_kind(v.units[op.a].kind) &&
            !u->getContainedBy()) {
            Object* box = live(2 + op.b);
            if (box && !box->isEffectivelyDead() && box->getTeam() == u->getTeam()) {
                box->getContain()->addToContain(u);
            }
        }
        if (op.code == 5 && u && u->getContainedBy()) {
            u->getContainedBy()->getContain()->removeFromContain(u);
        }
        if (op.code == 6) {
            Object* o = live(op.a == 0 ? 0 : 1 + op.a);
            if (o) o->friend_setUndetectedDefector(op.b != 0);
        }
        if (op.code == 7 && live(0)) {
            if (op.a) live(0)->setDisabled(DISABLED_EMP); else live(0)->clearDisabled(DISABLED_EMP);
        }
        if (op.code == 8 && u) {
            TheGameLogic->destroyObject(u);
            w.flush_destroy_list();
        }

        Fnv modules, objects, heap;
        uint32_t next[2] = {0, 0};
        uint32_t bonused = 0;
        for (size_t t = 0; t < 2; ++t) {
            Object* o = live(t);
            if (!o) continue;
            WeaponBonusUpdate* m = find_behavior<WeaponBonusUpdate>(o);
            next[t] = m->friend_getNextCallFrame();
            modules.text(oracle_wp51::World::snapshot_save_hex(m));
        }
        v.detail.clear();
        for (size_t k = 0; k < ids.size(); ++k) {
            Object* o = live(k);
            uint32_t d[4] = {0, 0, 0, 0};  // present, flags, helper next call, helper bytes
            std::string helper_hex = "-";
            if (o) {
                d[0] = 1;
                d[1] = o->getWeaponBonusCondition();
                if (TempWeaponBonusHelper* h = find_behavior<TempWeaponBonusHelper>(o)) {
                    d[2] = h->friend_getNextCallFrame();
                    helper_hex = oracle_wp51::World::snapshot_save_hex(h);
                    Fnv f;
                    f.text(helper_hex);
                    d[3] = f.h;
                }
                bonused += d[1] != 0;
            }
            for (uint32_t x : d) objects.word(x);
            char buf[96];
            std::snprintf(buf, sizeof(buf), "%u %u %u %u", d[0], d[1], d[2], d[3]);
            v.detail.push_back(buf);
        }
        const Heap h = read_heap(ids);
        for (size_t k = 0; k < h.code.size(); ++k) {
            heap.word(h.code[k]);
            heap.word(h.priority[k]);
        }
        v.rows.push_back({sleep, next[0], next[1], modules.h, objects.h, heap.h, bonused});
    }
    if (Object* o = live(0)) {
        WeaponBonusUpdate* m = find_behavior<WeaponBonusUpdate>(o);
        v.xfer = oracle_wp51::World::snapshot_save_hex(m);
        v.crc = oracle_wp51::World::snapshot_crc(m);
    }
}

// ----------------------------------------------------------------------------- output

static std::string join(const std::vector<uint32_t>& v) {
    std::string s;
    for (size_t k = 0; k < v.size(); ++k) s += (k ? " " : "") + std::to_string(v[k]);
    return s;
}

static std::string hex_of(const std::string& text) {
    std::string out;
    char buf[3];
    for (unsigned char c : text) {
        std::snprintf(buf, sizeof(buf), "%02x", c);
        out += buf;
    }
    return out;
}

static std::string mask_bits(const KindOfMaskType& m) {
    std::vector<uint32_t> set;
    for (int i = 0; i < KINDOF_COUNT; ++i) {
        if (m.test(i)) set.push_back(static_cast<uint32_t>(i));
    }
    return join(set);
}

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_wp55_weapon_bonus <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    oracle_wp51::World w;
    const std::string weapon_ini =
        "Weapon Wpn_WB\n  PrimaryDamage = 10.0\n  AttackRange = 100.0\n  DamageType = SMALL_ARMS\nEnd\n";
    const std::string objects_ini = fixture_ini();
    w.load_ini(weapon_ini + objects_ini);
    w.init_with_neutral_player(nullptr, nullptr);

    const int kTotal = 1000;
    std::vector<Vec> vecs;
    for (int i = 0; i < kTotal; ++i) {
        Vec v;
        generate(v, i);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(v.frame);
        InitRandom(1);
        run(w, v);
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
            if (!o->isDestroyed()) TheGameLogic->destroyObject(o);
        }
        w.flush_destroy_list();
        ThePartitionManager->update();
        vecs.push_back(std::move(v));
    }

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    std::fprintf(f, "{\n\"packet\": \"PKT-388\",\n\"vector_count\": %d,\n", kTotal);
    std::fprintf(f, "\"source_sha256\": \"%s\",\n",
                 oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/"
                                     "Object/Update/WeaponBonusUpdate.cpp").c_str());
    std::fprintf(f, "\"ini_hex\": \"%s\",\n", hex_of(objects_ini).c_str());
    std::fprintf(f, "\"parsed\": [\n");
    for (int i = 0; i < kVariants; ++i) {
        const std::string name = "WB_Tower" + std::to_string(i);
        const ModuleInfo& mi = TheThingFactory->findTemplate(name.c_str())->getBehaviorModuleInfo();
        const WeaponBonusUpdateModuleData* d = nullptr;
        for (Int k = 0; k < mi.getCount(); ++k) {
            if (mi.getNthName(k) == "WeaponBonusUpdate") {
                d = static_cast<const WeaponBonusUpdateModuleData*>(mi.getNthData(k));
            }
        }
        if (!d) { std::fprintf(stderr, "no module data in %s\n", name.c_str()); return 1; }
        std::fprintf(f, "{\"variant\":%d,\"required\":\"%s\",\"forbidden\":\"%s\",\"duration\":%u,"
                        "\"delay\":%u,\"range\":%u,\"condition\":%d}%s\n",
                     i, mask_bits(d->m_requiredAffectKindOf).c_str(),
                     mask_bits(d->m_forbiddenAffectKindOf).c_str(), d->m_bonusDuration,
                     d->m_bonusDelay, bits(d->m_bonusRange),
                     static_cast<Int>(d->m_bonusConditionType), i + 1 < kVariants ? "," : "");
    }
    std::fprintf(f, "],\n\"vectors\": [\n");
    for (int i = 0; i < kTotal; ++i) {
        const Vec& v = vecs[i];
        std::vector<uint32_t> head = {v.frame, v.towers, v.variant[0], v.variant[1], v.tower_team,
                                      v.tx, v.ty};
        std::vector<uint32_t> units, ops, rows, init;
        for (const Unit& u : v.units) units.insert(units.end(), {u.kind, u.team, u.x, u.y, u.box});
        for (const Op& o : v.ops) ops.insert(ops.end(), {o.code, o.a, o.b, o.c});
        for (const auto& r : v.rows) rows.insert(rows.end(), r.begin(), r.end());
        Fnv prio;
        for (uint32_t p : v.initial.priority) prio.word(p);
        init = v.initial.code;
        std::fprintf(f, "{\"seed\":%d,\"head\":\"%s\",\"units\":\"%s\",\"ops\":\"%s\",\"heap\":\"%s\","
                        "\"priorities\":%u,\"rows\":\"%s\",\"xfer\":\"%s\",\"crc\":%u",
                     i, join(head).c_str(), join(units).c_str(), join(ops).c_str(), join(init).c_str(),
                     prio.h, join(rows).c_str(), v.xfer.c_str(), v.crc);
        if (i < 8) {
            // Per object after the last operation: present, flags, helper next call, helper bytes.
            std::fprintf(f, ",\"detail\":\"");
            for (size_t k = 0; k < v.detail.size(); ++k) {
                std::fprintf(f, "%s%s", k ? "," : "", v.detail[k].c_str());
            }
            std::fprintf(f, "\"");
        }
        std::fprintf(f, "}%s\n", i + 1 < kTotal ? "," : "");
    }
    std::fprintf(f, "]\n}\n");
    if (std::fclose(f) != 0 || std::rename(tmp.c_str(), out_path.c_str()) != 0) {
        std::perror("write");
        return 1;
    }
    std::fprintf(stderr, "wrote %d vectors\n", kTotal);
    return 0;
}
