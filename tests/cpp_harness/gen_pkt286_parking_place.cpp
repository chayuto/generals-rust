// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for ParkingPlaceBehavior (PKT-286), run on the unmodified reference
// module in a small real world: one airfield or helipad (with the real ProductionUpdate
// in most layouts) and up to six aircraft (jets, plain aircraft, helicopters, one without
// AI) on three players. Layout bones come from a stand-in draw module, because the
// headless drawable has none. `JetAIUpdate` and `AIUpdateInterface` are the real classes
// with `aiDoCommand` replaced by a recorder.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt286_parking_place.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT286_ParkingPlace.json
//
// Not covered (all undefined behaviour or unported in the reference): a field with spaces
// but no runways, `calcPPInfo` or `getRunwayReservation` past the runway list, more than
// four spaces (ProductionUpdate has four doors), a hangar aircraft exiting without a door,
// objects without a drawable, healing entries with an invalid id (only a load makes one),
// and loading a save.
//
// In a row, a space-separated field equal to the previous row's is written as `~`.

#include "oracle_wp51_world.h"

#include "Common/DrawModule.h"
#include "Common/Module.h"
#include "Common/ModuleFactory.h"
#include "Common/Player.h"
#include "Common/Team.h"
#include "GameClient/Drawable.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/JetAIUpdate.h"
#include "GameLogic/Module/ParkingPlaceBehavior.h"
#include "GameLogic/Module/ProductionUpdate.h"
#include "GameLogic/Object.h"

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <sstream>
#include <string>
#include <vector>

struct DrawArrayTag { using type = Module** (Drawable::*)[NUM_DRAWABLE_MODULE_TYPES]; };
template struct MemberAccess<DrawArrayTag, &Drawable::m_modules>;
DrawArrayTag::type access_member(DrawArrayTag);

struct ExposedFactory : public ModuleFactory { using ModuleFactory::addModuleInternal; };

static void die(const char* what) {
    std::fprintf(stderr, "oracle: %s\n", what);
    std::exit(1);
}

static uint32_t bits(float f) {
    uint32_t u;
    std::memcpy(&u, &f, 4);
    return u;
}

static std::string coord(const Coord3D& c) {
    std::ostringstream s;
    s << bits(c.x) << ',' << bits(c.y) << ',' << bits(c.z);
    return s.str();
}

static uint32_t g_state = 0x28600286u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
static float real_in(float lo, float hi) {
    return lo + (hi - lo) * (static_cast<float>(pick(100001)) / 100000.0f);
}

// --- recording AI -----------------------------------------------------------

static std::string g_commands;

static void record(const Object* self, const AICommandParms* parms) {
    std::ostringstream s;
    int code = 9;
    if (parms->m_cmd == AICMD_MOVE_TO_POSITION) code = 1;
    else if (parms->m_cmd == AICMD_FOLLOW_EXITPRODUCTION_PATH) code = 2;
    else if (parms->m_cmd == AICMD_IDLE) code = 3;
    s << '{' << self->getID() << ':' << code << ':' << static_cast<int>(parms->m_cmdSource);
    if (code == 1) s << ':' << coord(parms->m_pos);
    if (code == 2) {
        s << ':' << (parms->m_obj ? parms->m_obj->getID() : INVALID_ID);
        for (const Coord3D& c : parms->m_coords) s << ':' << coord(c);
    }
    if (code == 9) s << ':' << static_cast<int>(parms->m_cmd);
    s << '}';
    g_commands += s.str();
}

class RecAI : public AIUpdateInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(RecAI, "OracleParkRecAI", 64, 64)
public:
    RecAI(Thing* thing, const ModuleData* data) : AIUpdateInterface(thing, data) {}
    void aiDoCommand(const AICommandParms* parms) override { record(getObject(), parms); }
};
RecAI::~RecAI() {}

class RecJet : public JetAIUpdate {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(RecJet, "OracleParkRecJet", 64, 64)
public:
    RecJet(Thing* thing, const ModuleData* data) : JetAIUpdate(thing, data) {}
    void aiDoCommand(const AICommandParms* parms) override { record(getObject(), parms); }
};
RecJet::~RecJet() {}

static Module* make_rec_ai(Thing* t, const ModuleData* d) { return newInstance(RecAI)(t, d); }
static Module* make_rec_jet(Thing* t, const ModuleData* d) { return newInstance(RecJet)(t, d); }

// --- bones ------------------------------------------------------------------

static std::map<std::string, Matrix3D> g_bones;

class OracleDraw : public DrawModule, public ObjectDrawInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(OracleDraw, "OracleParkDraw", 64, 64)
public:
    OracleDraw(Thing* thing, const ModuleData* data) : DrawModule(thing, data) {}

    NameKeyType getModuleNameKey() const override { return NAMEKEY("OracleParkDraw"); }
    void doDrawModule(const Matrix3D*) override {}
    void setShadowsEnabled(Bool) override {}
    void releaseShadows() override {}
    void allocateShadows() override {}
    void setFullyObscuredByShroud(Bool) override {}
    void reactToTransformChange(const Matrix3D*, const Coord3D*, Real) override {}
    void reactToGeometryChange() override {}
    ObjectDrawInterface* getObjectDrawInterface() override { return this; }
    const ObjectDrawInterface* getObjectDrawInterface() const override { return this; }

    Int getPristineBonePositionsForConditionState(const ModelConditionFlags&, const char* name,
                                                  Int start, Coord3D* positions,
                                                  Matrix3D* transforms, Int max) const override {
        if (start != 0 || max < 1) die("unexpected bone query");
        auto it = g_bones.find(name);
        if (it == g_bones.end()) return 0;
        if (positions) {
            const Vector3 t = it->second.Get_Translation();
            positions->x = t.X;
            positions->y = t.Y;
            positions->z = t.Z;
        }
        if (transforms) *transforms = it->second;
        return 1;
    }

    Bool clientOnly_getRenderObjInfo(Coord3D*, Real*, Matrix3D*) const override { return FALSE; }
    Bool clientOnly_getRenderObjBoundBox(OBBoxClass*) const override { return FALSE; }
    Bool clientOnly_getRenderObjBoneTransform(const AsciiString&, Matrix3D*) const override { return FALSE; }
    Int getCurrentBonePositions(const char*, Int, Coord3D*, Matrix3D*, Int) const override { return 0; }
    Bool getCurrentWorldspaceClientBonePositions(const char*, Matrix3D&) const override { return FALSE; }
    Bool getProjectileLaunchOffset(const ModelConditionFlags&, WeaponSlotType, Int, Matrix3D*,
                                   WhichTurretType, Coord3D*, Coord3D*) const override { return FALSE; }
    void updateProjectileClipStatus(UnsignedInt, UnsignedInt, WeaponSlotType) override {}
    void updateDrawModuleSupplyStatus(Int, Int) override {}
    void notifyDrawModuleDependencyCleared() override {}
    void setHidden(Bool) override {}
    void replaceModelConditionState(const ModelConditionFlags&) override {}
    void replaceIndicatorColor(Color) override {}
    Bool handleWeaponFireFX(WeaponSlotType, Int, const FXList*, Real, const Coord3D*, Real) override { return FALSE; }
    Int getBarrelCount(WeaponSlotType) const override { return 0; }
    void setSelectable(Bool) override {}
    void setAnimationLoopDuration(UnsignedInt) override {}
    void setAnimationCompletionTime(UnsignedInt) override {}
    Bool updateBonesForClientParticleSystems() override { return FALSE; }
    void setAnimationFrame(int) override {}
    void setPauseAnimation(Bool) override {}
    void updateSubObjects() override {}
    void showSubObject(const AsciiString&, Bool) override {}

protected:
    void crc(Xfer*) override {}
    void xfer(Xfer*) override {}
    void loadPostProcess() override {}
};
OracleDraw::~OracleDraw() {}

class OracleDrawData : public ModuleData {};

// --- fixture ----------------------------------------------------------------

struct Layout { int rows, cols, runways, hangars; const char* heal; const char* approach; const char* deck; int prod; };
static const Layout kLayouts[] = {
    {2, 2, 1, 0, "5", "50", "0", 1},     {2, 2, 1, 1, "10", "0", "0", 1},
    {2, 1, 1, 0, "0", "20", "15", 0},    {1, 2, 1, 0, "3.3", "35.5", "-7", 1},
    {4, 1, 1, 1, "7", "10", "3", 1},     {0, 0, 0, 0, "12", "0", "0", 0},
    {1, 4, 1, 0, "2", "0", "0", 1},      {3, 1, 1, 0, "1000", "5", "0", 1},
};
static const int kLayoutCount = 8;

struct Craft { const char* kindof; int ai; const char* offset; };
// ai: 0 none, 1 plain, 2 jet
static const Craft kCrafts[] = {
    {"VEHICLE AIRCRAFT", 2, "0"},    {"VEHICLE AIRCRAFT", 2, "-12.5"},
    {"VEHICLE AIRCRAFT", 1, ""},     {"VEHICLE AIRCRAFT PRODUCED_AT_HELIPAD", 1, ""},
    {"VEHICLE AIRCRAFT", 0, ""},     {"VEHICLE AIRCRAFT PRODUCED_AT_HELIPAD", 2, "7.25"},
};
static const int kCraftCount = 6;

static std::string fixture_ini() {
    std::ostringstream s;
    for (int i = 0; i < kLayoutCount; ++i) {
        const Layout& l = kLayouts[i];
        s << "Object P286_Field" << i << "\n  KindOf = PRELOAD STRUCTURE\n"
          << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n";
        if (l.prod)
            s << "  Behavior = ProductionUpdate ModuleTag_Prod\n    NumDoorAnimations = 4\n"
                 "    DoorOpeningTime = 100\n    DoorWaitOpenTime = 200\n    DoorCloseTime = 100\n  End\n";
        s << "  Behavior = ParkingPlaceBehavior ModuleTag_Park\n    NumRows = " << l.rows
          << "\n    NumCols = " << l.cols << "\n    HasRunways = " << (l.runways ? "Yes" : "No")
          << "\n    ParkInHangars = " << (l.hangars ? "Yes" : "No")
          << "\n    HealAmountPerSecond = " << l.heal << "\n    ApproachHeight = " << l.approach
          << "\n    LandingDeckHeightOffset = " << l.deck << "\n  End\nEnd\n";
    }
    for (int i = 0; i < kCraftCount; ++i) {
        const Craft& c = kCrafts[i];
        s << "Object P286_Craft" << i << "\n  KindOf = PRELOAD " << c.kindof << "\n"
          << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n";
        if (c.ai == 1) s << "  Behavior = OracleParkAI ModuleTag_AI\n  End\n";
        if (c.ai == 2)
            s << "  Behavior = OracleParkJet ModuleTag_AI\n    ParkingOffset = " << c.offset << "\n  End\n";
        s << "End\n";
    }
    return s.str();
}

template <typename T>
static T* find_behavior(Object* obj) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (T* typed = dynamic_cast<T*>(*m)) return typed;
    }
    return nullptr;
}


static std::string info_text(const ParkingPlaceBehaviorInterface::PPInfo& p) {
    std::ostringstream s;
    s << coord(p.parkingSpace) << ',' << bits(p.parkingOrientation) << ',' << coord(p.runwayPrep) << ','
      << coord(p.runwayStart) << ',' << coord(p.runwayEnd) << ',' << coord(p.runwayExit) << ','
      << coord(p.runwayLandingStart) << ',' << coord(p.runwayLandingEnd) << ','
      << coord(p.runwayApproach) << ',' << coord(p.hangarInternal) << ',' << bits(p.runwayTakeoffDist)
      << ',' << bits(p.hangarInternalOrient);
    return s.str();
}

struct Scene {
    Object* field = nullptr;
    ParkingPlaceBehavior* park = nullptr;
    ProductionUpdate* prod = nullptr;
    std::vector<ObjectID> craft;
    std::vector<int> kind;
};

static std::string row(const Scene& sc, const std::string& ret) {
    std::ostringstream out;
    out << oracle_wp51::World::snapshot_save_hex(sc.park) << ' ' << oracle_wp51::World::snapshot_crc(sc.park)
        << ' ';
    if (sc.prod) out << oracle_wp51::World::snapshot_save_hex(sc.prod);
    else out << '-';
    const ModelConditionFlags& f = sc.field->getDrawable()->getModelConditionFlags();
    out << ' ' << f.test(MODELCONDITION_DOOR_1_OPENING) << f.test(MODELCONDITION_DOOR_2_OPENING)
        << f.test(MODELCONDITION_DOOR_3_OPENING) << f.test(MODELCONDITION_DOOR_4_OPENING);
    out << ' ' << GetGameLogicRandomSeedCRC() << ' ' << (ret.empty() ? "-" : ret) << ' '
        << (g_commands.empty() ? "-" : g_commands) << ' ';
    for (size_t i = 0; i < sc.craft.size(); ++i) {
        Object* o = TheGameLogic->findObjectByID(sc.craft[i]);
        if (i) out << ';';
        if (!o) {
            out << 'x';
            continue;
        }
        out << (o->isEffectivelyDead() ? 1 : 0) << ',' << coord(*o->getPosition()) << ','
            << bits(o->getOrientation()) << ',' << bits(o->getBodyModule()->getHealth()) << ','
            << o->getProducerID() << ','
            << (o->testStatus(OBJECT_STATUS_DECK_HEIGHT_OFFSET) ? 1 : 0) << ','
            << o->getControllingPlayer()->getPlayerIndex();
    }
    g_commands.clear();
    return out.str();
}

// A field equal to the same field of the previous row is written as '~'.
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

static Matrix3D random_bone() {
    Matrix3D m(true);
    m.Translate(real_in(-120.0f, 120.0f), real_in(-120.0f, 120.0f), real_in(-4.0f, 20.0f));
    m.Rotate_Z(real_in(-3.14f, 3.14f));
    if (pick(6) == 0) m.Rotate_X(real_in(-0.5f, 0.5f));
    if (pick(6) == 0) m.Rotate_Y(real_in(-0.5f, 0.5f));
    return m;
}

int main(int argc, char** argv) {
    if (argc < 2) die("usage: gen_pkt286_parking_place <out.json>");
    oracle_wp51::World w;
    (TheModuleFactory->*(&ExposedFactory::addModuleInternal))(
        make_rec_ai, AIUpdateInterface::friend_newModuleData, MODULETYPE_BEHAVIOR,
        "OracleParkAI", AIUpdateInterface::getInterfaceMask());
    (TheModuleFactory->*(&ExposedFactory::addModuleInternal))(
        make_rec_jet, JetAIUpdate::friend_newModuleData, MODULETYPE_BEHAVIOR,
        "OracleParkJet", JetAIUpdate::getInterfaceMask());
    w.load_ini(fixture_ini());
    w.init_with_neutral_player();
    Team* teams[3] = {w.team_neutral, w.team_a, w.team_b};

    static OracleDrawData draw_data;
    FILE* out = std::fopen(argv[1], "w");
    if (!out) return 1;
    const int kTotal = 1000;
    std::fprintf(out, "{\"vector_count\":%d,\"vectors\":[", kTotal);
    for (int v = 0; v < kTotal; ++v) {
        InitRandom(0x286000u + v);
        const int layout = v % kLayoutCount;
        const Layout& l = kLayouts[layout];
        UnsignedInt frame = 1 + pick(5000);
        w.set_frame(frame);
        g_commands.clear();

        // Bones: usually all there, sometimes some or all missing.
        g_bones.clear();
        std::ostringstream bone_text;
        const uint32_t presence = pick(8);
        std::vector<std::string> names;
        for (int r = 0; r < l.rows; ++r)
            for (int c = 0; c < l.cols; ++c) {
                const std::string base = "Runway" + std::to_string(c + 1);
                names.push_back(base + "Park" + std::to_string(r + 1) + "Han");
                names.push_back(base + "Parking" + std::to_string(r + 1));
                names.push_back(base + "Prep" + std::to_string(r + 1));
            }
        for (int c = 0; c < l.cols; ++c) {
            names.push_back("RunwayStart" + std::to_string(c + 1));
            names.push_back("RunwayEnd" + std::to_string(c + 1));
        }
        names.push_back("HeliPark01");
        for (const std::string& n : names) {
            if (presence == 0 || (presence == 1 && pick(2) == 0)) continue;
            const Matrix3D m = random_bone();
            g_bones.insert({n, m});
            bone_text << (bone_text.tellp() > 0 ? ";" : "") << n << '=';
            for (int r = 0; r < 3; ++r)
                for (int c = 0; c < 4; ++c)
                    bone_text << (r || c ? "," : "") << bits(m[r][c]);
        }

        Scene sc;
        const std::string field_name = "P286_Field" + std::to_string(layout);
        const Coord3D at = {real_in(200.0f, 700.0f), real_in(200.0f, 700.0f), pick(3) ? 0.0f : real_in(0.0f, 30.0f)};
        const Real angle = pick(4) ? real_in(-3.14f, 3.14f) : 0.0f;
        sc.field = w.spawn_object(field_name.c_str(), w.team_a, at, angle);
        sc.park = find_behavior<ParkingPlaceBehavior>(sc.field);
        sc.prod = find_behavior<ProductionUpdate>(sc.field);
        if (!sc.park || (sc.prod != nullptr) != (l.prod != 0)) die("field modules");
        Drawable* d = sc.field->getDrawable();
        if (!d) die("field without a drawable");
        Module* fake[2] = {newInstance(OracleDraw)(d, &draw_data), nullptr};
        Module** saved = (d->*access_member(DrawArrayTag()))[0];
        (d->*access_member(DrawArrayTag()))[0] = fake;

        std::ostringstream craft_text;
        const int crafts = 2 + static_cast<int>(pick(5));
        const bool all_jets = pick(3) == 0;
        for (int i = 0; i < crafts; ++i) {
            const int k = all_jets ? static_cast<int>(pick(2)) : static_cast<int>(pick(kCraftCount));
            const int team = pick(6) == 0 ? static_cast<int>(pick(3)) : 1;
            const bool air = pick(3) == 0;
            const Coord3D p = {real_in(100.0f, 900.0f), real_in(100.0f, 900.0f), air ? real_in(1.0f, 80.0f) : 0.0f};
            const std::string name = "P286_Craft" + std::to_string(k);
            Object* o = w.spawn_object(name.c_str(), teams[team], p, 0.5f);
            const int producer = pick(2);
            if (producer) o->setProducer(sc.field);
            sc.craft.push_back(o->getID());
            sc.kind.push_back(k);
            craft_text << (i ? ";" : "") << k << ',' << team << ',' << coord(p) << ',' << producer;
        }
        const int construction = pick(10) == 0 ? 1 : 0;
        if (construction) sc.field->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
        g_commands.clear();

        std::vector<std::string> last;
        std::fprintf(out,
                     "%s\n{\"layout\":%d,\"frame\":%u,\"seed\":%u,\"field\":\"%s,%u\",\"construction\":%d,"
                     "\"bones\":\"%s\",\"crafts\":\"%s\",\"spawn\":\"%s\",\"ops\":[",
                     v ? "," : "", layout, frame, 0x286000u + v, coord(at).c_str(), bits(angle), construction,
                     bone_text.str().c_str(), craft_text.str().c_str(), delta(last, row(sc, "")).c_str());

        std::vector<std::string> rows;
        std::vector<int> doors;  // reserved doors not yet used
        const int ops = 6 + static_cast<int>(pick(36));
        const bool queue = pick(2) == 0;
        const int prelude = queue ? crafts : 0;
        const int contention = queue ? 4 + static_cast<int>(pick(14)) : 0;
        for (int k = 0; k < prelude + contention + ops; ++k) {
            const int who = k < prelude ? k : static_cast<int>(pick(sc.craft.size()));
            Object* o = TheGameLogic->findObjectByID(sc.craft[who]);
            const ObjectID id = sc.craft[who];
            const bool helipad = sc.kind[who] == 3 || sc.kind[who] == 5;
            const std::string tmpl_name = "P286_Craft" + std::to_string(sc.kind[who]);
            const ThingTemplate* tt = TheThingFactory->findTemplate(tmpl_name.c_str());
            // Weighted choice of operation.
            static const int kWeights[] = {
                0, 0, 0, 1, 1, 1, 1, 2, 3, 3, 4, 4, 4, 4, 5, 5, 6, 6, 6, 6, 6, 7, 7, 8, 8, 9, 9, 9,
                10, 10, 10, 11, 11, 11, 11, 12, 13, 13, 14, 15, 15, 15, 16, 17, 18, 18, 19, 19, 20, 21};
            int code = kWeights[pick(sizeof(kWeights) / sizeof(kWeights[0]))];
            if (k < prelude) {
                code = 4;
            } else if (k < prelude + contention) {
                static const int kRunway[] = {6, 6, 6, 6, 6, 10, 10, 7, 9, 8, 5};
                code = kRunway[pick(11)];
            }
            int a = who, b = 0, c = 0;
            std::ostringstream ret;
            w.set_frame(frame);
            switch (code) {
            case 0: {  // reserveDoorForExit
                const ExitDoorType door = sc.park->reserveDoorForExit(tt, nullptr);
                ret << static_cast<int>(door);
                if (door >= 0) doors.push_back(door);
                break;
            }
            case 1: {  // exitObjectViaDoor
                if (!o) { code = 99; break; }
                int door;
                if (helipad && pick(5) != 0) {
                    door = DOOR_NONE_NEEDED;
                } else if (!doors.empty() && pick(6) != 0) {
                    const size_t at_door = pick(doors.size());
                    door = doors[at_door];
                    doors.erase(doors.begin() + at_door);
                } else {
                    door = static_cast<int>(pick(4));
                }
                b = door;
                sc.park->exitObjectViaDoor(o, static_cast<ExitDoorType>(door));
                break;
            }
            case 2: {  // unreserveDoorForExit
                static const int kDoors[] = {0, 1, 2, 3, -1, -2};
                b = (!doors.empty() && pick(2)) ? doors[pick(doors.size())] : kDoors[pick(6)];
                sc.park->unreserveDoorForExit(static_cast<ExitDoorType>(b));
                break;
            }
            case 3:  // queries
                b = pick(8) == 0 ? 0 : 1;
                ret << sc.park->hasAvailableSpaceFor(tt) << sc.park->shouldReserveDoorWhenQueued(tt)
                    << sc.park->hasReservedSpace(b ? id : INVALID_ID) << ','
                    << sc.park->getSpaceIndex(b ? id : INVALID_ID) << ',' << sc.park->getRunwayCount()
                    << ',' << bits(sc.park->getApproachHeight()) << ','
                    << bits(sc.park->getLandingDeckHeightOffset()) << ',' << sc.park->isExitBusy();
                break;
            case 4: {  // reserveSpace
                static const float kOffsets[] = {0.0f, 0.0f, 5.5f, -20.0f};
                b = static_cast<int>(pick(4));
                c = pick(3) ? 1 : 0;
                ParkingPlaceBehaviorInterface::PPInfo info;
                std::memset(static_cast<void*>(&info), 0, sizeof(info));
                ret << sc.park->reserveSpace(id, kOffsets[b], c ? &info : nullptr);
                if (c) ret << ',' << info_text(info);
                break;
            }
            case 5:  // releaseSpace
                sc.park->releaseSpace(id);
                break;
            case 6:  // reserveRunway
                b = static_cast<int>(pick(2));
                ret << sc.park->reserveRunway(id, b);
                break;
            case 7:  // releaseRunway
                sc.park->releaseRunway(id);
                break;
            case 8:  // getRunwayIndex
                b = pick(8) == 0 ? 0 : 1;
                ret << sc.park->getRunwayIndex(b ? id : INVALID_ID);
                break;
            case 9:  // getRunwayReservation, calcPPInfo
                if (sc.park->getRunwayCount() > 0) {
                    b = static_cast<int>(pick(sc.park->getRunwayCount()));
                    c = static_cast<int>(pick(2));
                    ret << sc.park->getRunwayReservation(b, static_cast<RunwayReservationType>(c)) << ',';
                    ParkingPlaceBehaviorInterface::PPInfo info;
                    std::memset(static_cast<void*>(&info), 0, sizeof(info));
                    sc.park->calcPPInfo(id, &info);
                    ret << info_text(info);
                } else {
                    code = 99;
                }
                break;
            case 10:  // transferRunwayReservationToNextInLineForTakeoff
                sc.park->transferRunwayReservationToNextInLineForTakeoff(id);
                break;
            case 11:  // setHealee
                if (!o) { code = 99; break; }
                b = pick(4) != 0 ? 1 : 0;
                sc.park->setHealee(o, b);
                break;
            case 12:  // scheduled frames
                a = 1 + static_cast<int>(pick(pick(3) ? 4 : 20));
                for (int t = 0; t < a; ++t) {
                    w.set_frame(++frame);
                    if (sc.park->update() != UPDATE_SLEEP_NONE) die("sleep");
                }
                break;
            case 13:  // damage
                if (!o) { code = 99; break; }
                b = 1 + static_cast<int>(pick(60));
                {
                    DamageInfo dmg;
                    dmg.in.m_damageType = DAMAGE_UNRESISTABLE;
                    dmg.in.m_amount = static_cast<Real>(b);
                    o->attemptDamage(&dmg);
                }
                break;
            case 14:  // kill all parked units
                sc.park->killAllParkedUnits();
                break;
            case 15: {  // defect all parked units
                b = static_cast<int>(pick(3));
                c = static_cast<int>(pick(2) ? 0 : 30);
                sc.park->defectAllParkedUnits(teams[b], c);
                break;
            }
            case 16: {  // the die interface
                DamageInfo dmg;
                sc.park->onDie(&dmg);
                break;
            }
            case 17:  // an aircraft dies or leaves the world
                if (!o) { code = 99; break; }
                b = static_cast<int>(pick(2));
                if (b) {
                    TheGameLogic->destroyObject(o);
                    w.flush_destroy_list();
                } else {
                    o->kill();
                }
                break;
            case 18: {  // move an aircraft up or down
                if (!o) { code = 99; break; }
                b = static_cast<int>(pick(2));
                Coord3D p = *o->getPosition();
                p.z = b ? 40.0f : 0.0f;
                o->setPosition(&p);
                break;
            }
            case 19: {  // jet takeoff and landing flags
                if (!o) { code = 99; break; }
                JetAIUpdate* jet = find_behavior<JetAIUpdate>(o);
                if (!jet) { code = 99; break; }
                b = static_cast<int>(pick(4));
                jet->friend_setTakeoffInProgress((b & 1) != 0);
                jet->friend_setLandingInProgress((b & 2) != 0);
                break;
            }
            case 20: {  // rally point and exit position
                b = static_cast<int>(pick(3));
                if (b == 0) {
                    const Coord3D p = {static_cast<float>(pick(1000)), static_cast<float>(pick(1000)), 0.0f};
                    c = static_cast<int>(p.x) * 1000 + static_cast<int>(p.y);
                    sc.park->setRallyPoint(&p);
                }
                Coord3D p1, p2;
                p1.zero();
                p2.zero();
                const Bool r1 = sc.park->getExitPosition(p1);
                const Bool r2 = sc.park->getNaturalRallyPoint(p2);
                const Coord3D* rally = sc.park->getRallyPoint();
                ret << r1 << ',' << coord(p1) << ',' << r2 << ',' << coord(p2) << ','
                    << (rally ? coord(*rally) : std::string("n"));
                break;
            }
            case 21:  // construction state
                b = static_cast<int>(pick(3));
                if (b == 0) sc.field->clearStatus(MAKE_OBJECT_STATUS_MASK2(OBJECT_STATUS_UNDER_CONSTRUCTION, OBJECT_STATUS_SOLD));
                else sc.field->setStatus(MAKE_OBJECT_STATUS_MASK(b == 1 ? OBJECT_STATUS_UNDER_CONSTRUCTION : OBJECT_STATUS_SOLD));
                break;
            }
            if (code == 99) {
                // Nothing to act on: run one frame instead.
                code = 12;
                a = 1;
                b = c = 0;
                w.set_frame(++frame);
                sc.park->update();
            }
            std::fprintf(out, "%s[%d,%d,%d,%d]", k ? "," : "", code, a, b, c);
            rows.push_back(delta(last, row(sc, ret.str())));
        }
        std::fprintf(out, "],\"rows\":[");
        for (size_t r = 0; r < rows.size(); ++r)
            std::fprintf(out, "%s\"%s\"", r ? "," : "", rows[r].c_str());
        std::fprintf(out, "]}");

        (d->*access_member(DrawArrayTag()))[0] = saved;
        deleteInstance(fake[0]);
        for (ObjectID id : sc.craft)
            if (Object* o = TheGameLogic->findObjectByID(id)) TheGameLogic->destroyObject(o);
        TheGameLogic->destroyObject(sc.field);
        w.flush_destroy_list();
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return 0;
}
