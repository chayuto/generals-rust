// Dumps the running reference world at the entry of a logic update, one file per
// frame. Loaded into the stopped engine by `gen_game_frames.py`, which calls
// `zh_dump_frame` at each stop.
//
// The rows are those of `gen_game_start.cpp` (seed, game, player, team, obj, bridge,
// deck) plus, per player, `energy`, and per object, `wake` (the next call frame of
// every update module, by module tag) and `ai` (the state of the unit's AI machine).

#include "gen_game_start.cpp"

#include "Common/Energy.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/UpdateModule.h"

namespace {

void write_energy(FILE* out) {
    for (int i = 0; i < ThePlayerList->getPlayerCount(); ++i) {
        Player* p = ThePlayerList->getNthPlayer(i);
        std::fprintf(out, "energy\t%d\t%d\t%d\n", i,
            static_cast<int>(p->getEnergy()->getProduction()),
            static_cast<int>(p->getEnergy()->getConsumption()));
    }
}

void write_schedule(FILE* out, Object* obj) {
    std::fprintf(out, "wake\t%u\t", static_cast<unsigned>(obj->getID()));
    for (BehaviorModule** m = obj->getBehaviorModules(); *m; ++m) {
        if (!(*m)->getUpdate()) continue;
        UpdateModule* update = static_cast<UpdateModule*>(*m);
        std::fprintf(out, "%s:%u,",
            TheNameKeyGenerator->keyToName((*m)->getModuleTagNameKey()).str(),
            static_cast<unsigned>(update->friend_getNextCallFrame()));
    }
    std::fputc('\n', out);
    // Not part of a digest: the class behind each tag, for reading a difference.
    std::fprintf(out, "module\t%u\t", static_cast<unsigned>(obj->getID()));
    for (BehaviorModule** m = obj->getBehaviorModules(); *m; ++m) {
        if (!(*m)->getUpdate()) continue;
        std::fprintf(out, "%s:%s,",
            TheNameKeyGenerator->keyToName((*m)->getModuleTagNameKey()).str(),
            TheNameKeyGenerator->keyToName((*m)->getModuleNameKey()).str());
    }
    std::fputc('\n', out);
    if (AIUpdateInterface* ai = obj->getAI()) {
        std::fprintf(out, "ai\t%u\t%u\n", static_cast<unsigned>(obj->getID()),
            static_cast<unsigned>(ai->getCurrentStateID()));
    }
}

}  // namespace

// Writes `$ZH_FRAMES_DIR/f<frame>.txt` and returns the frame.
extern "C" __attribute__((visibility("default"), used)) unsigned zh_dump_frame() {
    const char* dir = std::getenv("ZH_FRAMES_DIR");
    if (!dir) return 0;
    const unsigned frame = static_cast<unsigned>(TheGameLogic->getFrame());
    char path[1024];
    std::snprintf(path, sizeof path, "%s/f%04u.txt", dir, frame);
    dump_game_start(path);
    FILE* out = std::fopen(path, "a");
    if (!out) return 0;
    write_energy(out);
    for (Object* obj = TheGameLogic->getFirstObject(); obj; obj = obj->getNextObject()) {
        write_schedule(out, obj);
    }
    std::fclose(out);
    return frame;
}
