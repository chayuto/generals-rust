// SPDX-License-Identifier: GPL-3.0-or-later
// Golden trace vectors for the waypoint, wander, panic and repulsor states (AI plan
// slice S11): the unmodified reference AIStates.cpp states, entered by real orders on
// the real pathfind map and stepped by the real GameLogic::update.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s11_waypoint_states.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/S11_WaypointStates.json
//
// The fixture, observation, searches and tape are those of
// gen_s10_movement_states.cpp, included here with its main renamed. Replaced by a
// state that records its calls: every state of another slice, and the attack-follow
// states of a waypoint path. AI::findClosestRepulsor answers a random enemy.
// Player::isSkirmishAIPlayer answers true for the computer player in some scenes.
//
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface10doPathfindEP25PathfindServicesInterface
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder19getMoveAwayFromPathEP6ObjectS1_P4PathS1_S3_
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a PartitionManager.cpp.o __ZN16PartitionManager26getGroundOrStructureHeightEff
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface17getNextMoodTargetEbb
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AI.cpp.o __ZN2AI19findClosestRepulsorEPK6Objectf
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a Player.cpp.o __ZN6Player18isSkirmishAIPlayerEv
//
// Not covered: the wall layer and bridges, loading a save, waypoint ids that repeat.

#define main s10_main
#include "gen_s10_movement_states.cpp"
#undef main

S10_TAG(NextGroupTag, UnsignedInt, AI, m_nextGroupID)

static bool g_skirmish = false;

Bool real_is_skirmish(Player*) __asm__("_oracle_real__ZN6Player18isSkirmishAIPlayerEv");
Bool Player::isSkirmishAIPlayer() {
    if (g_skirmish && getPlayerIndex() == 2) return TRUE;
    return real_is_skirmish(this);
}

static int g_repulsor_percent = 12;

Object* AI::findClosestRepulsor(const Object* me, Real range) {
    Object* found = chance(g_repulsor_percent) ? some_enemy(me) : nullptr;
    tape("R," + who(me) + "," + fx(range) + "=" + who(found));
    return found;
}

class S11Terrain : public S10Terrain {
public:
    void clearWaypoints() { deleteWaypoints(); }
    Waypoint* addPoint(int id, float x, float y) {
        Coord3D loc = at(x, y);
        const std::string name = "WP" + num(id);
        Waypoint* way = newInstance(Waypoint)(static_cast<WaypointID>(id), name.c_str(), &loc, "", "", "", false);
        way->setNext(m_waypointListHead);
        m_waypointListHead = way;
        return way;
    }
};

static void replace_for_s11(AIUpdateInterface* ai) {
    AIStateMachine* machine = ai->*access_member(MachineTag());
    StateMap& states = machine->*access_member(StateMapTag());
    for (auto& entry : states) {
        switch (entry.first) {
        case AI_IDLE:
        case AI_DEAD:
        case AI_BUSY:
        case AI_MOVE_TO:
        case AI_MOVE_OUT_OF_THE_WAY:
        case AI_MOVE_AND_TIGHTEN:
        case AI_FOLLOW_PATH:
        case AI_FOLLOW_EXITPRODUCTION_PATH:
        case AI_FOLLOW_WAYPOINT_PATH_AS_TEAM:
        case AI_FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS:
        case AI_FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT:
        case AI_FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS_EXACT:
        case AI_WANDER:
        case AI_PANIC:
        case AI_WANDER_IN_PLACE:
        case AI_MOVE_AWAY_FROM_REPULSORS:
            break;
        case AI_ATTACK_MOVE_TO: {
            StateMachine* sub = static_cast<AIAttackMoveToState*>(entry.second)->*access_member(SubMachineTag());
            for (auto& inner : sub->*access_member(StateMapTag()))
                if (inner.first != AI_IDLE) replace_state(sub, inner, true);
            break;
        }
        default:
            replace_state(machine, entry, false);
        }
    }
}

// --- fixture ----------------------------------------------------------------

static std::string s11_locomotors_ini() {
    return locomotors_ini() +
           locomotor("S11_Stroll", "GROUND", "TWO_LEGS", 25,
                     "  WanderWidthFactor = 2.4\n  WanderAboutPointRadius = 35\n") +
           locomotor("S11_Run", "GROUND", "TWO_LEGS", 55,
                     "  WanderWidthFactor = 1.2\n  WanderAboutPointRadius = 20\n") +
           locomotor("S11_Tight", "GROUND", "FOUR_WHEELS", 45, "  WanderWidthFactor = 0.2\n");
}

static std::string s11_objects_ini() {
    return objects_ini() +
           object_ini("S11_Civ", "PRELOAD SELECTABLE INFANTRY CAN_BE_REPULSED", "S9_Legs", 4, true,
                      "  Locomotor = SET_WANDER S11_Stroll\n  Locomotor = SET_PANIC S11_Run\n") +
           object_ini("S11_Cart", "PRELOAD SELECTABLE VEHICLE CAN_BE_REPULSED", "S11_Tight", 5, true,
                      "  Locomotor = SET_PANIC S11_Run\n");
}

static const char* const k11Templates[] = {"S9_Inf",     "S9_Veh",     "S9_Tank",     "S9_Dozer",  "S9_Big",
                                           "S9_Climber", "S9_Air",     "S9_Block",    "S9_Jet",    "S9_Tower",
                                           "S9_Meander", "S9_Crusher", "S10_Missile", "S10_Digger", "S11_Civ",
                                           "S11_Cart"};
enum { K_CIV = K_DIGGER + 1, K_CART };

// --- waypoints --------------------------------------------------------------

static std::vector<Waypoint*> g_ways;
// "<id>,<x bits>,<y bits>,<z bits>[,<link id>]*" per waypoint, ';' between.
static std::string g_way_text;

static void make_waypoints(S11Terrain* terrain, int shape) {
    g_ways.clear();
    const int n = shape == 4 ? 1 : 2 + static_cast<int>(pick(6));
    Coord3D p = at(between(60.0f, 260.0f), between(60.0f, 260.0f));
    for (int i = 0; i < n; ++i) {
        float x = p.x, y = p.y;
        if (shape == 1 && chance(45)) {
            // At the rim of the map or beyond it.
            const float edge = chance(50) ? between(1.0f, 14.0f) : between(-40.0f, -2.0f);
            const float other = between(30.0f, 290.0f);
            const uint32_t side = pick(4);
            x = side == 0 ? edge : side == 1 ? 320.0f - edge : other;
            y = side == 2 ? edge : side == 3 ? 320.0f - edge : other;
        }
        g_ways.push_back(terrain->addPoint(i + 1, x, y));
        const float reach = shape == 2 ? 22.0f : 110.0f;
        p = at(inside(p.x + between(-reach, reach)), inside(p.y + between(-reach, reach)));
    }
    for (int i = 0; i + 1 < n; ++i) g_ways[i]->addLink(g_ways[i + 1]);
    if (shape == 3 && n > 1) g_ways[n - 1]->addLink(g_ways[0]);
    // Branches and ways back.
    for (int extra = static_cast<int>(pick(3)); extra > 0 && n > 2; --extra) {
        const int from = static_cast<int>(pick(n));
        const int to = static_cast<int>(pick(n));
        if (from != to) g_ways[from]->addLink(g_ways[to]);
    }
    g_way_text.clear();
    for (Waypoint* way : g_ways) {
        g_way_text += (g_way_text.empty() ? "" : ";") + num(way->getID()) + "," + c3(*way->getLocation());
        for (int k = 0; k < way->getNumLinks(); ++k) g_way_text += "," + num(way->getLink(k)->getID());
    }
}

// --- operations -------------------------------------------------------------

// "wp <unit> <command> <player> <waypoint id or 0>"
static void op_waypoint(int u, const char* name, AICommandType type, bool player, int way) {
    Object* o = unit(u);
    if (!o || !o->getAI()) return;
    g_ops.push_back(std::string(name) + " " + num(u) + " " + num(player ? 1 : 0) + " " +
                    num(way < 0 ? 0 : static_cast<int>(g_ways[way]->getID())));
    AICommandParms parms(type, player ? CMD_FROM_PLAYER : CMD_FROM_AI);
    parms.m_waypoint = way < 0 ? nullptr : g_ways[way];
    parms.m_intValue = NO_MAX_SHOTS_LIMIT;
    o->getAI()->aiDoCommand(&parms);
}

static void op_group(const std::vector<int>& members) {
    AIGroupPtr group = TheAI->createGroup();
    std::string text = "group";
    for (int u : members) {
        Object* o = unit(u);
        if (!o || !o->getAI()) continue;
        group->add(o);
        text += " " + num(u);
    }
    g_ops.push_back(text);
}

// What a scene's orders lean to.
enum Walk { W_FOLLOW, W_TEAM, W_EXACT, W_WANDER, W_PANIC, W_PLACE, W_MIXED };

static void walk_order(int u, int walk) {
    const bool player = chance(55);
    if (walk == W_MIXED) walk = static_cast<int>(pick(6));
    // The first waypoint mostly, so that long walks happen.
    int way = chance(65) ? 0 : static_cast<int>(pick(static_cast<uint32_t>(g_ways.size())));
    if (chance(4) && walk != W_TEAM) way = -1;
    const bool team = chance(50);
    switch (walk) {
    case W_FOLLOW: op_waypoint(u, "follow_way", AICMD_FOLLOW_WAYPOINT_PATH, player, way); break;
    case W_TEAM: op_waypoint(u, "follow_team", AICMD_FOLLOW_WAYPOINT_PATH_AS_TEAM, player, way < 0 ? 0 : way); break;
    case W_EXACT:
        if (team && way >= 0) op_waypoint(u, "exact_team", AICMD_FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT, player, way);
        else op_waypoint(u, "exact", AICMD_FOLLOW_WAYPOINT_PATH_EXACT, player, way);
        break;
    case W_WANDER: op_waypoint(u, "wander", AICMD_WANDER, player, way); break;
    case W_PANIC: op_waypoint(u, "panic", AICMD_PANIC, player, way); break;
    default: op_waypoint(u, "in_place", AICMD_WANDER_IN_PLACE, player, -1); break;
    }
}

static void s11_random_op(int units, int walk) {
    const int u = static_cast<int>(pick(units));
    Object* o = unit(u);
    if (!o || !o->getAI()) return;
    AIUpdateInterface* ai = o->getAI();
    const int v = static_cast<int>(pick(units));
    Object* other = unit(v);
    const uint32_t r = pick(100);
    if (r < 45) {
        walk_order(u, chance(70) ? walk : W_MIXED);
    } else if (r < 55) {
        give_order(u, units, F_ANY);
    } else if (r < 63) {
        std::vector<int> members = {u};
        if (v != u) members.push_back(v);
        if (chance(40)) members.push_back(static_cast<int>(pick(units)));
        op_group(members);
        for (int m : members) walk_order(m, chance(70) ? W_TEAM : W_EXACT);
    } else if (r < 70) {
        if (!other || !other->getAI() || v == u || !o->getPhysics()) return;
        const Bool result = ai->processCollision(o->getPhysics(), other);
        g_ops.push_back("coll " + num(u) + " " + num(v) + " " + num(result ? 1 : 0));
    } else if (r < 76) {
        const int attitude = chance(60) ? ATTITUDE_AGGRESSIVE : (chance(50) ? ATTITUDE_ALERT : ATTITUDE_NORMAL);
        g_ops.push_back("att " + num(u) + " " + num(attitude));
        ai->setAttitude(static_cast<AttitudeType>(attitude));
    } else if (r < 82) {
        static const ObjectStatusTypes kBits[] = {OBJECT_STATUS_IMMOBILE, OBJECT_STATUS_PARACHUTING};
        const int bit = static_cast<int>(pick(2));
        const int on = chance(50) ? 1 : 0;
        g_ops.push_back("status " + num(u) + " " + num(kBits[bit]) + " " + num(on));
        set_status_bit(o, kBits[bit], on != 0);
        if (on && chance(60)) walk_order(u, walk);
    } else if (r < 90) {
        // Next to a waypoint, so that it is reached at once.
        const Waypoint* way = g_ways[pick(static_cast<uint32_t>(g_ways.size()))];
        Coord3D p = at(inside(way->getLocation()->x + between(-9.0f, 9.0f)),
                       inside(way->getLocation()->y + between(-9.0f, 9.0f)));
        const float angle = between(-3.14f, 3.14f);
        g_ops.push_back("pos " + num(u) + " " + c3(p) + " " + fx(angle));
        o->setPosition(&p);
        o->setOrientation(angle);
    } else if (r < 94) {
        const float speed = chance(30) ? 999999.0f : between(5.0f, 60.0f);
        g_ops.push_back("speed " + num(u) + " " + fx(speed));
        ai->setDesiredSpeed(speed);
    } else {
        op_order(u, "idle", AICMD_IDLE, chance(50), -1, {});
    }
}

// `observe` with a long path (field 7) shortened to its node count and a digest:
// paths of hundreds of nodes would be most of the file.
static std::string observe_short(int i) {
    std::string s = observe(i);
    size_t begin = 0;
    for (int field = 0; field < 7; ++field) {
        begin = s.find('|', begin);
        if (begin == std::string::npos) return s;
        ++begin;
    }
    const size_t end = s.find('|', begin);
    const std::string path = s.substr(begin, end - begin);
    if (path.size() <= 120) return s;
    return s.substr(0, begin) + path.substr(0, path.find('/')) + "#" + hex64(fnv(path)) + s.substr(end);
}

// Each unit's group and its team's current waypoint.
static std::string team_text(int units) {
    std::string s;
    for (int i = 0; i < units; ++i) {
        Object* o = unit(i);
        if (!o) {
            s += (i ? ";" : "") + std::string("gone");
            continue;
        }
        AIGroup* group = o->getGroup();
        const Waypoint* way = o->getTeam()->getCurrentWaypoint();
        s += (i ? ";" : "") + num(group ? static_cast<long long>(group->getID()) : 0) + "," +
             num(way ? static_cast<int>(way->getID()) : 0);
    }
    return s;
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_s11_waypoint_states <out.json>\n");
        return 2;
    }
    g_state = 0x53313157u;
    oracle_wp51::World w;
    TheFramePacer = new FramePacer;
    TheCommandList = NEW CommandList;
    TheCommandList->init();
    TheRecorder = new RecorderClass;
    TheRecorder->init();
    S11Terrain* terrain = new S11Terrain;
    g_terrain = terrain;
    TheTerrainLogic = terrain;
    TheTerrainLogic->init();
    TheWritableGlobalData->m_gravity = -1.0f;
    TheWritableGlobalData->m_partitionCellSize = 20.0f;
    ThePartitionManager = NEW PartitionManager;
    ThePartitionManager->init();
    const_cast<TAiData*>(TheAI->getAiData())->m_forceIdleFramesCount = 45;
    const_cast<TAiData*>(TheAI->getAiData())->m_repulsedDistance = 50.0f;
    const_cast<TAiData*>(TheAI->getAiData())->m_skirmishGroupFudgeValue = 40.0f;
    w.load_ini(s11_locomotors_ini());
    w.load_ini(s11_objects_ini());
    w.init_with_neutral_player();
    ThePlayerList->getNthPlayer(1)->setPlayerType(PLAYER_HUMAN, FALSE);
    ThePlayerList->getNthPlayer(2)->setPlayerType(PLAYER_COMPUTER, FALSE);

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* out = std::fopen(tmp.c_str(), "w");
    if (!out) return 1;
    auto escape = [](const std::string& text) {
        std::string escaped;
        for (char c : text) escaped += c == '\n' ? std::string("\\n") : std::string(1, c);
        return escaped;
    };

    const int kScenarios = 70;
    const int kFrames = 120;
    const int kLongFrom = 63;
    const int kLongFrames = 420;
    std::fprintf(out, "{\"vector_count\":%d,\"cells\":%d,\"locomotors\":\"%s\",\"objects\":\"%s\",\"vectors\":[\n",
                 kScenarios, kCells, escape(s11_locomotors_ini()).c_str(), escape(s11_objects_ini()).c_str());

    bool first_row = true;
    for (int scenario = 0; scenario < kScenarios; ++scenario) {
        const bool quiet = scenario >= kLongFrom;
        const int walk = scenario % 7;
        const int shape = quiet ? (scenario % 2 ? 0 : 3) : static_cast<int>(pick(5));
        const UnsignedInt frame = scenario % 9 == 0 ? 1 : 100 + pick(3000);
        const UnsignedInt seed = 1 + pick(1000000000);
        g_skirmish = scenario % 3 == 1;
        g_repulsor_percent = quiet ? 2 : 10 + static_cast<int>(pick(25));
        std::string& cliff = g_terrain->cliff;
        cliff.assign(kCells * kCells, '0');
        g_pen = false;
        if (chance(55))
            for (int n = 1 + static_cast<int>(pick(4)); n > 0; --n)
                blob(cliff, 4 + static_cast<int>(pick(24)), 4 + static_cast<int>(pick(24)), 1 + pick(4), 1 + pick(4));

        ThePlayerList->getNthPlayer(1)->getDefaultTeam()->setCurrentWaypoint(nullptr);
        w.team_a->setCurrentWaypoint(nullptr);
        w.team_b->setCurrentWaypoint(nullptr);
        terrain->clearWaypoints();
        make_waypoints(terrain, shape);

        std::vector<Spawn> spawns;
        const bool crowd = chance(35);
        const int count = 3 + static_cast<int>(pick(4));
        for (int i = 0; i < count; ++i) {
            static const int kKinds[] = {K_INF, K_VEH,  K_TANK, K_MEANDER, K_CIV,  K_CIV,
                                         K_CART, K_CART, K_BIG,  K_DOZER,   K_AIR, K_MISSILE};
            int kind = kKinds[pick(12)];
            if ((walk == W_WANDER || walk == W_PANIC || walk == W_PLACE) && chance(50))
                kind = chance(70) ? K_CIV : K_CART;
            const float x = crowd ? 100.0f + between(-30.0f, 30.0f) : between(40.0f, 280.0f);
            const float y = crowd ? 100.0f + between(-30.0f, 30.0f) : between(40.0f, 280.0f);
            const int team = walk == W_TEAM ? (g_skirmish ? 1 : 0) : (chance(60) ? 0 : 1);
            spawns.push_back({kind, i == count - 1 ? 1 - team : team, x, y, between(-3.14f, 3.14f)});
        }

        TheGameLogic->*access_member(Wp51GLFrameTag()) = frame;
        InitRandom(seed);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1000));
        TheAI->*access_member(NextGroupTag()) = 0;
        Pathfinder* pf = TheAI->pathfinder();
        pf->reset();
        PathfindCellInfo::allocateCellInfos();
        pf->newMap();

        g_ids.clear();
        g_ops.clear();
        g_tape.clear();
        std::ostringstream spawn_text;
        for (size_t i = 0; i < spawns.size(); ++i) {
            const Spawn& s = spawns[i];
            const ThingTemplate* tt = TheThingFactory->findTemplate(k11Templates[s.kind]);
            if (!tt) return 3;
            Object* o = TheThingFactory->newObject(tt, s.team ? w.team_b : w.team_a);
            if (o->getAI()) replace_for_s11(o->getAI());
            Coord3D p = at(s.x, s.y);
            if ((s.kind == K_AIR || s.kind == K_MISSILE) && s.angle > 0.0f) p.z = 40.0f;
            o->setOrientation(s.angle);
            o->setPosition(&p);
            g_ids.push_back(o->getID());
            spawn_text << (i ? ";" : "") << s.kind << "," << s.team << "," << c3(p) << "," << fx(s.angle);
        }
        const int units = static_cast<int>(g_ids.size());

        std::fprintf(out,
                     "%s{\"s\":%d,\"frame\":%u,\"seed\":%u,\"skirmish\":%d,\"ways\":\"%s\",\"spawns\":\"%s\","
                     "\"cliff\":\"%s\",\"tape\":\"%s\"}",
                     first_row ? "" : ",\n", scenario, static_cast<unsigned>(frame), static_cast<unsigned>(seed),
                     g_skirmish ? 1 : 0, g_way_text.c_str(), spawn_text.str().c_str(), cliff.c_str(), g_tape.c_str());
        first_row = false;

        std::vector<std::string> last(units);
        const int frames = quiet ? kLongFrames : kFrames;
        for (int f = 0; f < frames; ++f) {
            g_ops.clear();
            g_tape.clear();
            if (f == 1) {
                if (walk == W_TEAM || (walk == W_EXACT && chance(50))) {
                    std::vector<int> members;
                    for (int u = 0; u + 1 < units; ++u) members.push_back(u);
                    op_group(members);
                    for (int u : members) walk_order(u, walk);
                    walk_order(units - 1, W_MIXED);
                } else {
                    for (int u = 0; u < units; ++u)
                        if (chance(90)) walk_order(u, quiet ? walk : (chance(80) ? walk : W_MIXED));
                }
            } else if (f > 1 && chance(quiet ? 1 : 12)) {
                for (int n = 1 + static_cast<int>(pick(2)); n > 0; --n) s11_random_op(units, walk);
            }
            TheGameLogic->update();
            std::string ops_text;
            for (size_t k = 0; k < g_ops.size(); ++k) ops_text += (k ? ";" : "") + g_ops[k];
            std::ostringstream row;
            row << "{\"s\":" << scenario << ",\"f\":" << f << ",\"ops\":\"" << ops_text << "\",\"tape\":\"" << g_tape
                << "\",\"seed\":\"" << hex32(GetGameLogicRandomSeedCRC()) << "\",\"map\":\"" << cells_digest()
                << "\",\"teams\":\"" << team_text(units) << "\",\"u\":\"";
            for (int i = 0; i < units; ++i) {
                const std::string now = observe_short(i);
                row << (i ? "~" : "") << (now == last[i] ? std::string("=") : now);
                last[i] = now;
            }
            row << "\"}";
            std::fprintf(out, ",\n%s", row.str().c_str());
        }
        std::fflush(out);

        for (ObjectID id : g_ids)
            if (Object* o = TheGameLogic->findObjectByID(id)) TheGameLogic->destroyObject(o);
        w.flush_destroy_list();
        pf->processPathfindQueue();
        pf->processPathfindQueue();
        ThePartitionManager->update();
        if (TheGameLogic->getFirstObject()) return 4;
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return std::rename(tmp.c_str(), argv[1]) == 0 ? 0 : 1;
}
