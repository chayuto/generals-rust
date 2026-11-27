// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for `StructureCollapseUpdate` (`PKT-376`), run on the
// unmodified reference module in `oracle_wp51::World`:
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_structure_collapse_update.cpp -o /tmp/gen_scu_o0 --opt O0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_structure_collapse_update.cpp -o /tmp/gen_scu_o2 --opt O2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_structure_collapse_update.cpp -o /tmp/gen_scu_ub --opt O2 --sanitize
//   /tmp/gen_scu_o0 /tmp/scu_o0.json && /tmp/gen_scu_o2 /tmp/scu_o2.json && /tmp/gen_scu_ub /tmp/scu_ub.json
//   cmp /tmp/scu_o0.json /tmp/scu_o2.json && cmp /tmp/scu_o0.json /tmp/scu_ub.json
//   cp /tmp/scu_o0.json docs/golden_vectors/StructureCollapseUpdate.json
//
// Observed through: `XferSave` hex & CRC, decoded module state, `XferCRC`,
// `update()` sleep return, `nextCallFrame`, object transform matrix,
// `isEffectivelyDead()`, `MODELCONDITION_RUBBLE`, `MODELCONDITION_POST_COLLAPSE`,
// `FXList::doFXPos` calls, `ObjectCreationList::create` calls, optional
// `BoneFXUpdate` save hex (verifying `stopAllBoneFX`), logic RNG seed CRC,
// and client RNG probe `GameClientRandomValue(0, 1000000)`.
//
// Not covered: `Drawable::setInstanceMatrix` visual translation and
// `BodyModuleInterface::updateBodyParticleSystems` (client-only).

#include "oracle_wp51_world.h"

#include "Common/RandomValue.h"
#include "Common/ThingFactory.h"
#include "GameClient/ClientRandomValue.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameLogic/Damage.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/BoneFXUpdate.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/StructureCollapseUpdate.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

static uint32_t g_state = 0x3765c0deu;
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
static Real grid(int lo, int hi) {
    return static_cast<Real>(lo * 64 + static_cast<int>(pick((hi - lo) * 64 + 1))) / 64.0f;
}
static void die(const char* why) {
    std::fprintf(stderr, "%s\n", why);
    std::exit(1);
}

// ---------------------------------------------------------------------------------------------
// Recorders & sloped terrain
// ---------------------------------------------------------------------------------------------
static std::ostringstream g_fx, g_ocl;
static const int kFxLists = 8;
static const char* const kFxNames[kFxLists] = {
    "SC_FX_I0", "SC_FX_I1", "SC_FX_D0", "SC_FX_D1",
    "SC_FX_B0", "SC_FX_B1", "SC_FX_F0", "SC_FX_F1",
};
static const int kOcls = 8;
static const char* const kOclNames[kOcls] = {
    "SC_OCL_I0", "SC_OCL_I1", "SC_OCL_D0", "SC_OCL_D1",
    "SC_OCL_B0", "SC_OCL_B1", "SC_OCL_F0", "SC_OCL_F1",
};

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
    Object* create(const Object*, const Coord3D* primary, const Coord3D* secondary, Real,
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

class SlopeTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) {
            normal->x = 0.0f;
            normal->y = 0.0f;
            normal->z = 1.0f;
        }
        return x * 0.01f + y * 0.02f;
    }
};

// ---------------------------------------------------------------------------------------------
// Fixture INI
// ---------------------------------------------------------------------------------------------
static const int kVariants = 24;

static std::string opt(const char* field, const char* value) {
    return value ? std::string("    ") + field + " = " + value + "\n" : std::string();
}

static std::string fixture_ini() {
    std::string s;
    static const char* const kPhase[4] = {"INITIAL", "DELAY", "BURST", "FINAL"};
    static const char* const kDelay[] = {nullptr, "0", "33", "100", "500", "1000"};
    static const char* const kBurstDelay[] = {nullptr, "0", "33", "67", "150", "300"};
    static const char* const kDamping[] = {nullptr, "0.0", "0.25", "0.5", "0.85", "1.0", "-0.5"};
    static const char* const kShudder[] = {nullptr, "0.0", "0.5", "2.5", "8.0"};
    static const char* const kBigBurst[] = {nullptr, "0", "1", "2", "4"};
    static const char* const kDeathTypes[] = {nullptr, "ALL", "ALL -CRUSHED", "NONE +NORMAL +EXPLODED"};
    static const char* const kVet[] = {nullptr, "ALL", "ALL -REGULAR", "NONE +REGULAR +VETERAN"};
    static const char* const kRequired[] = {nullptr, nullptr, nullptr, "SOLD"};
    static const char* const kExempt[] = {nullptr, nullptr, "SOLD", "UNDER_CONSTRUCTION"};

    for (int i = 0; i < kVariants; ++i) {
        const bool stick_slope = (i == 3 || i == 11);
        const bool with_bone_fx = (i == 3 || i == 7 || i == 15);
        s += "Object SC_V" + std::to_string(i) + "\n  KindOf = PRELOAD STRUCTURE";
        if (stick_slope) s += " STICK_TO_TERRAIN_SLOPE";
        s += "\n  Geometry = ";
        s += (i % 2 == 0) ? "BOX" : "CYLINDER";
        s += "\n  GeometryMajorRadius = " + std::to_string(5 + (i % 7) * 4) + ".0\n";
        s += "  GeometryMinorRadius = " + std::to_string(4 + (i % 5) * 3) + ".0\n";
        if (i == 0) {
            // Exact `m_currentHeight + maxHeight == 0.0` landing check under gravity -1.0.
            s += "  GeometryHeight = 1.0\n";
        } else if (i == 1) {
            s += "  GeometryHeight = 0.0\n";
        } else if (i == 2) {
            s += "  GeometryHeight = 20.0\n  StructureRubbleHeight = 1.0\n";
        } else {
            const int h = 2 + static_cast<int>(pick(60));
            s += "  GeometryHeight = " + std::to_string(h) + ".0\n";
            if (pick(2)) s += "  StructureRubbleHeight = " + std::to_string(1 + pick(5)) + ".0\n";
        }
        s += "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n    InitialHealth = 100.0\n  End\n";
        if (with_bone_fx) {
            s += "  Behavior = BoneFXDamage ModuleTag_BoneDamage\n  End\n"
                 "  Behavior = BoneFXUpdate ModuleTag_BoneFX\n"
                 "    PristineOCL1 = bone:B1 onlyonce:No 33 100 ocl:SC_OCL_I0\n"
                 "    RubbleFXList1 = bone:B1 onlyonce:No 33 100 fxlist:SC_FX_I0\n"
                 "  End\n";
        }
        s += "  Behavior = StructureCollapseUpdate ModuleTag_Collapse\n";
        if (i == 0) {
            s += "    MinCollapseDelay = 0\n    MaxCollapseDelay = 0\n"
                 "    MinBurstDelay = 0\n    MaxBurstDelay = 0\n"
                 "    CollapseDamping = 0.0\n    MaxShudder = 0.0\n    BigBurstFrequency = 0\n";
            for (int p = 0; p < 4; ++p) {
                s += std::string("    FXList = ") + kPhase[p] + " " + kFxNames[p * 2] + "\n";
                s += std::string("    OCL = ") + kPhase[p] + " " + kOclNames[p * 2] + "\n";
            }
        } else if (i == 1) {
            s += "    MinCollapseDelay = 0\n    MaxCollapseDelay = 0\n"
                 "    MinBurstDelay = 100\n    MaxBurstDelay = 100\n"
                 "    CollapseDamping = 0.0\n    MaxShudder = 2.5\n    BigBurstFrequency = 1\n"
                 "    FXList = INITIAL SC_FX_I0\n    FXList = BURST SC_FX_B0\n    FXList = FINAL SC_FX_F0\n"
                 "    OCL = INITIAL SC_OCL_I0\n    OCL = BURST SC_OCL_B0\n    OCL = FINAL SC_OCL_F0\n";
        } else if (i == 2) {
            s += "    MinCollapseDelay = 67\n    MaxCollapseDelay = 67\n"
                 "    MinBurstDelay = 33\n    MaxBurstDelay = 33\n"
                 "    CollapseDamping = 0.25\n    MaxShudder = 1.5\n    BigBurstFrequency = 2\n"
                 "    FXList = INITIAL SC_FX_I0 None\n"
                 "    FXList = DELAY None SC_FX_D0\n"
                 "    FXList = BURST SC_FX_B0 SC_FX_B1\n"
                 "    FXList = FINAL SC_FX_MISSING SC_FX_F1\n"
                 "    OCL = INITIAL SC_OCL_I0 SC_OCL_MISSING\n"
                 "    OCL = DELAY None SC_OCL_D1\n"
                 "    OCL = BURST SC_OCL_B0 None\n"
                 "    OCL = FINAL SC_OCL_F0\n";
        } else if (i == 3) {
            s += "    MinCollapseDelay = 33\n    MaxCollapseDelay = 100\n"
                 "    MinBurstDelay = 33\n    MaxBurstDelay = 67\n"
                 "    CollapseDamping = 0.0\n    MaxShudder = 3.0\n    BigBurstFrequency = 1\n"
                 "    FXList = INITIAL SC_FX_I0 SC_FX_I1\n"
                 "    FXList = FINAL SC_FX_F0\n"
                 "    OCL = BURST SC_OCL_B0\n"
                 "    OCL = FINAL SC_OCL_F0 SC_OCL_F1\n";
        } else if (i == 4) {
            s += "    MinCollapseDelay = 0\n    MaxCollapseDelay = 0\n"
                 "    MinBurstDelay = 33\n    MaxBurstDelay = 33\n"
                 "    CollapseDamping = 0.5\n    MaxShudder = 0.0\n    BigBurstFrequency = 3\n"
                 "    DeathTypes = NONE +NORMAL +BURNED\n"
                 "    VeterancyLevels = NONE +VETERAN\n"
                 "    RequiredStatus = SOLD\n"
                 "    ExemptStatus = UNDER_CONSTRUCTION\n"
                 "    FXList = INITIAL SC_FX_I0\n    OCL = FINAL SC_OCL_F0\n";
        } else {
            s += opt("MinCollapseDelay", choose(kDelay));
            s += opt("MaxCollapseDelay", choose(kDelay));
            s += opt("MinBurstDelay", choose(kBurstDelay));
            s += opt("MaxBurstDelay", choose(kBurstDelay));
            s += opt("CollapseDamping", choose(kDamping));
            s += opt("MaxShudder", choose(kShudder));
            s += opt("BigBurstFrequency", choose(kBigBurst));
            for (int p = 0; p < 4; ++p) {
                if (pick(4) == 0) continue;
                s += std::string("    FXList = ") + kPhase[p];
                const int n = 1 + static_cast<int>(pick(3));
                for (int k = 0; k < n; ++k) {
                    const uint32_t r = pick(6);
                    if (r == 0) s += " None";
                    else if (r == 1) s += " SC_FX_MISSING";
                    else s += std::string(" ") + kFxNames[p * 2 + (r & 1)];
                }
                s += "\n";
            }
            for (int p = 0; p < 4; ++p) {
                if (pick(4) == 0) continue;
                s += std::string("    OCL = ") + kPhase[p];
                const int n = 1 + static_cast<int>(pick(3));
                for (int k = 0; k < n; ++k) {
                    const uint32_t r = pick(6);
                    if (r == 0) s += " None";
                    else if (r == 1) s += " SC_OCL_MISSING";
                    else s += std::string(" ") + kOclNames[p * 2 + (r & 1)];
                }
                s += "\n";
            }
            s += opt("DeathTypes", choose(kDeathTypes));
            s += opt("VeterancyLevels", choose(kVet));
            s += opt("RequiredStatus", choose(kRequired));
            s += opt("ExemptStatus", choose(kExempt));
        }
        s += "  End\nEnd\n";
    }
    return s;
}

// ---------------------------------------------------------------------------------------------
// Helpers
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

static std::string take(std::ostringstream& s) {
    std::string v = s.str();
    s.str("");
    return v.empty() ? std::string("-") : v;
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

// Round-trip `Snapshot::xfer` on a scratch module state without losing `m_indexInLogic`.
static void verify_cpp_xfer_roundtrip(UpdateModule* m, const std::string& hex) {
    std::vector<uint8_t> bytes;
    bytes.reserve(hex.size() / 2);
    for (size_t i = 0; i < hex.size(); i += 2) {
        bytes.push_back(static_cast<uint8_t>(std::stoul(hex.substr(i, 2), nullptr, 16)));
    }
    const Int saved_idx = m->friend_getIndexInLogic();
    oracle_wp51::World::snapshot_load_bytes(m, bytes);
    m->friend_setIndexInLogic(saved_idx);
    const std::string again = oracle_wp51::World::snapshot_save_hex(m);
    if (again != hex) die("C++ xfer roundtrip mismatch");
}

static uint32_t u32_le(const std::string& hex, size_t byte_offset) {
    uint32_t v = 0;
    for (int i = 3; i >= 0; --i) {
        v = (v << 8) | static_cast<uint32_t>(std::stoul(hex.substr((byte_offset + i) * 2, 2), nullptr, 16));
    }
    return v;
}

static std::string row_of(Object* o, StructureCollapseUpdate* m, BoneFXUpdate* bfx, uint32_t ret) {
    uint32_t save_crc = 0;
    const std::string hex = oracle_wp51::World::snapshot_save_hex(m, &save_crc);
    verify_cpp_xfer_roundtrip(m, hex);
    // Decode the trailing 20 bytes (`collapseFrame`, `burstFrame`, `state`, `vel`, `height`)
    // for self-diagnosing Seed-0 failures.
    if (hex.size() < 40) die("unexpected StructureCollapseUpdate save length");
    const size_t tail = hex.size() / 2 - 20;
    const uint32_t c_frame = u32_le(hex, tail);
    const uint32_t b_frame = u32_le(hex, tail + 4);
    const uint32_t c_state = u32_le(hex, tail + 8);
    const uint32_t vel_bits = u32_le(hex, tail + 12);
    const uint32_t height_bits = u32_le(hex, tail + 16);

    uint32_t bfx_crc = 0;
    if (bfx) {
        oracle_wp51::World::snapshot_save_hex(bfx, &bfx_crc);
    }

    std::ostringstream out;
    out << hex << ' ' << save_crc << ' ' << oracle_wp51::World::snapshot_crc(m) << ' '
        << ret << ' ' << m->friend_getNextCallFrame() << ' '
        << c_frame << ',' << b_frame << ',' << c_state << ',' << vel_bits << ',' << height_bits << ' '
        << matrix_of(o) << ' ' << (o->isEffectivelyDead() ? 1 : 0)
        << flag(o, MODELCONDITION_RUBBLE) << flag(o, MODELCONDITION_POST_COLLAPSE) << ' '
        << bfx_crc << ' ' << GetGameLogicRandomSeedCRC() << ' '
        << GameClientRandomValue(0, 1000000) << ' ' << take(g_fx) << ' ' << take(g_ocl);
    return out.str();
}

static void destroy_all(oracle_wp51::World& w) {
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        TheGameLogic->destroyObject(o);
    }
    w.flush_destroy_list();
    ThePartitionManager->update();
}

struct Op {
    uint32_t v[4] = {0, 0, 0, 0};
};

int main(int argc, char** argv) {
    if (argc < 2) die("usage: gen_structure_collapse_update <out.json>");
    oracle_wp51::World w;
    TheTerrainLogic = NEW SlopeTerrain;
    TheTerrainLogic->init();
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
    TheThingFactory->postProcessLoad();
    w.init_with_neutral_player();
    ThePartitionManager->revealMapForPlayerPermanently(1);
    ThePlayerList->setLocalPlayer(ThePlayerList->getNthPlayer(1));

    const std::string tmp_path = std::string(argv[1]) + ".tmp";
    FILE* out = std::fopen(tmp_path.c_str(), "w");
    if (!out) die("cannot open output tmp");

    std::string escaped;
    for (char c : ini) {
        if (c == '\n') escaped += "\\n";
        else escaped += c;
    }

    const int kTotal = 240;
    std::fprintf(out,
                 "{\"packet\":\"PKT-376\",\"module\":\"StructureCollapseUpdate\","
                 "\"source_sha256\":{\"h\":\"21fbe725e7bf4f57c7d327655b75d5dccd7ade9411e36dbb08873fadf196b00a\","
                 "\"cpp\":\"8662d877461d3f485a3cf55d3ddceb32a68fb53fd5c015319760b3cd9dda1cb2\"},"
                 "\"vector_count\":%d,\n\"ini\":\"%s\",\n\"vectors\":[",
                 kTotal, escaped.c_str());

    static const Real kGravities[] = {-1.0f, -1.0f, -0.75f, -1.25f, -0.3f};
    for (int i = 0; i < kTotal; ++i) {
        const uint32_t variant = (i < 5) ? static_cast<uint32_t>(i)
                               : (i == 5) ? 2u
                               : static_cast<uint32_t>(pick(kVariants));
        const uint32_t seed = 1 + pick(0x7fffffffu);
        const UnsignedInt start_frame = 100 + pick(100000);
        const Real gravity = (i <= 5) ? -1.0f : kGravities[pick(5)];
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        InitRandom(seed);
        take(g_fx);
        take(g_ocl);
        std::fprintf(out, "%s\n", i ? "," : "");
        // Pass start_frame by value; write exact start_frame in a separate prefix?
        // Wait: `run_vector` mutates its local `frame`, so capture `start_frame` before running!
        TheWritableGlobalData->m_gravity = gravity;
        const std::string tmpl = "SC_V" + std::to_string(variant);
        Coord3D pos;
        pos.x = grid(300, 700);
        pos.y = grid(300, 700);
        pos.z = (i == 3) ? TheTerrainLogic->getGroundHeight(pos.x, pos.y) : (pick(2) ? 0.0f : grid(0, 20));
        const Real angle = (i == 0) ? 0.0f : grid(-3, 3);
        UnsignedInt frame = start_frame;
        w.set_frame(frame);
        Object* o = w.spawn_object(tmpl.c_str(), w.team_a, pos, angle);
        StructureCollapseUpdate* m = find_behavior<StructureCollapseUpdate>(o);
        if (!m) die("missing StructureCollapseUpdate");
        BoneFXUpdate* bfx = find_behavior<BoneFXUpdate>(o);
        ThePartitionManager->update();

        std::vector<Op> ops;
        std::vector<std::string> rows;
        const std::string spawn = row_of(o, m, bfx, 0);

        auto push_op = [&](const Op& op, uint32_t ret) {
            ops.push_back(op);
            rows.push_back(row_of(o, m, bfx, ret));
        };
        auto apply_op = [&](Op op) {
            uint32_t ret = 0;
            w.set_frame(frame);
            if (op.v[0] == 0) {
                DamageInfo info;
                info.in.m_damageType = static_cast<DamageType>(op.v[1]);
                info.in.m_deathType = static_cast<DeathType>(op.v[2]);
                info.in.m_amount = static_cast<Real>(op.v[3]);
                o->attemptDamage(&info);
            } else if (op.v[0] == 1) {
                for (uint32_t t = 0; t < op.v[1]; ++t) step(w, m, ++frame);
            } else if (op.v[0] == 2) {
                ret = static_cast<uint32_t>(static_cast<UpdateModuleInterface*>(m)->update());
            } else if (op.v[0] == 3) {
                DamageInfo info;
                info.in.m_damageType = static_cast<DamageType>(op.v[1]);
                info.in.m_deathType = static_cast<DeathType>(op.v[2]);
                info.in.m_amount = 100.0f;
                static_cast<DieModuleInterface*>(m)->onDie(&info);
            } else if (op.v[0] == 4) {
                const ObjectStatusMaskType mask =
                    MAKE_OBJECT_STATUS_MASK(static_cast<ObjectStatusTypes>(op.v[1]));
                if (op.v[2]) o->setStatus(mask);
                else o->clearStatus(mask);
            } else if (op.v[0] == 5) {
                o->getExperienceTracker()->setVeterancyLevel(
                    static_cast<VeterancyLevel>(op.v[1]), FALSE);
            } else if (op.v[0] == 6) {
                frame += op.v[1];
                w.set_frame(frame);
                ret = static_cast<uint32_t>(static_cast<UpdateModuleInterface*>(m)->update());
            } else if (op.v[0] == 7 && bfx) {
                step(w, bfx, ++frame);
            }
            push_op(op, ret);
        };

        if (i == 0) {
            apply_op({2, 0, 0, 0});
            apply_op({0, DAMAGE_EXPLOSION, DEATH_NORMAL, 1000});
            apply_op({2, 0, 0, 0});
            apply_op({1, 1, 0, 0});
            apply_op({2, 0, 0, 0});
        } else if (i == 1) {
            apply_op({0, DAMAGE_EXPLOSION, DEATH_NORMAL, 1000});
            apply_op({2, 0, 0, 0});
        } else if (i == 2) {
            apply_op({0, DAMAGE_EXPLOSION, DEATH_NORMAL, 1000});
            apply_op({1, 3, 0, 0});
            apply_op({6, 5, 0, 0});
            apply_op({1, 10, 0, 0});
        } else if (i == 3) {
            apply_op({7, 0, 0, 0});
            apply_op({7, 0, 0, 0});
            apply_op({0, DAMAGE_EXPLOSION, DEATH_NORMAL, 1000});
            apply_op({1, 18, 0, 0});
            apply_op({7, 0, 0, 0});
        } else if (i == 4) {
            apply_op({3, DAMAGE_EXPLOSION, DEATH_CRUSHED, 0});
            apply_op({3, DAMAGE_EXPLOSION, DEATH_NORMAL, 0});
            apply_op({5, LEVEL_VETERAN, 0, 0});
            apply_op({3, DAMAGE_EXPLOSION, DEATH_NORMAL, 0});
            apply_op({4, OBJECT_STATUS_SOLD, 1, 0});
            apply_op({4, OBJECT_STATUS_UNDER_CONSTRUCTION, 1, 0});
            apply_op({3, DAMAGE_EXPLOSION, DEATH_NORMAL, 0});
            apply_op({4, OBJECT_STATUS_UNDER_CONSTRUCTION, 0, 0});
            apply_op({3, DAMAGE_EXPLOSION, DEATH_BURNED, 0});
            apply_op({1, 12, 0, 0});
        } else if (i == 5) {
            apply_op({3, DAMAGE_EXPLOSION, DEATH_NORMAL, 0});
            apply_op({1, 4, 0, 0});
            apply_op({3, DAMAGE_EXPLOSION, DEATH_NORMAL, 0});
            apply_op({1, 8, 0, 0});
        } else {
            static const uint32_t kDamage[] = {
                DAMAGE_EXPLOSION, DAMAGE_CRUSH, DAMAGE_SMALL_ARMS, DAMAGE_UNRESISTABLE, DAMAGE_FLAME,
            };
            static const uint32_t kDeath[] = {
                DEATH_NORMAL, DEATH_CRUSHED, DEATH_BURNED, DEATH_EXPLODED,
            };
            const int n_ops = 5 + static_cast<int>(pick(8));
            for (int k = 0; k < n_ops; ++k) {
                uint32_t code = pick(16);
                if (k == 0 && pick(6) == 0) code = 2;
                else if (k == 0 || (k == 1 && pick(2))) code = pick(3) ? 0 : 3;
                else if (code >= 8) code = 1;
                if (code == 7 && !bfx) code = 1;
                Op op;
                op.v[0] = code;
                if (code == 0) {
                    op.v[1] = kDamage[pick(5)];
                    op.v[2] = kDeath[pick(4)];
                    op.v[3] = pick(4) == 0 ? 40 : 500;
                } else if (code == 1) {
                    op.v[1] = 1 + pick(pick(2) ? 4 : 25);
                } else if (code == 3) {
                    op.v[1] = kDamage[pick(5)];
                    op.v[2] = kDeath[pick(4)];
                } else if (code == 4) {
                    op.v[1] = pick(2) ? OBJECT_STATUS_SOLD : OBJECT_STATUS_UNDER_CONSTRUCTION;
                    op.v[2] = pick(2);
                } else if (code == 5) {
                    op.v[1] = pick(4);
                } else if (code == 6) {
                    op.v[1] = 1 + pick(6);
                }
                apply_op(op);
            }
        }

        std::fprintf(out,
                     "{\"variant\":%u,\"seed\":%u,\"frame\":%u,\"gravity\":%u,\"x\":%u,\"y\":%u,\"z\":%u,\"angle\":%u,\"spawn\":\"%s\",\"ops\":[",
                     variant, seed, start_frame, bits(gravity),
                     bits(pos.x), bits(pos.y), bits(pos.z), bits(angle), spawn.c_str());
        for (size_t k = 0; k < ops.size(); ++k) {
            std::fprintf(out, "%s[%u,%u,%u,%u]", k ? "," : "", ops[k].v[0], ops[k].v[1], ops[k].v[2], ops[k].v[3]);
        }
        std::fprintf(out, "],\"rows\":[");
        for (size_t r = 0; r < rows.size(); ++r) {
            std::fprintf(out, "%s\"%s\"", r ? "," : "", rows[r].c_str());
        }
        std::fprintf(out, "],\"client\":%d}", GameClientRandomValue(0, 1000000));
        destroy_all(w);
    }
    std::fprintf(out, "\n]}\n");
    if (std::fclose(out) != 0) die("close failed");
    if (std::rename(tmp_path.c_str(), argv[1]) != 0) die("rename failed");
    return 0;
}
