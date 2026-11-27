// Records the world the reference engine holds right after a game start: the stock
// headless `GeneralsXZH` plays a replay, and at the first `GameLogic::update` this
// library walks `TheGameLogic`, `ThePlayerList` and `TheTeamFactory`.
//
// Built and driven by `tests/cpp_harness/gen_game_start.py` (a dylib loaded into the
// stopped process by lldb; engine symbols resolve against the executable).
// Output lines (tab separated, floats as bits):
//   seed <GetGameLogicRandomSeed> <GetGameLogicRandomSeedCRC>
//   game <frame> <next object id>
//   player <index> <name> <side> <money> <default team> <enemy mask> <ally mask>
//   team <team id> <prototype id> <name> <player> <singleton> <active> <members>
//   obj <id> <template> <team id> <team name> <player> <x> <y> <z> <angle> <layer>
//       <health> <max health> <status words> <disabled> <vision> <shroud clearing>
//   bridge <list index> <template> <layer> <damage state> <object id> <tower ids>
//       <from> <to> <from left> <from right> <to left> <to right> (x y z each)
//   deck <list index> <first cell x> <first cell y> <cells across> <cells down>
//       <per cell: type and connect layer of the bridge layer, then of the ground>

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>

#include "Common/NameKeyGenerator.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/RandomValue.h"
#include "Common/Team.h"
#include "Common/ThingTemplate.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/TerrainLogic.h"

namespace {

uint32_t bits(float value) {
    uint32_t out;
    std::memcpy(&out, &value, sizeof out);
    return out;
}

void write_object(FILE* out, Object* obj) {
    const Coord3D* pos = obj->getPosition();
    Team* team = obj->getTeam();
    BodyModuleInterface* body = obj->getBodyModule();
    std::fprintf(out, "obj\t%u\t%s\t%u\t%s\t%d\t%08x\t%08x\t%08x\t%08x\t%d",
        static_cast<unsigned>(obj->getID()),
        obj->getTemplate()->getName().str(),
        team ? static_cast<unsigned>(team->getID()) : 0u,
        team ? team->getName().str() : "",
        obj->getControllingPlayer() ? obj->getControllingPlayer()->getPlayerIndex() : -1,
        bits(pos->x), bits(pos->y), bits(pos->z), bits(obj->getOrientation()),
        static_cast<int>(obj->getLayer()));
    std::fprintf(out, "\t%08x\t%08x",
        body ? bits(body->getHealth()) : 0u,
        body ? bits(body->getMaxHealth()) : 0u);
    std::fputc('\t', out);
    ObjectStatusMaskType status = obj->getStatusBits();
    for (int i = 0; i < OBJECT_STATUS_COUNT; ++i) {
        if (status.test(i)) std::fprintf(out, "%d,", i);
    }
    std::fputc('\t', out);
    for (int i = 0; i < DISABLED_COUNT; ++i) {
        if (obj->isDisabledByType(static_cast<DisabledType>(i))) std::fprintf(out, "%d,", i);
    }
    std::fprintf(out, "\t%08x\t%08x\n",
        bits(obj->getVisionRange()), bits(obj->getShroudClearingRange()));
}

void write_cell(FILE* out, PathfindLayerEnum layer, int x, int y) {
    PathfindCell* cell = TheAI->pathfinder()->getCell(layer, x, y);
    if (!cell) {
        std::fputs("--", out);
        return;
    }
    std::fprintf(out, "%x%x", static_cast<unsigned>(cell->getType()),
        static_cast<unsigned>(cell->getConnectLayer()));
}

// The terrain's bridges and the pathfind cells under each, two cells beyond its bounds.
void write_bridges(FILE* out) {
    int index = 0;
    for (Bridge* bridge = TheTerrainLogic->getFirstBridge(); bridge; bridge = bridge->getNext(), ++index) {
        BridgeInfo info;
        bridge->getBridgeInfo(&info);
        std::fprintf(out, "bridge\t%d\t%s\t%d\t%d\t%u\t%u,%u,%u,%u", index,
            bridge->getBridgeTemplateName().str(), static_cast<int>(bridge->getLayer()),
            static_cast<int>(info.curDamageState), static_cast<unsigned>(info.bridgeObjectID),
            static_cast<unsigned>(info.towerObjectID[0]), static_cast<unsigned>(info.towerObjectID[1]),
            static_cast<unsigned>(info.towerObjectID[2]), static_cast<unsigned>(info.towerObjectID[3]));
        const Coord3D* points[6] = { &info.from, &info.to, &info.fromLeft, &info.fromRight,
            &info.toLeft, &info.toRight };
        for (const Coord3D* p : points) {
            std::fprintf(out, "\t%08x\t%08x\t%08x", bits(p->x), bits(p->y), bits(p->z));
        }
        std::fputc('\n', out);
        const Region2D* bounds = bridge->getBounds();
        const int x0 = static_cast<int>(bounds->lo.x / 10.0f) - 2;
        const int y0 = static_cast<int>(bounds->lo.y / 10.0f) - 2;
        const int nx = static_cast<int>(bounds->hi.x / 10.0f) + 3 - x0;
        const int ny = static_cast<int>(bounds->hi.y / 10.0f) + 3 - y0;
        std::fprintf(out, "deck\t%d\t%d\t%d\t%d\t%d\t", index, x0, y0, nx, ny);
        for (int y = y0; y < y0 + ny; ++y) {
            for (int x = x0; x < x0 + nx; ++x) {
                write_cell(out, bridge->getLayer(), x, y);
                write_cell(out, LAYER_GROUND, x, y);
            }
        }
        std::fputc('\n', out);
    }
}

void dump_game_start(const char* path) {
    FILE* out = std::fopen(path, "w");
    if (!out) return;
    std::fprintf(out, "seed\t%u\t%08x\n",
        static_cast<unsigned>(GetGameLogicRandomSeed()),
        static_cast<unsigned>(GetGameLogicRandomSeedCRC()));
    std::fprintf(out, "game\t%u\t%u\n",
        static_cast<unsigned>(TheGameLogic->getFrame()),
        static_cast<unsigned>(TheGameLogic->getObjectIDCounter()));
    for (int i = 0; i < ThePlayerList->getPlayerCount(); ++i) {
        Player* p = ThePlayerList->getNthPlayer(i);
        unsigned enemies = 0;
        unsigned allies = 0;
        for (int j = 0; j < ThePlayerList->getPlayerCount(); ++j) {
            Player* other = ThePlayerList->getNthPlayer(j);
            if (!other->getDefaultTeam()) continue;
            Relationship r = p->getRelationship(other->getDefaultTeam());
            if (r == ENEMIES) enemies |= 1u << j;
            if (r == ALLIES) allies |= 1u << j;
        }
        std::fprintf(out, "player\t%d\t%s\t%s\t%u\t%u\t%04x\t%04x\n", i,
            TheNameKeyGenerator->keyToName(p->getPlayerNameKey()).str(),
            p->getSide().str(),
            static_cast<unsigned>(p->getMoney()->countMoney()),
            p->getDefaultTeam() ? static_cast<unsigned>(p->getDefaultTeam()->getID()) : 0u,
            enemies, allies);
        for (Player::PlayerTeamList::const_iterator it = p->getPlayerTeams()->begin();
             it != p->getPlayerTeams()->end(); ++it) {
            for (DLINK_ITERATOR<Team> t = (*it)->iterate_TeamInstanceList(); !t.done(); t.advance()) {
                Team* team = t.cur();
                int members = 0;
                for (DLINK_ITERATOR<Object> m = team->iterate_TeamMemberList(); !m.done(); m.advance()) ++members;
                std::fprintf(out, "team\t%u\t%u\t%s\t%d\t%d\t%d\t%d\n",
                    static_cast<unsigned>(team->getID()),
                    static_cast<unsigned>((*it)->getID()),
                    team->getName().str(), i,
                    (*it)->getIsSingleton() ? 1 : 0,
                    team->isActive() ? 1 : 0, members);
            }
        }
    }
    for (Object* obj = TheGameLogic->getFirstObject(); obj; obj = obj->getNextObject()) {
        write_object(out, obj);
    }
    write_bridges(out);
    std::fclose(out);
}

}  // namespace

// Runs when lldb loads the library into the stopped engine.
__attribute__((constructor)) static void zh_dump_on_load() {
    if (const char* path = std::getenv("ZH_DUMP_PATH")) dump_game_start(path);
}
