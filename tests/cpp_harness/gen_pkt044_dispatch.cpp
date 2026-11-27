// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for the selection and hotkey squad handling of
// `GameLogic::logicMessageDispatcher` (PKT-044), run on real objects.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt044_dispatch.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT-044.json
//
// Not covered: the orders themselves (movement, production, special powers and so on)
// and the replay camera.

#include "oracle_wp51_world.h"

#include "Common/MessageStream.h"
#include "Common/Player.h"
#include "Common/Recorder.h"
#include "Common/Team.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/Squad.h"

#include <cstdio>
#include <string>
#include <unistd.h>
#include <vector>

// `setGameMode` also tells the mouse, which is not there.
struct GameModeTag { using type = GameMode GameLogic::*; };
template struct MemberAccess<GameModeTag, &GameLogic::m_gameMode>;
inline GameMode GameLogic::* access_member(GameModeTag);
struct SelectionTag { using type = Squad* Player::*; };
template struct MemberAccess<SelectionTag, &Player::m_currentSelection>;
inline Squad* Player::* access_member(SelectionTag);
struct SquadsTag { using type = Squad* (Player::*)[NUM_HOTKEY_SQUADS]; };
template struct MemberAccess<SquadsTag, &Player::m_squads>;
inline Squad* (Player::* access_member(SquadsTag))[NUM_HOTKEY_SQUADS];
struct SquadIdsTag { using type = VecObjectID Squad::*; };
template struct MemberAccess<SquadIdsTag, &Squad::m_objectIDs>;
inline VecObjectID Squad::* access_member(SquadIdsTag);

static uint32_t g_state = 0x04400044u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}

static std::string num(long long v) { return std::to_string(v); }

static std::string ids(Squad* s) {
    std::string out;
    if (!s) return out;
    for (ObjectID id : s->*access_member(SquadIdsTag())) out += (out.empty() ? "" : ".") + num(id);
    return out;
}

static std::string squads_dump() {
    std::string out;
    for (int p = 0; p < 2; ++p) {
        Player* player = ThePlayerList->getNthPlayer(p);
        out += ids(player->*access_member(SelectionTag()));
        for (int i = 0; i < NUM_HOTKEY_SQUADS; ++i) out += "/" + ids((player->*access_member(SquadsTag()))[i]);
        out += "|";
    }
    return out;
}

// "id,selectable,mass,join,owner;" for every live object.
static std::string objects_dump() {
    std::string out;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        const bool join = o->getAIUpdateInterface() || o->isKindOf(KINDOF_STRUCTURE) || o->isKindOf(KINDOF_ALWAYS_SELECTABLE);
        int owner = -1;
        for (int p = 0; p < 2; ++p)
            if (o->getControllingPlayer() == ThePlayerList->getNthPlayer(p)) owner = p;
        out += num(o->getID()) + "," + num(o->isSelectable()) + "," + num(o->isMassSelectable()) + "," + num(join) +
               "," + num(owner) + ";";
    }
    return out;
}

int main(int argc, char** argv) {
    if (argc != 2) return 2;
    oracle_wp51::World w;
    TheCommandList = NEW CommandList;
    TheRecorder = NEW RecorderClass;
    TheRecorder->init();

    static const char* kinds[] = {"SELECTABLE", "STRUCTURE SELECTABLE", "SELECTABLE", "ALWAYS_SELECTABLE",
                                  "PRELOAD", "STRUCTURE", "SELECTABLE"};
    static const bool with_ai[] = {true, false, false, false, true, false, true};
    std::string ini;
    for (int k = 0; k < 7; ++k) {
        ini += "Object P044_" + std::to_string(k) + "\n  KindOf = " + kinds[k] +
               "\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n";
        if (with_ai[k]) ini += "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n";
        ini += "End\n";
    }
    w.load_ini(ini);

    std::vector<ObjectID> all;
    auto spawn = [&](int kind, int team) {
        const std::string name = "P044_" + std::to_string(kind);
        Object* o = w.spawn_object(name.c_str(), team ? w.team_b : w.team_a);
        if (!o) { std::fprintf(stderr, "spawn failed\n"); std::exit(1); }
        all.push_back(o->getID());
    };
    for (int i = 0; i < 12; ++i) spawn(i % 7, (i / 3) % 2);

    TheGameLogic->*access_member(GameModeTag()) = GAME_SKIRMISH;
    bool in_game = true;

    std::string out = "{\n\"vectors\":[\n";
    const int steps = 900;
    for (int step = 0; step < steps; ++step) {
        std::string world_op;
        const uint32_t roll = pick(100);
        if (roll < 3 && all.size() < 60) {
            spawn(pick(7), pick(2));
        } else if (roll < 6) {
            if (Object* o = TheGameLogic->findObjectByID(all[pick(all.size())])) {
                TheGameLogic->destroyObject(o);
                TheGameLogic->processDestroyList();
            }
        } else if (roll < 10) {
            if (Object* o = TheGameLogic->findObjectByID(all[pick(all.size())])) o->setSelectable(pick(2));
        } else if (roll < 12) {
            if (Object* o = TheGameLogic->findObjectByID(all[pick(all.size())])) o->setTeam(pick(2) ? w.team_b : w.team_a);
        } else if (roll < 14) {
            in_game = !in_game || pick(2);
            TheGameLogic->*access_member(GameModeTag()) = in_game ? GAME_SKIRMISH : GAME_NONE;
        }

        const int player = pick(20) == 0 ? 7 : static_cast<int>(pick(2));
        auto some_id = [&]() -> ObjectID {
            if (pick(12) == 0) return static_cast<ObjectID>(pick(2) ? 0 : 5000);
            // Prefer recent objects so squads overlap.
            const size_t n = all.size();
            return all[n - 1 - pick(static_cast<uint32_t>(std::min<size_t>(n, 14)))];
        };
        GameMessage::Type type;
        const uint32_t what = pick(100);
        if (what < 30) type = pick(2) ? GameMessage::MSG_CREATE_SELECTED_GROUP : GameMessage::MSG_CREATE_SELECTED_GROUP_NO_SOUND;
        else if (what < 45) type = GameMessage::MSG_REMOVE_FROM_SELECTED_GROUP;
        else if (what < 49) type = GameMessage::MSG_DESTROY_SELECTED_GROUP;
        else if (what < 62) type = static_cast<GameMessage::Type>(GameMessage::MSG_CREATE_TEAM0 + pick(what < 55 ? 3 : 10));
        else if (what < 74) type = static_cast<GameMessage::Type>(GameMessage::MSG_SELECT_TEAM0 + pick(what < 68 ? 3 : 10));
        else if (what < 86) type = static_cast<GameMessage::Type>(GameMessage::MSG_ADD_TEAM0 + pick(what < 80 ? 3 : 10));
        // A network order that does nothing here: only the common selection work runs.
        else type = GameMessage::MSG_PURCHASE_SCIENCE;

        GameMessage* m = newInstance(GameMessage)(type);
        m->friend_setPlayerIndex(player);
        std::string args;
        auto add_id = [&](ObjectID id) {
            m->appendObjectIDArgument(id);
            args += " 3:" + num(id);
        };
        if (type == GameMessage::MSG_CREATE_SELECTED_GROUP || type == GameMessage::MSG_CREATE_SELECTED_GROUP_NO_SOUND) {
            const bool create = pick(3) != 0;
            m->appendBooleanArgument(create);
            args += " 2:" + num(create);
            for (int i = pick(6); i >= 0; --i) add_id(some_id());
        } else if (type == GameMessage::MSG_REMOVE_FROM_SELECTED_GROUP) {
            for (int i = pick(4); i >= 0; --i) add_id(some_id());
        } else if (type >= GameMessage::MSG_CREATE_TEAM0 && type <= GameMessage::MSG_CREATE_TEAM9) {
            for (int i = pick(6); i > 0; --i) add_id(some_id());
        } else if (type == GameMessage::MSG_PURCHASE_SCIENCE) {
            m->appendIntegerArgument(SCIENCE_INVALID);
            args += " 0:-1";
        }

        const std::string objects = objects_dump();
        // `getNthPlayer` is null for index 7, and the dispatcher returns at once.
        TheGameLogic->logicMessageDispatcher(m, nullptr);
        deleteInstance(m);
        out += "{\"seed\":" + num(step) + ",\"in_game\":" + num(in_game) + ",\"msg\":\"" + num(type) + " " +
               num(player) + args + "\",\"objects\":\"" + objects + "\",\"squads\":\"" + squads_dump() + "\"}" +
               (step + 1 < steps ? ",\n" : "\n");
    }
    out += "]\n}\n";
    oracle::write_file_atomic(argv[1], out);
    std::fflush(nullptr);
    _exit(0);
}
