// PKT-260 `BridgeBehavior` with the landmark half of `TerrainLogic`'s bridges: the
// reference modules, `addLandmarkBridgeToLogic`, `updateBridgeDamageStates` and the
// pathfind layer, under scripted damage, healing, scaffolding and death.
//
// The Rust side is `crates/zh_logic/tests/golden_bridge_behavior.rs`: it builds the
// same world from the `ini` string in the output and replays the `case` and `op`
// rows, comparing every `at` row. One row per line, floats as bits.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_bridge_behavior.cpp -o <bin>
//   <bin> docs/golden_vectors/PKT-260.json

#include "oracle_wp51_world.h"

#include "Common/ThingTemplate.h"
#include "GameClient/FXList.h"
#include "GameClient/Line2D.h"
#include "GameClient/TerrainRoads.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/Damage.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/BridgeBehavior.h"
#include "GameLogic/Module/BridgeScaffoldBehavior.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/TerrainLogic.h"

#include <cmath>
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

static std::vector<std::string> g_rows;

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }
static std::string f32(Real v) {
    char buf[12];
    std::snprintf(buf, sizeof(buf), "%08x", bits(v));
    return buf;
}
static std::string num(long long v) { return std::to_string(v); }

struct Fnv {
    uint32_t h = 0x811c9dc5u;
    void add(uint32_t v) {
        for (int i = 0; i < 4; ++i) {
            h ^= (v >> (8 * i)) & 0xffu;
            h *= 0x01000193u;
        }
    }
    void real(Real v) { add(bits(v)); }
    void text(const std::string& s) {
        for (unsigned char c : s) { h ^= c; h *= 0x01000193u; }
    }
    std::string hex() const {
        char buf[12];
        std::snprintf(buf, sizeof(buf), "%08x", h);
        return buf;
    }
};

// Harness-local input generator, independent of the engine PRNG streams.
static uint32_t g_state = 0x260B21DEu;
static uint32_t rnd() {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state;
}
static int below(int n) { return static_cast<int>(rnd() % static_cast<uint32_t>(n)); }
static Real between(Real lo, Real hi) {
    return lo + (hi - lo) * (static_cast<Real>(rnd() % 100000u) / 100000.0f);
}

// ---- effect logs --------------------------------------------------------------

// Effects and creations apart: the Rust sim records them in two lists.
static std::string g_fx;
static std::string g_ocl;

class OracleNugget : public FXNugget {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(OracleNugget, "OracleNugget", 16, 16)
public:
    OracleNugget(const char* name) : m_name(name) {}
    virtual void doFXPos(const Coord3D* primary, const Matrix3D*, const Real,
                         const Coord3D*, const Real) const override {
        g_fx += std::string(m_name) + ":" + f32(primary->x) + f32(primary->y) + f32(primary->z) + ";";
    }
private:
    const char* m_name;
};
EMPTY_DTOR(OracleNugget)

class OracleCreation : public ObjectCreationNugget {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(OracleCreation, "OracleCreation", 16, 16)
public:
    OracleCreation(const char* name) : m_name(name) {}
    virtual Object* create(const Object* owner, const Coord3D* primary, const Coord3D*, Real,
                           UnsignedInt) const override {
        log(primary ? primary : owner->getPosition());
        return nullptr;
    }
    virtual Object* create(const Object* primary, const Object*, UnsignedInt) const override {
        log(primary->getPosition());
        return nullptr;
    }
private:
    void log(const Coord3D* p) const {
        g_ocl += std::string(m_name) + ":" + f32(p->x) + f32(p->y) + f32(p->z) + ";";
    }
    const char* m_name;
};
EMPTY_DTOR(OracleCreation)

// ---- terrain ------------------------------------------------------------------

// Flat ground at zero. Layer heights follow `W3DTerrainLogic::getLayerHeight`.
class FlatTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    void clear_bridges() { deleteBridges(); }
    Real getLayerHeight(Real x, Real y, PathfindLayerEnum layer, Coord3D* normal = nullptr,
                        Bool clip = true) const override {
        Real height = getGroundHeight(x, y, normal);
        if (layer == LAYER_GROUND) return height;
        Coord3D loc;
        loc.x = x;
        loc.y = y;
        loc.z = height;
        if (Bridge* bridge = findBridgeLayerAt(&loc, layer, clip)) {
            const Real deck = bridge->getBridgeHeight(&loc, normal);
            if (deck > height) height = deck;
        }
        return height;
    }
};

static FlatTerrain* g_terrain = nullptr;

// ---- fixtures -----------------------------------------------------------------

static const char* kBridges[4] = {"BB_Full", "BB_Gap", "BB_NoRoad", "BB_Quiet"};
// A tower of no bridge among them: the fall on the die passes bridge parts by.
static const char* kUnits[4] = {"BB_Walker", "BB_Roller", "BB_Flyer", "BB_TowerA"};

static std::string body(const char* type, const char* health) {
    return std::string("  Body = ") + type + " ModuleTag_Body\n    MaxHealth = " + health +
           "\n    InitialHealth = " + health + "\n  End\n";
}

static std::string active_body(const char* health) { return body("ActiveBody", health); }

static std::string fixture_ini() {
    std::string s;
    const char* fx[3] = {"FX_BB_Die", "FX_BB_Damage", "FX_BB_Repair"};
    const char* ocl[3] = {"OCL_BB_Die", "OCL_BB_Damage", "OCL_BB_Repair"};
    for (int i = 0; i < 3; ++i) s += std::string("FXList ") + fx[i] + "\nEnd\n";
    for (int i = 0; i < 3; ++i) s += std::string("ObjectCreationList ") + ocl[i] + "\nEnd\n";

    // Definitions: all four towers; one with a missing tower template; no effects.
    for (int variant = 0; variant < 3; ++variant) {
        const char* name = variant == 2 ? kBridges[3] : kBridges[variant];
        s += std::string("Bridge ") + name + "\n";
        s += "  ScaffoldObjectName = BB_Scaffold\n  ScaffoldSupportObjectName = BB_Support\n";
        s += std::string("  TowerObjectNameFromLeft = ") + (variant == 2 ? "BB_Missing" : "BB_TowerA") +
             "\n" +
             "  TowerObjectNameFromRight = " + (variant == 1 ? "BB_Missing" : "BB_TowerB") + "\n" +
             "  TowerObjectNameToLeft = BB_TowerC\n  TowerObjectNameToRight = " +
             (variant == 1 ? "BB_TowerA" : "BB_TowerB") + "\n";
        if (variant != 2) {
            s += "  TransitionEffectsHeight = 9.5\n  NumFXPerType = 2\n";
            const char* states[4] = {"PRISTINE", "DAMAGED", "REALLYDAMAGED", "RUBBLE"};
            for (int st = 0; st < 4; ++st) {
                if (st > 0) {
                    s += std::string("  TransitionToFX = Transition:Damage ToState:") + states[st] +
                         " EffectNum:1 FX:FX_BB_Damage\n";
                    s += std::string("  TransitionToOCL = Transition:Damage ToState:") + states[st] +
                         " EffectNum:" + (st == 3 ? "1" : "3") + " OCL:OCL_BB_Damage\n";
                }
                if (st < 3) {
                    s += std::string("  TransitionToFX = Transition:Repair ToState:") + states[st] +
                         " EffectNum:2 FX:FX_BB_Repair\n";
                    s += std::string("  TransitionToOCL = Transition:Repair ToState:") + states[st] +
                         " EffectNum:2 OCL:" + (st == 1 ? "OCL_BB_Absent" : "OCL_BB_Repair") + "\n";
                }
            }
        }
        s += "End\n";
    }

    for (int i = 0; i < 4; ++i) {
        s += std::string("Object ") + kBridges[i] + "\n  KindOf = PRELOAD IMMOBILE BRIDGE\n";
        // Retail landmark bridges mostly have a `StructureBody`.
        s += body(i == 1 ? "StructureBody" : "ActiveBody", "1000.0");
        s += "  Behavior = BridgeBehavior ModuleTag_Bridge\n"
             "    LateralScaffoldSpeed = 100.0\n    VerticalScaffoldSpeed = 70.0\n"
             "    BridgeDieFX = FX:FX_BB_Die Delay:0\n"
             "    BridgeDieFX = FX:FX_BB_Die Delay:60\n"
             "    BridgeDieOCL = OCL:OCL_BB_Die Delay:30\n"
             "    BridgeDieOCL = OCL:OCL_BB_Die Delay:60 Bone:ParentObject\n"
             "  End\n";
        s += "  Geometry = BOX\n  GeometryMajorRadius = " + std::string(i == 1 ? "47.5" : "62.0") +
             "\n  GeometryMinorRadius = " + (i == 3 ? "31.0" : "22.5") +
             "\n  GeometryHeight = 10.0\n  GeometryIsSmall = No\nEnd\n";
    }
    const char* towers[3] = {"BB_TowerA", "BB_TowerB", "BB_TowerC"};
    const char* tower_health[3] = {"200.0", "350.0", "90.0"};
    const char* tower_radius[3] = {"4.0", "7.5", "2.25"};
    for (int i = 0; i < 3; ++i) {
        s += std::string("Object ") + towers[i] + "\n  KindOf = PRELOAD IMMOBILE BRIDGE_TOWER\n";
        // The retail towers have an `ImmortalBody`.
        s += body(i == 2 ? "ImmortalBody" : "ActiveBody", tower_health[i]);
        s += "  Behavior = BridgeTowerBehavior ModuleTag_Tower\n  End\n";
        s += std::string("  Geometry = CYLINDER\n  GeometryMajorRadius = ") + tower_radius[i] +
             "\n  GeometryMinorRadius = " + tower_radius[i] + "\n  GeometryHeight = 12.0\nEnd\n";
    }
    const char* pieces[2] = {"BB_Scaffold", "BB_Support"};
    for (int i = 0; i < 2; ++i) {
        s += std::string("Object ") + pieces[i] + "\n  KindOf = PRELOAD IMMOBILE\n";
        s += "  Behavior = BridgeScaffoldBehavior ModuleTag_Scaffold\n  End\n";
        s += i == 0 ? "  Geometry = BOX\n  GeometryMajorRadius = 13.0\n  GeometryMinorRadius = 9.0\n"
                      "  GeometryHeight = 8.0\nEnd\n"
                    : "  Geometry = SPHERE\n  GeometryMajorRadius = 3.25\nEnd\n";
    }
    s += std::string("Object BB_Walker\n  KindOf = PRELOAD INFANTRY\n") + active_body("120.0") +
         "  Geometry = CYLINDER\n  GeometryMajorRadius = 4.0\n  GeometryHeight = 10.0\nEnd\n";
    s += std::string("Object BB_Roller\n  KindOf = PRELOAD VEHICLE\n") + active_body("300.0") +
         "  Behavior = PhysicsBehavior ModuleTag_Physics\n  End\n"
         "  Geometry = BOX\n  GeometryMajorRadius = 8.0\n  GeometryMinorRadius = 5.0\n"
         "  GeometryHeight = 8.0\nEnd\n";
    s += std::string("Object BB_Flyer\n  KindOf = PRELOAD AIRCRAFT\n") + active_body("80.0") +
         "  Geometry = CYLINDER\n  GeometryMajorRadius = 6.0\n  GeometryHeight = 5.0\nEnd\n";
    return s;
}

// ---- state --------------------------------------------------------------------

template <typename T>
static T* find_behavior(Object* obj) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (T* typed = dynamic_cast<T*>(*m)) return typed;
    }
    return nullptr;
}

struct Case {
    Object* bridge = nullptr;
    std::vector<ObjectID> units;
};

static std::string body_row(Object* o) {
    if (!o) return "-";
    return f32(o->getBodyModule()->getHealth()) + ":" + num(o->getBodyModule()->getDamageState()) +
           ":" + num(o->isEffectivelyDead());
}

static std::string deck_digest(Bridge* b) {
    Fnv d;
    const Region2D* bounds = b->getBounds();
    const int x0 = static_cast<int>(bounds->lo.x / 10.0f) - 2;
    const int y0 = static_cast<int>(bounds->lo.y / 10.0f) - 2;
    const int x1 = static_cast<int>(bounds->hi.x / 10.0f) + 3;
    const int y1 = static_cast<int>(bounds->hi.y / 10.0f) + 3;
    for (int y = y0; y < y1; ++y) {
        for (int x = x0; x < x1; ++x) {
            for (PathfindLayerEnum layer : {b->getLayer(), LAYER_GROUND}) {
                PathfindCell* c = TheAI->pathfinder()->getCell(layer, x, y);
                d.add(c ? static_cast<uint32_t>(c->getType()) * 100u +
                              static_cast<uint32_t>(c->getConnectLayer())
                        : 9999u);
            }
        }
    }
    return d.hex();
}

static std::string state_row(Case& c) {
    BridgeBehavior* module = find_behavior<BridgeBehavior>(c.bridge);
    std::string r = "at rng=";
    char buf[12];
    std::snprintf(buf, sizeof(buf), "%08x", GetGameLogicRandomSeedCRC());
    r += buf;
    r += " bridge=" + body_row(c.bridge) + " towers=";
    BridgeBehaviorInterface* itf = module;
    for (int i = 0; i < BRIDGE_MAX_TOWERS; ++i) {
        const ObjectID id = itf->getTowerID(static_cast<BridgeTowerType>(i));
        r += num(id) + "/" + body_row(TheGameLogic->findObjectByID(id)) + ",";
    }
    Bridge* b = TheTerrainLogic->getFirstBridge();
    BridgeInfo info;
    b->getBridgeInfo(&info);
    r += " record=" + num(info.curDamageState) + ":" + num(info.damageStateChanged) + ":" +
         num(info.bridgeObjectID) + ":" + num(b->getLayer()) + " deck=" + deck_digest(b);
    r += " scaffold=" + num(itf->isScaffoldPresent()) + ":" + num(itf->isScaffoldInMotion());
    const std::string save = oracle_wp51::World::snapshot_save_hex(module);
    char crc[12];
    std::snprintf(crc, sizeof(crc), "%08x", oracle_wp51::World::snapshot_crc(module));
    // The update header's bytes are the schedule's; they are part of the save.
    r += " save=" + save + " crc=" + crc;

    Fnv objects;
    int count = 0;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        ++count;
        Fnv one;
        one.add(o->getID());
        one.text(o->getTemplate()->getName().str());
        one.real(o->getPosition()->x);
        one.real(o->getPosition()->y);
        one.real(o->getPosition()->z);
        one.real(o->getOrientation());
        one.add(o->getLayer());
        one.add(o->getBodyModule() ? bits(o->getBodyModule()->getHealth()) : 0u);
        one.add(o->isEffectivelyDead() ? 1u : 0u);
        one.add(o->isDestroyed() ? 1u : 0u);
        one.add(o->getBodyModule() && o->getBodyModule()->isIndestructible() ? 1u : 0u);
        one.add(o->getPhysics() && o->getPhysics()->getAllowToFall() ? 1u : 0u);
        if (BridgeScaffoldBehavior* piece = find_behavior<BridgeScaffoldBehavior>(o)) {
            // Positions, speeds and motion, without the schedule in the update header.
            const std::string bytes = oracle_wp51::World::snapshot_save_hex(piece);
            one.text(bytes.substr(bytes.size() - 2 * (4 + 12 * 4 + 8)));
        }
        // Order independent: the lists of the two engines run in different orders.
        objects.h ^= one.h;
    }
    r += " objects=" + num(count) + ":" + objects.hex();
    for (const std::string* log : {&g_fx, &g_ocl}) {
        Fnv effects;
        effects.text(*log);
        int count = 0;
        for (char ch : *log) count += ch == ';';
        r += std::string(log == &g_fx ? " fx=" : " ocl=") + num(count) + ":" + effects.hex();
    }
    return r;
}

// ---- script -------------------------------------------------------------------

static void spawn_unit(oracle_wp51::World& w, Case& c, int kind, Real along, Real across,
                       Real lift, bool on_deck) {
    Bridge* b = TheTerrainLogic->getFirstBridge();
    BridgeInfo info;
    b->getBridgeInfo(&info);
    Coord3D p;
    p.x = info.fromLeft.x + (info.toLeft.x - info.fromLeft.x) * along +
          (info.fromRight.x - info.fromLeft.x) * across;
    p.y = info.fromLeft.y + (info.toLeft.y - info.fromLeft.y) * along +
          (info.fromRight.y - info.fromLeft.y) * across;
    p.z = info.fromLeft.z + lift;
    Object* o = w.spawn_object(kUnits[kind], w.team_a, p, 0.0f);
    o->setLayer(on_deck ? b->getLayer() : LAYER_GROUND);
    c.units.push_back(o->getID());
    g_rows.push_back("unit " + num(kind) + " " + f32(p.x) + " " + f32(p.y) + " " + f32(p.z) + " " +
                     num(on_deck));
}

static void damage(Object* target, DamageType type, Real amount, ObjectID source) {
    DamageInfo info;
    info.in.m_damageType = type;
    info.in.m_deathType = DEATH_NORMAL;
    info.in.m_sourceID = source;
    info.in.m_amount = amount;
    target->attemptDamage(&info);
}

static void run_case(oracle_wp51::World& w, int index) {
    const int variant = index % 4;
    const bool indestructible = index % 11 == 7;
    const Real x = between(180.0f, 420.0f);
    const Real y = between(180.0f, 420.0f);
    const Real z = index % 3 == 0 ? 0.0f : between(0.0f, 30.0f);
    const Real angle = index % 7 == 0 ? 0.0f : between(-3.14f, 3.14f);
    const uint32_t seed = rnd() & 0x7fffffff;
    const UnsignedInt start = 100 + below(2000);

    TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
    w.set_frame(start);
    InitRandom(seed);
    g_fx.clear();
    g_ocl.clear();

    g_rows.push_back("case " + num(index) + " " + kBridges[variant] + " " + f32(x) + " " + f32(y) +
                     " " + f32(z) + " " + f32(angle) + " " + num(indestructible) + " " + num(seed) +
                     " " + num(start));
    Case c;
    c.bridge = TheThingFactory->newObject(TheThingFactory->findTemplate(kBridges[variant]), w.team_a);
    Coord3D pos;
    pos.x = x;
    pos.y = y;
    pos.z = z;
    c.bridge->setOrientation(angle);
    c.bridge->setPosition(&pos);
    if (indestructible) c.bridge->getBodyModule()->setIndestructible(TRUE);
    TheTerrainLogic->addLandmarkBridgeToLogic(c.bridge);
    TheAI->pathfinder()->newMap();
    if (indestructible && index % 3 != 0) c.bridge->getBodyModule()->setIndestructible(index % 2 == 0);

    const int units = 3 + below(5);
    for (int i = 0; i < units; ++i) {
        spawn_unit(w, c, below(4), between(-0.15f, 1.15f), between(-0.2f, 1.2f),
                   below(3) == 0 ? between(-4.0f, 12.0f) : 0.0f, below(4) != 0);
    }
    ThePartitionManager->update();
    g_rows.push_back(state_row(c));

    UnsignedInt frame = start;
    BridgeBehavior* module = find_behavior<BridgeBehavior>(c.bridge);
    BridgeBehaviorInterface* itf = module;
    UpdateModuleInterface* update = static_cast<UpdateModule*>(module)->getUpdate();
    for (int step = 0; step < 16; ++step) {
        int code = below(10);
        if (index % 5 == 0 && step == 6) code = 7;
        const int which = below(BRIDGE_MAX_TOWERS);
        const int source = below(4);  // none, a tower, a unit, the bridge
        const Real amount = below(5) == 0 ? between(900.0f, 3000.0f) : between(1.0f, 420.0f);
        const DamageType type = below(4) == 0 ? DAMAGE_UNRESISTABLE : DAMAGE_EXPLOSION;
        Object* tower = TheGameLogic->findObjectByID(itf->getTowerID(static_cast<BridgeTowerType>(which)));
        ObjectID from = INVALID_ID;
        if (source == 1) from = itf->getTowerID(static_cast<BridgeTowerType>((which + 1) % 4));
        if (source == 2 && !c.units.empty()) from = c.units.front();
        if (source == 3) from = c.bridge->getID();
        g_rows.push_back("op " + num(code) + " " + num(which) + " " + num(source) + " " + f32(amount) +
                         " " + num(type == DAMAGE_UNRESISTABLE));
        switch (code) {
            case 0:
                damage(c.bridge, type, amount, from);
                break;
            case 1:
                c.bridge->attemptHealing(amount, TheGameLogic->findObjectByID(from));
                break;
            case 2:
                if (tower) damage(tower, type, amount * 0.4f, from);
                break;
            case 3:
                if (tower) tower->attemptHealing(amount * 0.4f, TheGameLogic->findObjectByID(from));
                break;
            case 4:
                // The reference does not check for a missing definition.
                if (variant != 2) itf->createScaffolding();
                break;
            case 5:
                itf->removeScaffolding();
                break;
            case 6:
            case 9:
                for (int i = 0; i < 1 + source; ++i) {
                    w.set_frame(++frame);
                    update->update();
                }
                break;
            case 7:
                c.bridge->kill();
                break;
            case 8:
                if (tower) tower->kill();
                break;
        }
        ThePartitionManager->update();
        g_rows.push_back(state_row(c));
    }

    // A layer keeps its destroyed flag across a pathfinder reset.
    TheAI->pathfinder()->changeBridgeState(TheTerrainLogic->getFirstBridge()->getLayer(), TRUE);
    g_terrain->clear_bridges();
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        if (!o->isDestroyed()) TheGameLogic->destroyObject(o);
    }
    w.flush_destroy_list();
    ThePartitionManager->update();
    TheAI->pathfinder()->reset();
}

// `PointInsideArea2D` over quadrilaterals, with points on and near their edges.
static void area_rows(int count) {
    for (int n = 0; n < count; ++n) {
        Coord3D area[4];
        const bool lattice = n % 2 == 0;
        for (Coord3D& a : area) {
            a.x = lattice ? static_cast<Real>(below(5) * 10) : between(0.0f, 100.0f);
            a.y = lattice ? static_cast<Real>(below(5) * 10) : between(0.0f, 100.0f);
            a.z = 0.0f;
        }
        Coord3D p;
        p.x = lattice ? static_cast<Real>(below(9) * 5) : between(-20.0f, 120.0f);
        p.y = lattice ? static_cast<Real>(below(9) * 5) : between(-20.0f, 120.0f);
        p.z = 0.0f;
        if (n % 10 == 1) p = area[below(4)];
        std::string r = "area " + f32(p.x) + " " + f32(p.y);
        for (const Coord3D& a : area) r += " " + f32(a.x) + " " + f32(a.y);
        r += " " + num(PointInsideArea2D(&p, area, 4));
        g_rows.push_back(r);
    }
}

static std::string escaped(const std::string& s) {
    std::string out;
    for (char c : s) {
        if (c == '\n') out += "\\n";
        else out += c;
    }
    return out;
}

int main(int argc, char** argv) {
    if (argc < 2) return 2;
    oracle_wp51::World w;
    g_terrain = new FlatTerrain;
    g_terrain->extent.hi.x = 600.0f;
    g_terrain->extent.hi.y = 600.0f;
    TheTerrainLogic = g_terrain;
    const std::string ini = fixture_ini();
    w.load_ini(ini);
    w.init_with_neutral_player("FactionBB", nullptr);
    TheTerrainLogic = g_terrain;
    TheWritableGlobalData->m_unitDamagedThresh = 0.7f;
    TheWritableGlobalData->m_unitReallyDamagedThresh = 0.35f;
    for (const char* name : {"FX_BB_Die", "FX_BB_Damage", "FX_BB_Repair"}) {
        FXList* fx = const_cast<FXList*>(TheFXListStore->findFXList(name));
        if (!fx) return 1;
        fx->addFXNugget(newInstance(OracleNugget)(name));
    }
    for (const char* name : {"OCL_BB_Die", "OCL_BB_Damage", "OCL_BB_Repair"}) {
        ObjectCreationList* ocl =
            const_cast<ObjectCreationList*>(TheObjectCreationListStore->findObjectCreationList(name));
        if (!ocl) return 1;
        ocl->addObjectCreationNugget(newInstance(OracleCreation)(name));
    }
    // Reveal before making the player local: a local player's reveal needs a display.
    const Int local = w.team_a->getControllingPlayer()->getPlayerIndex();
    ThePartitionManager->revealMapForPlayerPermanently(local);
    ThePlayerList->setLocalPlayer(w.team_a->getControllingPlayer());

    for (int index = 0; index < 120; ++index) run_case(w, index);
    area_rows(600);

    std::FILE* out = std::fopen(argv[1], "wb");
    if (!out) return 1;
    std::fprintf(out, "{\n \"ini\": \"%s\",\n \"rows\": [", escaped(ini).c_str());
    for (size_t r = 0; r < g_rows.size(); ++r) {
        std::fprintf(out, "%s\n  \"%s\"", r ? "," : "", g_rows[r].c_str());
    }
    std::fputs("\n ]\n}\n", out);
    std::fclose(out);
    std::fprintf(stderr, "%zu rows\n", g_rows.size());
    return 0;
}
