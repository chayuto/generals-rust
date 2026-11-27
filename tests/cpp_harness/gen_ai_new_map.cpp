// Records what `AIPlayer::newMap` leaves behind that the game start comparison
// (`gen_game_start.cpp`) does not show, from the running reference engine, and calls
// `AISkirmishPlayer::adjustBuildList`, `AIPlayer::computeCenterAndRadiusOfBase` and
// `BuildAssistant::moveObjectsForConstruction` with chosen inputs.
//
// Built and driven by `tests/cpp_harness/gen_ai_new_map.py` (a dylib loaded into the
// stopped engine by lldb).
//
// State lines are tab separated, `<kind>\t<id>\t...`, floats as bits; their content
// names retail data, so only their digests are kept. Lines starting with `synth` have
// inputs chosen here and are kept whole.

#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <functional>
#include <map>
#include <string>
#include <utility>
#include <vector>

#define private public
#define protected public

#include "Common/BuildAssistant.h"
#include "Common/Energy.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/RandomValue.h"
#include "Common/ScoreKeeper.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/AIPlayer.h"
#include "GameLogic/AISkirmishPlayer.h"
#include "GameLogic/AIStateMachine.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/SidesList.h"
#include "GameLogic/TerrainLogic.h"

namespace {

uint32_t bits(float value) {
    uint32_t out;
    std::memcpy(&out, &value, sizeof out);
    return out;
}

// Saves into memory.
class ByteXfer : public Xfer {
public:
    std::vector<unsigned char> bytes;
    ByteXfer() { m_xferMode = XFER_SAVE; }
    void open(AsciiString) override {}
    void close() override {}
    Int beginBlock() override { return 0; }
    void endBlock() override {}
    void skip(Int) override {}
    void xferSnapshot(Snapshot* snapshot) override { snapshot->xfer(this); }
    // As `XferSave` does: a length byte, then the characters.
    void xferAsciiString(AsciiString* text) override {
        unsigned char length = static_cast<unsigned char>(text->getLength());
        xferImplementation(&length, 1);
        if (length) xferImplementation(const_cast<char*>(text->str()), length);
    }
    void xferImplementation(void* data, Int size) override {
        const unsigned char* p = static_cast<const unsigned char*>(data);
        bytes.insert(bytes.end(), p, p + size);
    }
};

void write_hex(FILE* out, const std::vector<unsigned char>& bytes) {
    for (unsigned char b : bytes) std::fprintf(out, "%02x", b);
}

void write_players(FILE* out) {
    for (int i = 0; i < ThePlayerList->getPlayerCount(); ++i) {
        Player* p = ThePlayerList->getNthPlayer(i);
        std::fprintf(out, "flags\t%d\t%d\n", i, p->m_canBuildUnits ? 1 : 0);
        if (p->m_ai) {
            ByteXfer x;
            static_cast<Snapshot*>(p->m_ai)->xfer(&x);
            std::fprintf(out, "ai\t%d\t", i);
            write_hex(out, x.bytes);
            std::fprintf(out, "\n");
        }
        int at = 0;
        for (BuildListInfo* info = p->getBuildList(); info; info = info->getNext(), ++at) {
            ByteXfer x;
            static_cast<Snapshot*>(info)->xfer(&x);
            std::fprintf(out, "build\t%d\t%d\t", i, at);
            write_hex(out, x.bytes);
            std::fprintf(out, "\n");
        }
        ScoreKeeper* score = p->getScoreKeeper();
        std::vector<std::string> built;
        for (const auto& entry : score->m_objectsBuilt) {
            built.push_back(std::string(entry.first->getName().str()) + ":" +
                            std::to_string(entry.second));
        }
        std::sort(built.begin(), built.end());
        std::fprintf(out, "score\t%d\t%d\t%d\t%d\t%d\t", i, score->m_totalMoneySpent,
            score->m_totalMoneyEarned, score->m_totalUnitsBuilt, score->m_totalBuildingsBuilt);
        for (const std::string& entry : built) std::fprintf(out, "%s,", entry.c_str());
        std::fprintf(out, "\n");
        std::fprintf(out, "energy\t%d\t%d\t%d\n", i, p->getEnergy()->m_energyProduction,
            p->getEnergy()->m_energyConsumption);
    }
}

void write_objects(FILE* out) {
    std::map<unsigned, Object*> objects;
    for (Object* obj = TheGameLogic->getFirstObject(); obj; obj = obj->getNextObject()) {
        objects[static_cast<unsigned>(obj->getID())] = obj;
    }
    for (const auto& entry : objects) {
        Object* obj = entry.second;
        std::fprintf(out, "objx\t%u\t%s\t%u\n", entry.first, obj->getName().str(),
            static_cast<unsigned>(obj->m_scriptStatus));
        AIUpdateInterface* ai = obj->getAIUpdateInterface();
        if (!ai || obj->isDestroyed()) continue;
        const Coord3D* goal = ai->getGoalPosition();
        const ICoord2D* cell = ai->getPathfindGoalCell();
        std::fprintf(out, "unit\t%u\t%u\t%08x\t%08x\t%08x\t%u\t%d\n", entry.first,
            static_cast<unsigned>(ai->getCurrentStateID()), bits(goal->x), bits(goal->y),
            bits(goal->z), static_cast<unsigned>(ai->getStateMachine()->m_goalObjectID),
            static_cast<int>(ai->getLastCommandSource()));
        // The reserved goal cell and whether a path was asked for.
        std::fprintf(out, "path\t%u\t%d\t%d\t%d\n", entry.first, cell->x, cell->y,
            ai->isWaitingForPath() ? 1 : 0);
    }
}

void write_maps(FILE* out) {
    Pathfinder* pathfinder = TheAI->pathfinder();
    const IRegion2D& e = pathfinder->m_extent;
    for (int y = e.lo.y; y <= e.hi.y; ++y) {
        std::fprintf(out, "cells\t%d\t", y);
        for (int x = e.lo.x; x <= e.hi.x; ++x) {
            PathfindCell* cell = pathfinder->getCell(LAYER_GROUND, x, y);
            std::fprintf(out, "%d.%u,", static_cast<int>(cell->getType()),
                static_cast<unsigned>(cell->getObstacleID()));
        }
        std::fprintf(out, "\n");
    }
    for (int player = 0; player < ThePlayerList->getPlayerCount(); ++player) {
        std::fprintf(out, "shroud\t%d\t", player);
        for (int y = 0; y < ThePartitionManager->m_cellCountY; ++y) {
            for (int x = 0; x < ThePartitionManager->m_cellCountX; ++x) {
                std::fputc('0' + static_cast<int>(
                    ThePartitionManager->getShroudStatusForPlayer(player, x, y)), out);
            }
        }
        std::fprintf(out, "\n");
    }
}

// ---- chosen inputs ----------------------------------------------------------

BuildListInfo* make_list(const std::vector<std::pair<const ThingTemplate*, Coord3D>>& entries) {
    BuildListInfo* head = nullptr;
    BuildListInfo* tail = nullptr;
    for (const auto& entry : entries) {
        BuildListInfo* info = newInstance(BuildListInfo);
        info->setTemplateName(entry.first->getName());
        info->setLocation(entry.second);
        if (tail) tail->setNextBuildList(info);
        else head = info;
        tail = info;
    }
    return head;
}

const ThingTemplate* first_template(bool command_center, const ThingTemplate* not_this) {
    for (const ThingTemplate* t = TheThingFactory->firstTemplate(); t; t = t->friend_getNextTemplate()) {
        if (t != not_this && t->isKindOf(KINDOF_COMMANDCENTER) == command_center &&
            t->isKindOf(KINDOF_STRUCTURE)) {
            return t;
        }
    }
    return nullptr;
}

Player* first_skirmish_player(Object** command_center) {
    for (int i = 0; i < ThePlayerList->getPlayerCount(); ++i) {
        Player* p = ThePlayerList->getNthPlayer(i);
        if (!p->m_ai || !dynamic_cast<AISkirmishPlayer*>(p->m_ai)) continue;
        for (Object* obj = TheGameLogic->getFirstObject(); obj; obj = obj->getNextObject()) {
            if (obj->getControllingPlayer() == p && obj->isKindOf(KINDOF_COMMANDCENTER)) {
                *command_center = obj;
                return p;
            }
        }
    }
    return nullptr;
}

float unit_float(uint32_t& state, float lo, float hi) {
    state = state * 1664525u + 1013904223u;
    return lo + (hi - lo) * static_cast<float>(state >> 8) / 16777216.0f;
}

void synth_adjust(FILE* out) {
    Object* center = nullptr;
    Player* player = first_skirmish_player(&center);
    if (!player) return;
    AISkirmishPlayer* ai = static_cast<AISkirmishPlayer*>(player->m_ai);
    const ThingTemplate* cc = center->getTemplate();
    const ThingTemplate* other = first_template(false, nullptr);
    Region3D bounds;
    TheTerrainLogic->getMaximumPathfindExtent(&bounds);
    TAiData* data = const_cast<TAiData*>(TheAI->getAiData());
    const Bool rotate_was = data->m_rotateSkirmishBases;
    uint32_t state = 12345;
    // Which entries are command centers; the last two have none at the head or at all.
    const char* shapes[] = {"cccc", "c..c.", "c....", "..c..", "....."};
    for (int shape = 0; shape < 5; ++shape) {
        for (int ninth = 0; ninth < 9; ++ninth) {
            const int rotate = (shape == 1 && ninth % 2) ? 0 : 1;
            data->m_rotateSkirmishBases = rotate != 0;
            Coord3D start;
            start.x = bounds.lo.x + bounds.width() * (static_cast<float>(ninth % 3) +
                unit_float(state, 0.05f, 0.95f)) / 3.0f;
            start.y = bounds.lo.y + bounds.height() * (static_cast<float>(ninth / 3) +
                unit_float(state, 0.05f, 0.95f)) / 3.0f;
            start.z = 0.0f;
            center->setPosition(&start);
            start = *center->getPosition();
            std::vector<std::pair<const ThingTemplate*, Coord3D>> entries;
            for (const char* s = shapes[shape]; *s; ++s) {
                Coord3D at;
                at.x = unit_float(state, -900.0f, 900.0f);
                at.y = unit_float(state, -900.0f, 900.0f);
                at.z = 0.0f;
                entries.push_back({*s == 'c' ? cc : other, at});
            }
            BuildListInfo* list = make_list(entries);
            ai->adjustBuildList(list);
            std::fprintf(out, "synth adjust shape=%s rotate=%d start=%08x,%08x bounds=%08x,%08x,%08x,%08x in=",
                shapes[shape], rotate, bits(start.x), bits(start.y), bits(bounds.lo.x),
                bits(bounds.lo.y), bits(bounds.hi.x), bits(bounds.hi.y));
            for (const auto& entry : entries) {
                std::fprintf(out, "%08x:%08x,", bits(entry.second.x), bits(entry.second.y));
            }
            std::fprintf(out, " out=");
            for (BuildListInfo* info = list; info; info = info->getNext()) {
                std::fprintf(out, "%08x:%08x:%d,", bits(info->getLocation()->x),
                    bits(info->getLocation()->y), info->isInitiallyBuilt() ? 1 : 0);
            }
            std::fprintf(out, "\n");
            deleteInstance(list);
        }
    }
    // Starts exactly on the lines between the ninths, and just past them.
    data->m_rotateSkirmishBases = true;
    const float xs[] = {bounds.lo.x + bounds.width() / 3, bounds.lo.x + 2 * bounds.width() / 3};
    const float ys[] = {bounds.lo.y + bounds.height() / 3, bounds.lo.y + 2 * bounds.height() / 3};
    for (int line = 0; line < 8; ++line) {
        Coord3D start;
        start.x = xs[line & 1];
        start.y = ys[(line >> 1) & 1];
        start.z = 0.0f;
        if (line & 4) {
            start.x = std::nextafterf(start.x, 1.0e9f);
            start.y = std::nextafterf(start.y, 1.0e9f);
        }
        center->setPosition(&start);
        start = *center->getPosition();
        std::vector<std::pair<const ThingTemplate*, Coord3D>> entries;
        Coord3D at;
        at.x = 100.0f;
        at.y = 40.0f;
        at.z = 0.0f;
        entries.push_back({cc, at});
        at.x = 250.0f;
        at.y = -75.0f;
        entries.push_back({other, at});
        BuildListInfo* list = make_list(entries);
        ai->adjustBuildList(list);
        std::fprintf(out, "synth adjust shape=c. rotate=1 start=%08x,%08x bounds=%08x,%08x,%08x,%08x in=",
            bits(start.x), bits(start.y), bits(bounds.lo.x), bits(bounds.lo.y), bits(bounds.hi.x),
            bits(bounds.hi.y));
        for (const auto& entry : entries) {
            std::fprintf(out, "%08x:%08x,", bits(entry.second.x), bits(entry.second.y));
        }
        std::fprintf(out, " out=");
        for (BuildListInfo* info = list; info; info = info->getNext()) {
            std::fprintf(out, "%08x:%08x:%d,", bits(info->getLocation()->x),
                bits(info->getLocation()->y), info->isInitiallyBuilt() ? 1 : 0);
        }
        std::fprintf(out, "\n");
        deleteInstance(list);
    }
    data->m_rotateSkirmishBases = rotate_was;
}

void synth_center(FILE* out) {
    Object* center = nullptr;
    Player* player = first_skirmish_player(&center);
    if (!player) return;
    // Two templates given geometry chosen here.
    const ThingTemplate* a = first_template(false, nullptr);
    const ThingTemplate* b = first_template(false, a);
    const float radius_a = 37.25f;
    const float radius_b = 113.7f;
    const_cast<GeometryInfo&>(a->getTemplateGeometryInfo()).set(GEOMETRY_CYLINDER, false, 10.0f, radius_a, radius_a);
    const_cast<GeometryInfo&>(b->getTemplateGeometryInfo()).set(GEOMETRY_CYLINDER, false, 10.0f, radius_b, radius_b);
    uint32_t state = 999;
    BuildListInfo* was = player->m_pBuildList;
    for (int count = 0; count < 36; ++count) {
        std::vector<std::pair<const ThingTemplate*, Coord3D>> entries;
        const bool close = count >= 24;
        for (int i = 0; i < (close ? 2 + count % 4 : count % 12); ++i) {
            Coord3D at;
            at.x = close ? 1000.0f + unit_float(state, -0.9f, 0.9f) : unit_float(state, -200.0f, 3000.0f);
            at.y = close ? 700.0f + unit_float(state, -0.9f, 0.9f) : unit_float(state, -200.0f, 3000.0f);
            at.z = 0.0f;
            entries.push_back({(i + count / 12) % 3 ? a : b, at});
        }
        player->m_pBuildList = make_list(entries);
        Coord3D at;
        at.x = at.y = at.z = 0.0f;
        Real radius = 0.0f;
        player->m_ai->computeCenterAndRadiusOfBase(&at, &radius);
        std::fprintf(out, "synth center in=");
        for (const auto& entry : entries) {
            std::fprintf(out, "%08x:%08x:%08x,", bits(entry.second.x), bits(entry.second.y),
                bits(entry.first == a ? radius_a : radius_b));
        }
        std::fprintf(out, " out=%08x,%08x,%08x,%d\n", bits(at.x), bits(at.y), bits(radius),
            player->m_ai->m_baseCenterSet ? 1 : 0);
        deleteInstance(player->m_pBuildList);
    }
    player->m_pBuildList = was;
}

// How often `WWMath::Pow(x, 2)` differs from `x * x`.
void synth_square(FILE* out) {
    uint32_t state = 77;
    int differ = 0;
    int written = 0;
    for (int i = 0; i < 2000000; ++i) {
        const float x = unit_float(state, 1.0f, 512.0f);
        const float power = static_cast<float>(WWMath::Pow(x, 2));
        if (bits(power) != bits(x * x)) ++differ;
        if (written < 32 && (i < 16 || bits(power) != bits(x * x))) {
            std::fprintf(out, "synth square in=%08x out=%08x\n", bits(x), bits(power));
            ++written;
        }
    }
    std::fprintf(out, "synth squares differ=%d\n", differ);
}

void synth_move(FILE* out) {
    Object* center = nullptr;
    Player* player = first_skirmish_player(&center);
    if (!player) return;
    Object* unit = nullptr;
    for (Object* obj = TheGameLogic->getFirstObject(); obj; obj = obj->getNextObject()) {
        if (obj->getControllingPlayer() == player && obj->getAIUpdateInterface() &&
            !obj->isKindOf(KINDOF_STRUCTURE) && !obj->isKindOf(KINDOF_IMMOBILE)) {
            unit = obj;
            break;
        }
    }
    if (!unit) return;
    const ThingTemplate* what = first_template(false, nullptr);
    GeometryInfo& geometry = const_cast<GeometryInfo&>(what->getTemplateGeometryInfo());
    Region3D bounds;
    TheTerrainLogic->getMaximumPathfindExtent(&bounds);
    uint32_t state = 4242;
    int made = 0;
    for (int attempt = 0; attempt < 600 && made < 48; ++attempt) {
        const int box = attempt % 3 == 0 ? 0 : 1;
        const float major = unit_float(state, 8.0f, 90.0f);
        const float minor = unit_float(state, 8.0f, 90.0f);
        geometry.set(box ? GEOMETRY_BOX : GEOMETRY_CYLINDER, false, 20.0f, major, minor);
        Coord3D at;
        at.x = unit_float(state, bounds.lo.x + 500.0f, bounds.hi.x - 500.0f);
        at.y = unit_float(state, bounds.lo.y + 500.0f, bounds.hi.y - 500.0f);
        at.z = unit_float(state, -5.0f, 5.0f);
        const float angle = unit_float(state, -3.0f, 3.0f);
        const unsigned seed = 1000u + attempt;
        unit->setPosition(&at);
        ThePartitionManager->update();
        // What the generator holds after the two draws one moved unit makes.
        InitRandom(seed);
        GameLogicRandomValueReal(0.5, 1.5);
        GameLogicRandomValueReal(-PI, PI);
        const unsigned expected = GetGameLogicRandomSeedCRC();
        InitRandom(seed);
        unit->getAIUpdateInterface()->aiIdle(CMD_FROM_AI);
        InitRandom(seed);
        TheBuildAssistant->moveObjectsForConstruction(what, &at, angle, player);
        // Anything else in the way would have drawn too.
        if (GetGameLogicRandomSeedCRC() != expected) continue;
        const Coord3D* goal = unit->getAIUpdateInterface()->getGoalPosition();
        std::fprintf(out, "synth move box=%d major=%08x minor=%08x at=%08x,%08x,%08x seed=%u out=%08x,%08x,%08x\n",
            box, bits(major), bits(minor), bits(at.x), bits(at.y), bits(at.z), seed,
            bits(goal->x), bits(goal->y), bits(goal->z));
        ++made;
    }
}

void write_state(FILE* out) {
    std::fprintf(out, "seed\t%08x\n", static_cast<unsigned>(GetGameLogicRandomSeedCRC()));
    write_players(out);
    write_objects(out);
    write_maps(out);
}

// The first structure by name of `side` which docks supply trucks.
const ThingTemplate* supply_center_of(const AsciiString& side) {
    const ThingTemplate* best = nullptr;
    for (const ThingTemplate* t = TheThingFactory->firstTemplate(); t; t = t->friend_getNextTemplate()) {
        if (t->getDefaultOwningSide() != side || !t->isKindOf(KINDOF_STRUCTURE)) continue;
        const ModuleInfo& modules = t->getBehaviorModuleInfo();
        bool docks = false;
        for (int i = 0; i < modules.getCount(); ++i) {
            if (modules.getNthName(i) == "SupplyCenterDockUpdate") docks = true;
        }
        if (docks && (!best || std::strcmp(t->getName().str(), best->getName().str()) < 0)) best = t;
    }
    return best;
}

// `AIPlayer::newMap` once more, over the first skirmish player's build list with
// most entries marked as built from the start and a supply centre added at the end:
// the faction lists hold none. Some entries are unsellable or named.
void build_whole_base(const std::string& path) {
    Object* center = nullptr;
    Player* player = first_skirmish_player(&center);
    if (!player) return;
    int i = 0;
    for (BuildListInfo* info = player->getBuildList(); info; info = info->getNext(), ++i) {
        info->setInitiallyBuilt(i % 5 != 4);
        info->setUnsellable(i % 3 == 0);
        if (i % 4 == 0) {
            AsciiString name;
            name.format("n%d", i);
            info->setBuildingName(name);
        }
    }
    const ThingTemplate* supply = supply_center_of(player->getSide());
    if (supply) {
        BuildListInfo* last = player->getBuildList();
        while (last && last->getNext()) last = last->getNext();
        Coord3D at = *center->getPosition();
        at.x += 300.0f;
        at.y += 260.0f;
        BuildListInfo* info = newInstance(BuildListInfo);
        info->setTemplateName(supply->getName());
        info->setLocation(at);
        info->setInitiallyBuilt(true);
        if (last) last->setNextBuildList(info);
    }
    player->m_ai->AIPlayer::newMap();
    FILE* out = std::fopen(path.c_str(), "w");
    if (!out) return;
    write_state(out);
    std::fclose(out);
}

void dump(const char* path, bool synthetic, bool build) {
    FILE* out = std::fopen(path, "w");
    if (!out) return;
    write_state(out);
    // These change the world, so they come last.
    if (build) {
        build_whole_base(std::string(path) + ".built");
    } else if (synthetic) {
        synth_adjust(out);
        synth_center(out);
        synth_square(out);
        synth_move(out);
    }
    std::fclose(out);
}

}  // namespace

// Runs when lldb loads the library into the stopped engine.
__attribute__((constructor)) static void zh_dump_on_load() {
    if (const char* path = std::getenv("ZH_DUMP_PATH")) {
        dump(path, std::getenv("ZH_DUMP_SYNTHETIC") != nullptr,
             std::getenv("ZH_DUMP_BUILD") != nullptr);
    }
}
