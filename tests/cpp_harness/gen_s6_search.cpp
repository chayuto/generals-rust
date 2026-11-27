// Golden vectors for the other path searches (PKT-411): findClosestPath,
// patchPath, getMoveAwayFromPath, findSafePath and findAttackPath, over the
// unmodified reference library.
//
// The scenes, state digests and setup rows are those of gen_s5_search.cpp,
// which is included whole. Each row is "<op> <inputs> = <answer> | <state>".
// What the attack search asks of the weapon is recorded as a table over every
// position it can ask about.
//
// The included S5 driver answers doPathfind, so its wrap applies here too.
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface10doPathfindEP25PathfindServicesInterface
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s6_search.cpp -o <exe>
//   <exe> docs/golden_vectors/S6_Search.json

#define main gen_s5_main
#include "gen_s5_search.cpp"
#undef main

#include "GameLogic/Weapon.h"

#include <set>

S5_TAG(AiLoco, LocomotorSet, AIUpdateInterface, m_locomotorSet)
S5_TAG(AiMachine, AIStateMachine*, AIUpdateInterface, m_stateMachine)
typedef void (Pathfinder::*S6Adjust)(Int, Int, Bool, Coord3D&, PathfindLayerEnum);
S5_FN(PfAdjust, S6Adjust, Pathfinder::adjustCoordToCell)
S5_FN(PfGetRadius, void (Pathfinder::*)(const Object*, Int&, Bool&), Pathfinder::getRadiusAndCenter)

static std::vector<std::string> g_armed, g_airfields;
// Units standing on an airfield of this scene, with that airfield.
static std::vector<std::pair<Object*, Object*>> g_parked;
static std::map<std::string, std::pair<int, int>> g_tally;

static void tally(const std::string& what, const Path* path) {
    (path ? g_tally[what].first : g_tally[what].second)++;
}

static void s6_templates() {
    static const float ranges[] = {15.0f, 40.0f, 75.0f, 120.0f, 200.0f, 60.0f};
    for (int i = 0; i < 6; ++i) {
        char text[256];
        std::snprintf(text, sizeof(text),
                      "Weapon S6Gun%d\n  PrimaryDamage = 10.0\n  AttackRange = %.1f\n  MinimumAttackRange = %.1f\n"
                      "  DamageType = ARMOR_PIERCING\n  DelayBetweenShots = 100\nEnd\n",
                      i, ranges[i], i % 3 == 2 ? ranges[i] * 0.4f : 0.0f);
        g_ini += text;
    }
    const std::string ai = "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n  Locomotor = SET_NORMAL S5Loco\n";
    static const float radii[] = {2.0f, 7.0f, 10.0f, 15.0f, 19.0f, 5.0f};
    for (int i = 0; i < 12; ++i) {
        const std::string name = "S6Armed" + num(i);
        std::string kinds = i % 3 == 0 ? "INFANTRY" : "VEHICLE";
        if (i % 4 == 3) kinds += " ATTACK_NEEDS_LINE_OF_SIGHT";
        if (i % 4 == 1) kinds += " ATTACK_NEEDS_LINE_OF_SIGHT";
        const int crusher = i % 5 == 4 ? 2 : i % 5 == 1 ? 1 : 0;
        std::string extra = "  CrusherLevel = " + num(crusher) + "\n  CrushableLevel = " + num(i % 3) + "\n" + ai +
                            "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY S6Gun" + num(i % 6) +
                            "\n  End\n";
        object_ini(name, kinds, "CYLINDER", radii[i % 6], radii[i % 6], extra);
        g_armed.push_back(name);
    }
    for (int i = 0; i < 3; ++i) {
        const std::string name = "S6Air" + num(i);
        object_ini(name, "STRUCTURE IMMOBILE FS_AIRFIELD", "BOX", 25.0f + 10.0f * i, 20.0f + 5.0f * i, "");
        g_airfields.push_back(name);
    }
}

// ---- rows -----------------------------------------------------------------------

// The nodes of a path as a call takes it: count, then position, layer and the
// next optimised node.
static std::string nodes_text(Path* path) {
    std::vector<PathNode*> nodes;
    for (PathNode* n = path->getFirstNode(); n; n = n->getNext()) nodes.push_back(n);
    std::string out = num(static_cast<long long>(nodes.size()));
    for (PathNode* n : nodes) {
        int next = -1;
        for (size_t i = 0; i < nodes.size(); ++i)
            if (nodes[i] == n->getNextOptimized()) next = static_cast<int>(i);
        out += " " + c3(*n->getPosition()) + " " + num(n->getLayer()) + " " + num(next);
    }
    return out;
}

static void radius_center(Object* o, Int& radius, Bool& center) {
    (pf()->*access_member(PfGetRadius()))(o, radius, center);
}

static void goal_object_rows(const Scene& s) {
    for (Object* o : s.everything) {
        row("goalobj " + num(o->getID()) + " " + c3(*o->getPosition()) + " " + num(o->getLayer()) + " " +
            num(o->isKindOf(KINDOF_FS_AIRFIELD) != 0));
    }
}

// Some units busy, the rest idle. Changing state moves goal reservations, so
// this comes before the map is built again.
static void set_busy(const Scene& s) {
    for (Object* o : s.units) {
        AIStateMachine* machine = o->getAIUpdateInterface()->*access_member(AiMachine());
        if (chance(50)) machine->setState(AI_BUSY);
    }
}

static void idle_row(const Scene& s) {
    std::string out = "idle";
    for (Object* o : s.units) {
        if (o->getAIUpdateInterface()->isIdle()) out += " " + num(o->getID());
    }
    row(out);
}

// A mover with the unit moved to `at` for the length of the call. The AI's own
// locomotor set takes the same surfaces, for the search that reads it.
struct Placed {
    Mover m;
    Coord3D at, was;
    int own_surfaces = 0;
    Bool own_downhill = false;
};

static void place(const Scene& s, Placed& p, const Coord3D& at, Object* who = nullptr) {
    choose(s, p.m, at);
    if (who) {
        p.m.unit->*access_member(ObjLayer()) = p.m.was;
        p.m.unit->getAIUpdateInterface()->*access_member(AiThrough()) = false;
        p.m.unit = who;
        p.m.was = who->getLayer();
        who->*access_member(ObjLayer()) = p.m.layer;
        who->getAIUpdateInterface()->*access_member(AiThrough()) = p.m.through;
    }
    p.at = at;
    p.was = *p.m.unit->getPosition();
    *const_cast<Coord3D*>(p.m.unit->getPosition()) = at;
    LocomotorSet& set = p.m.unit->getAIUpdateInterface()->*access_member(AiLoco());
    p.own_surfaces = set.*access_member(LsSurfaces());
    p.own_downhill = set.*access_member(LsDownhill());
    set.*access_member(LsSurfaces()) = p.m.surfaces;
    set.*access_member(LsDownhill()) = p.m.downhill;
}

static void unplace(Placed& p) {
    LocomotorSet& set = p.m.unit->getAIUpdateInterface()->*access_member(AiLoco());
    set.*access_member(LsSurfaces()) = p.own_surfaces;
    set.*access_member(LsDownhill()) = p.own_downhill;
    *const_cast<Coord3D*>(p.m.unit->getPosition()) = p.was;
    done(p.m);
}

// Mostly ordinary ground units, so that most searches get somewhere.
static void ease(Placed& p) {
    if (!chance(65)) return;
    p.m.surfaces = 1;
    p.m.downhill = false;
    p.m.set.*access_member(LsSurfaces()) = 1;
    p.m.set.*access_member(LsDownhill()) = false;
    LocomotorSet& set = p.m.unit->getAIUpdateInterface()->*access_member(AiLoco());
    set.*access_member(LsSurfaces()) = 1;
    set.*access_member(LsDownhill()) = false;
}

// A place on or just off a fence, which only a crusher may stand on.
static bool by_fence(const Scene& s, float off, Coord3D& at) {
    std::vector<Object*> fences;
    for (Object* o : s.blocks)
        if (o->getTemplate()->getFenceWidth() > 0.0f) fences.push_back(o);
    if (fences.empty()) return false;
    at = *pick(fences)->getPosition();
    at.x += between(-off, off);
    at.y += between(-off, off);
    return true;
}

static void closest_row(Scene& s) {
    Coord3D from, to;
    if (chance(50)) endpoints(s, from, to);
    else far_endpoints(s, from, to);
    passable(s);
    Placed p;
    place(s, p, from);
    ease(p);
    // Goals on and beside structures, with one of them ignored.
    if (!s.blocks.empty() && chance(30)) {
        Object* under = pick(s.blocks);
        to = *under->getPosition();
        to.x += between(-15.0f, 15.0f);
        to.y += between(-15.0f, 15.0f);
        if (chance(70)) {
            p.m.ignore = chance(60) ? under->getID() : pick(s.blocks)->getID();
            pf()->*access_member(PfIgnore()) = p.m.ignore;
        }
    }
    // A goal on an airfield, ignoring something that stands on it.
    if (!g_parked.empty() && chance(20)) {
        const auto& parked = g_parked[below(static_cast<int>(g_parked.size()))];
        to = *parked.second->getPosition();
        to.x += between(-15.0f, 15.0f);
        to.y += between(-15.0f, 15.0f);
        // Not the mover itself, whose goal object row would be stale.
        p.m.ignore = chance(70) && parked.first != p.m.unit ? parked.first->getID() : pick(s.blocks)->getID();
        pf()->*access_member(PfIgnore()) = p.m.ignore;
    }
    if (chance(8)) {  // an ignored obstacle that is gone
        p.m.ignore = oid(999999);
        pf()->*access_member(PfIgnore()) = p.m.ignore;
    }
    if (chance(3)) p.m.surfaces = 0, p.m.set.*access_member(LsSurfaces()) = 0;
    const bool blocked = chance(35);
    static const float mults[] = {0.0f, 1.0f, 1.0f, 0.2f, 1.2f, 25.0f};
    const float mult = mults[below(6)];
    Coord3D raw = to;
    Path* path = services()->findClosestPath(p.m.unit, p.m.set, &from, &raw, blocked, mult, FALSE);
    tally("closest", path);
    row("closest " + mover_text(p.m) + " " + c3(from) + " " + c3(to) + " " + num(blocked) + " " + f32(mult) + " = " +
        path_dump(path) + " " + c3(raw) + " " + num(pf()->*access_member(PfIgnore())) + " | " + state(s));
    if (path) deleteInstance(path);
    unplace(p);
}

static void occupy_cell(const Scene& s, PathfindLayerEnum layer, const Coord3D& pos) {
    const ICoord2D at = cell_of(pos);
    PathfindCell* cell = pf()->getCell(layer, at.x, at.y);
    if (!cell) return;
    Object* u = pick(s.units);
    const std::string where = num(layer) + " " + num(at.x) + " " + num(at.y) + " " + num(u->getID());
    if (chance(55)) {
        cell->setPosUnit(u->getID(), at);
        row("pos " + where);
    } else {
        cell->setGoalUnit(u->getID(), at);
        row("goal " + where);
    }
}

// A unit standing still: its position and goal on one cell.
static void stand_row(const Scene& s, const Coord3D& pos) {
    const ICoord2D at = cell_of(pos);
    PathfindCell* cell = pf()->getCell(LAYER_GROUND, at.x, at.y);
    if (!cell) return;
    Object* u = pick(s.units);
    const std::string where = num(LAYER_GROUND) + " " + num(at.x) + " " + num(at.y) + " " + num(u->getID());
    cell->setGoalUnit(u->getID(), at);
    row("goal " + where);
    cell->setPosUnit(u->getID(), at);
    row("pos " + where);
}

// A path to patch: one the ground search found, or nodes at cell centres.
static Path* old_path(Scene& s, Coord3D& from) {
    if (chance(80)) {
        Coord3D to;
        for (int n = 0; n < 6; ++n) {
            endpoints(s, from, to);
            if (chance(60)) {
                from = far_point(s);
                to = far_point(s);
            }
            passable(s);
            Placed p;
            place(s, p, from);
            ease(p);
            Path* path = (pf()->*access_member(PfInternal()))(p.m.unit, p.m.set, &from, &to);
            row("find " + mover_text(p.m) + " " + c3(from) + " " + c3(to) + " = " + path_dump(path) + " | " + state(s));
            unplace(p);
            if (path) return path;
        }
        return nullptr;
    }
    from = point(s, false);
    Path* path = newInstance(Path);
    Coord3D at = from;
    for (int n = 1 + below(8); n > 0; --n) {
        path->appendNode(&at, LAYER_GROUND);
        at.x += 10.0f * (below(7) - 3);
        at.y += 10.0f * (below(7) - 3);
        const ICoord2D c = cell_of(at);
        at.x = c.x * 10.0f + (chance(70) ? 5.0f : 0.0f);
        at.y = c.y * 10.0f + (chance(70) ? 5.0f : 0.0f);
    }
    return path;
}

static void patch_rows(Scene& s) {
    Coord3D from;
    Path* old = old_path(s, from);
    if (!old) return;
    std::vector<PathNode*> nodes;
    for (PathNode* n = old->getFirstNode(); n; n = n->getNext()) nodes.push_back(n);
    // patchPath faults on a rejoin point off the map.
    for (PathNode* n : nodes) {
        if (!pf()->getCell(LAYER_GROUND, n->getPosition())) {
            deleteInstance(old);
            return;
        }
    }
    for (int round = 0; round < 3; ++round) {
        // Something stands on the old path.
        for (int k = below(round == 0 ? 2 : 4); k > 0; --k) {
            PathNode* n = nodes[below(static_cast<int>(nodes.size()))];
            Coord3D p = *n->getPosition();
            if (n->getNext() && chance(50)) {
                p.x = (p.x + n->getNext()->getPosition()->x) * 0.5f;
                p.y = (p.y + n->getNext()->getPosition()->y) * 0.5f;
            }
            occupy_cell(s, n->getLayer(), p);
        }
        // The unit has strayed from part of the way along.
        Coord3D at = *nodes[below(static_cast<int>(nodes.size() + 1) / 2)]->getPosition();
        if (chance(85)) {
            const float reach = chance(70) ? 25.0f : 90.0f;
            at.x += between(-reach, reach);
            at.y += between(-reach, reach);
        }
        passable(s);
        Placed p;
        place(s, p, at);
        ease(p);
        const bool blocked = chance(50);
        Path* path = services()->patchPath(p.m.unit, p.m.set, old, blocked);
        tally("patch", path);
        row("patch " + mover_text(p.m) + " " + c3(at) + " " + num(blocked) + " " + nodes_text(old) + " = " +
            path_dump(path) + " | " + state(s));
        if (path) deleteInstance(path);
        unplace(p);
    }
    deleteInstance(old);
}

// Set by the fence rows: a crusher that cannot do more than crush a fence,
// standing on one.
static Object* g_crusher = nullptr;
static Coord3D g_fence;

static void moveaway_row(Scene& s) {
    Coord3D from_a, from_b;
    Path* avoid = old_path(s, from_a);
    if (!avoid) return;
    Path* avoid2 = chance(40) ? old_path(s, from_b) : nullptr;
    // Usually from somewhere on the path to avoid.
    std::vector<PathNode*> nodes;
    for (PathNode* n = avoid->getFirstNode(); n; n = n->getNext()) nodes.push_back(n);
    Coord3D at = *nodes[below(static_cast<int>(nodes.size()))]->getPosition();
    if (chance(60)) {
        at.x += between(-12.0f, 12.0f);
        at.y += between(-12.0f, 12.0f);
    }
    if (chance(8)) at = point(s, true);
    if (chance(20)) by_fence(s, 6.0f, at);
    if (g_crusher) at = g_fence;
    passable(s);
    Placed p;
    place(s, p, at, g_crusher);
    ease(p);
    Object* other = pick(s.units);
    const bool tunneling = chance(20);
    pf()->*access_member(PfTunnel()) = tunneling;
    Path* path = pf()->getMoveAwayFromPath(p.m.unit, other, avoid, avoid2 ? pick(s.units) : nullptr, avoid2);
    tally("moveaway", path);
    row("moveaway " + mover_text(p.m) + " " + c3(at) + " " + num(tunneling) + " " + num(other->getID()) + " " +
        nodes_text(avoid) + " " + (avoid2 ? nodes_text(avoid2) : std::string("0")) + " = " + path_dump(path) + " | " +
        state(s));
    if (path) deleteInstance(path);
    pf()->*access_member(PfTunnel()) = false;
    unplace(p);
    deleteInstance(avoid);
    if (avoid2) deleteInstance(avoid2);
}

static void safe_row(Scene& s) {
    const Coord3D at = chance(70) ? far_point(s) : point(s);
    Coord3D r1 = at, r2 = at;
    r1.x += between(-60.0f, 60.0f);
    r1.y += between(-60.0f, 60.0f);
    r2.x += between(-90.0f, 90.0f);
    r2.y += between(-90.0f, 90.0f);
    if (chance(20)) r2 = r1;
    const float radius = chance(10) ? 5000.0f : chance(20) ? between(0.0f, 20.0f) : between(20.0f, 300.0f);
    Placed p;
    place(s, p, at);
    ease(p);
    const bool tunneling = chance(20);
    pf()->*access_member(PfTunnel()) = tunneling;
    Path* path = services()->findSafePath(p.m.unit, p.m.set, &at, &r1, &r2, radius);
    tally("safe", path);
    row("safe " + mover_text(p.m) + " " + c3(at) + " " + num(tunneling) + " " + c3(r1) + " " + c3(r2) + " " +
        f32(radius) + " = " + path_dump(path) + " | " + state(s));
    if (path) deleteInstance(path);
    pf()->*access_member(PfTunnel()) = false;
    unplace(p);
}

static void attack_row(Scene& s, const std::vector<Object*>& armed) {
    if (armed.empty()) return;
    Object* attacker = armed[below(static_cast<int>(armed.size()))];
    const Weapon* weapon = attacker->getCurrentWeapon();
    if (!weapon) {
        std::fprintf(stderr, "armed unit without a weapon\n");
        std::exit(1);
    }
    Coord3D at = chance(80) ? far_point(s) : point(s);
    if (chance(15)) by_fence(s, 14.0f, at);
    Object* victim = nullptr;
    if (chance(60)) {
        victim = chance(50) || s.blocks.empty() ? pick(s.units) : pick(s.blocks);
        if (victim == attacker) victim = nullptr;
    }
    Coord3D target = chance(50) ? far_point(s) : point(s, chance(10));
    if (chance(50)) {
        const float reach = chance(50) ? 60.0f : 220.0f;
        target = at;
        target.x += between(-reach, reach);
        target.y += between(-reach, reach);
    }
    if (g_crusher) {
        at = g_fence;
        at.x += between(-8.0f, 8.0f);
        at.y += between(-8.0f, 8.0f);
        target = at;
        target.x += between(-90.0f, 90.0f);
        target.y += between(-90.0f, 90.0f);
        victim = nullptr;
    }
    Coord3D victim_was;
    if (victim) {
        victim_was = *victim->getPosition();
        // A unit moves where wanted; a structure keeps its footprint.
        if (!victim->isKindOf(KINDOF_STRUCTURE)) {
            if (chance(15)) target.z = 40.0f;
            *const_cast<Coord3D*>(victim->getPosition()) = target;
        } else {
            target = victim_was;
        }
    }
    const Coord3D victim_at = victim ? *victim->getPosition() : target;
    // The attacker may aim at where the victim was last seen.
    if (victim && chance(20)) {
        target.x += between(-70.0f, 70.0f);
        target.y += between(-70.0f, 70.0f);
    }
    // Units about where the attacker would stop, in the way of a vehicle.
    for (int k = below(8); k > 0; --k) {
        Coord3D back;
        back.set(at.x - target.x, at.y - target.y, 0);
        back.normalize();
        const float out = weapon->getAttackRange(attacker) * between(0.6f, 1.6f);
        Coord3D spot = target;
        spot.x += back.x * out + between(-15.0f, 15.0f);
        spot.y += back.y * out + between(-15.0f, 15.0f);
        if (chance(30)) occupy_cell(s, LAYER_GROUND, spot);
        else stand_row(s, spot);
    }
    Placed p;
    place(s, p, at, attacker);
    ease(p);
    const bool tunneling = chance(25);
    pf()->*access_member(PfTunnel()) = tunneling;

    // Every position the search can ask the weapon about.
    Int radius;
    Bool center;
    radius_center(attacker, radius, center);
    std::vector<Coord3D> asked;
    {
        Coord3D delta;
        const Coord3D aim = victim ? victim_at : target;
        delta.set(aim.x - at.x, aim.y - at.y, 0);
        delta.normalize();
        delta.x *= PATHFIND_CELL_SIZE_F;
        delta.y *= PATHFIND_CELL_SIZE_F;
        for (int i = 1; i < 10; ++i) {
            Coord3D test = at;
            test.x += delta.x * i * 0.5f;
            test.y += delta.y * i * 0.5f;
            asked.push_back(test);
        }
    }
    for (PathfindCell* cell : s.cells) {
        const int k = s.key_of[cell];
        Coord3D centre;
        (pf()->*access_member(PfAdjust()))(k / 1000 % 1000, k % 1000, center, centre,
                                             static_cast<PathfindLayerEnum>(k / 1000000));
        asked.push_back(centre);
    }
    std::string table;
    int entries = 0;
    std::set<std::pair<std::string, std::string>> seen;
    for (const Coord3D& pos : asked) {
        if (!weapon->isGoalPosWithinAttackRange(attacker, &pos, victim, &target)) continue;
        if (!seen.insert({f32(pos.x), f32(pos.y)}).second) continue;
        const bool hidden = pf()->isAttackViewBlockedByObstacle(attacker, pos, victim, target) != 0;
        table += " " + f32(pos.x) + " " + f32(pos.y) + " " + num(hidden);
        ++entries;
    }
    const float distance = weapon->getAttackDistance(attacker, victim, &target);
    const bool above = victim && victim->isSignificantlyAboveTerrain();
    Path* path = services()->findAttackPath(attacker, p.m.set, &at, victim, &target, weapon);
    tally("attack", path);
    row("attack " + mover_text(p.m) + " " + c3(at) + " " + num(tunneling) + " " + num(victim != nullptr) + " " +
        num(above) + " " + c3(victim_at) + " " + c3(target) + " " + f32(distance) + " " + num(entries) + table + " = " + path_dump(path) +
        " | " + state(s));
    if (path) deleteInstance(path);
    pf()->*access_member(PfTunnel()) = false;
    unplace(p);
    if (victim) *const_cast<Coord3D*>(victim->getPosition()) = victim_was;
}

// ---- scenes ---------------------------------------------------------------------

struct Shape {
    int w, h;
    bool strips, rough, walls, bridges;
    int blocks, airfields, units, armed, clusters;
    int barrier;  // a column of water across the whole map, 0 for none
};

static void s6_open(Scene& s, int index, const Shape& shape, std::vector<Object*>& armed) {
    std::fprintf(stderr, "scene %d at %zu rows\n", index, g_rows.size());
    row("scenario " + num(index));
    s.w = shape.w;
    s.h = shape.h;
    s.maxx = s.w * 10.0f;
    s.maxy = s.h * 10.0f;
    S5Terrain& t = *g_terrain;
    t.w = s.w;
    t.h = s.h;
    t.cliff.assign(static_cast<size_t>((s.w + 2) * (s.h + 2)), '0');
    t.water = t.cliff;
    t.height = t.cliff;
    struct Strip { bool across_x; int at, width; };
    std::vector<Strip> strips;
    if (shape.strips) {
        for (int k = 1 + below(2); k > 0; --k) {
            Strip strip;
            strip.across_x = k == 1 || chance(50);
            const int span = strip.across_x ? s.w : s.h;
            strip.width = 2 + below(3);
            strip.at = span / 4 + below(span / 2);
            strips.push_back(strip);
            std::string& table = chance(75) ? t.water : t.cliff;
            for (int i = 0; i < (strip.across_x ? s.h : s.w) + 2; ++i) {
                for (int d = 0; d < strip.width; ++d) {
                    table[t.at(strip.across_x ? strip.at + d : i, strip.across_x ? i : strip.at + d)] = '1';
                }
            }
            if (chance(30)) {  // a ford
                const int y = below((strip.across_x ? s.h : s.w) - 5);
                for (int i = y; i < y + 4; ++i)
                    for (int d = 0; d < strip.width; ++d)
                        table[t.at(strip.across_x ? strip.at + d : i, strip.across_x ? i : strip.at + d)] = '0';
            }
        }
    }
    if (shape.rough) {
        blobs(t.cliff, '1', 1 + below(4), 6);
        blobs(t.water, '1', below(4), 6);
        // The S5 suite's bridge terrain is flat, so bridged maps stay flat.
        for (int v = 1; v <= 5 && !shape.bridges; ++v) blobs(t.height, static_cast<char>('0' + v), below(5), 8);
    }
    for (int j = 0; shape.barrier && j < s.h + 2; ++j) {
        for (int d = 0; d < 3; ++d) t.water[t.at(shape.barrier + d, j)] = '1';
    }
    t.extent.lo.x = 0.0f;
    t.extent.lo.y = 0.0f;
    t.extent.lo.z = 0.0f;
    t.extent.hi.x = s.maxx;
    t.extent.hi.y = s.maxy;
    t.extent.hi.z = 100.0f;
    const float wall_height = 12.0f;
    const_cast<TAiData*>(TheAI->getAiData())->m_wallHeight = wall_height;
    const_cast<TAiData*>(TheAI->getAiData())->m_attackUsesLineOfSight = true;
    pf()->reset();
    row("map " + f32(s.maxx) + " " + f32(s.maxy) + " " + f32(wall_height));
    row("terrain " + num(s.w) + " " + num(s.h) + " " + t.cliff + " " + t.water + " " + t.height);
    g_world->set_frame(100);
    pf()->newMap();

    auto at_random = [&](const std::string& name) {
        return spawn(name, chance(50) ? g_world->team_a : g_world->team_b, coord(0.0f, s.maxx),
                     coord(0.0f, s.maxy), 0.0f, chance(50) ? 0.0f : between(-3.2f, 3.2f));
    };
    for (int n = shape.blocks; n > 0; --n) {
        s.blocks.push_back(at_random(g_blocks[below(static_cast<int>(g_blocks.size()))]));
    }
    for (int n = shape.airfields; n > 0; --n) {
        const std::string name = g_airfields[below(static_cast<int>(g_airfields.size()))];
        Object* field = at_random(name);
        s.blocks.push_back(field);
        // Some overlap, so an airfield's cells touch another structure's.
        if (chance(50)) {
            const Coord3D p = *field->getPosition();
            s.blocks.push_back(spawn(chance(50) ? name : g_blocks[below(12)], g_world->team_a,
                                     p.x + between(-40.0f, 40.0f), p.y + between(-40.0f, 40.0f), 0.0f, 0.0f));
        }
    }
    if (shape.walls) {
        float x = coord(30.0f, s.maxx * 0.4f);
        const float y = coord(30.0f, s.maxy - 30.0f);
        for (int n = 2 + below(4); n > 0; --n) {
            s.walls.push_back(spawn("S5Wall", g_world->team_a, x, y, 0.0f, 0.0f));
            x += 48.0f;
        }
    }
    for (int n = shape.units; n > 0; --n) {
        s.units.push_back(at_random(g_units[below(static_cast<int>(g_units.size()))]));
    }
    for (int n = shape.armed; n > 0; --n) {
        Object* u = at_random(g_armed[below(static_cast<int>(g_armed.size()))]);
        s.units.push_back(u);
        armed.push_back(u);
    }
    g_parked.clear();
    for (Object* field : std::vector<Object*>(s.blocks)) {
        if (!field->isKindOf(KINDOF_FS_AIRFIELD)) continue;
        const Coord3D p = *field->getPosition();
        Object* u = spawn(g_units[below(static_cast<int>(g_units.size()))], g_world->team_a,
                          p.x + between(-8.0f, 8.0f), p.y + between(-8.0f, 8.0f), 0.0f, 0.0f);
        s.units.push_back(u);
        g_parked.push_back({u, field});
    }
    for (Object* o : s.units) {
        if (!s.blocks.empty() && chance(25)) o->getAIUpdateInterface()->ignoreObstacle(pick(s.blocks));
    }
    pf()->reset();
    PathfindCellInfo::allocateCellInfos();
    for (Object* o : s.walls) {
        pf()->addWallPiece(o);
        row("wallpiece " + num(o->getID()));
    }
    if (shape.bridges) {
        g_decks_up = true;
        for (const Strip& strip : strips) {
            const int length = strip.across_x ? s.h : s.w;
            for (int n = 1 + below(3); n > 0; --n) {
                const float across = 10.0f * (2 + below(length - 4)) + (chance(50) ? 5.0f : 0.0f);
                const float lo = 10.0f * strip.at - 10.0f * (1 + below(2));
                const float hi = 10.0f * (strip.at + strip.width) + 10.0f * (1 + below(2));
                add_bridge(s, strip.across_x, lo, hi, across);
            }
        }
        g_decks_up = false;
    }
    s.everything.insert(s.everything.end(), s.blocks.begin(), s.blocks.end());
    s.everything.insert(s.everything.end(), s.walls.begin(), s.walls.end());
    s.everything.insert(s.everything.end(), s.units.begin(), s.units.end());
    for (const std::string& r : s.bridges) row(r);
    std::string order = "order";
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        order += " " + num(o->getID());
        object_row(o);
    }
    row(order);
    set_busy(s);
    ThePartitionManager->update();
    pf()->newMap();
    index_cells(s);
    unit_rows(s);
    goal_object_rows(s);
    idle_row(s);
    occupy(s, shape.clusters);
    connect(s);
    row("setup = " + state(s));
    begin_frame(s, 1000);
    row("setup = " + state(s));
}

static void s6_close(Scene& s) {
    for (Object* o : s.everything) TheGameLogic->destroyObject(o);
    TheGameLogic->processDestroyList();
    ThePartitionManager->update();
    TheTerrainLogic->reset();
    pf()->reset();
    row("end");
}

// One of each search.
static void each_search(Scene& s, const std::vector<Object*>& armed) {
    closest_row(s);
    patch_rows(s);
    moveaway_row(s);
    safe_row(s);
    attack_row(s, armed);
}

static void fence_rows(Scene& s, const std::vector<Object*>& armed) {
    for (int n = 0; n < 6 && by_fence(s, 3.0f, g_fence); ++n) {
        std::vector<Object*> light, light_armed;
        for (Object* o : s.units)
            if (o->getCrusherLevel() == 1) light.push_back(o);
        for (Object* o : armed)
            if (o->getCrusherLevel() == 1) light_armed.push_back(o);
        if (!light.empty()) {
            g_crusher = pick(light);
            moveaway_row(s);
        }
        if (!light_armed.empty()) {
            g_crusher = pick(light_armed);
            attack_row(s, light_armed);
        }
        g_crusher = nullptr;
    }
}

static void s6_scene(int index) {
    Scene s;
    std::vector<Object*> armed;
    const bool large = index % 4 == 3;
    const bool strips = index % 4 == 2 || index % 8 == 7;
    Shape shape;
    shape.w = large ? 78 + below(12) : 20 + below(30);
    shape.h = large ? 72 + below(12) : 18 + below(24);
    shape.strips = strips;
    shape.rough = index % 3 != 0;
    shape.walls = index % 4 == 1;
    shape.bridges = strips;
    shape.blocks = index % 5 == 0 ? below(4) : 3 + below(large ? 30 : 9);
    shape.airfields = index % 2 == 0 ? 1 + below(3) : 0;
    shape.units = 5 + below(4);
    shape.armed = 3 + below(3);
    shape.clusters = index % 5 == 0 ? 4 : 15 + below(large ? 90 : 35);
    shape.barrier = 0;
    s6_open(s, index, shape, armed);
    if (index % 3 == 0) occupy_edges(s, 60);
    const int rounds = large ? 6 : 10;
    for (int n = 0; n < rounds; ++n) {
        each_search(s, armed);
        closest_row(s);
        attack_row(s, armed);
        attack_row(s, armed);
    }
    fence_rows(s, armed);
    // The same calls under the fixed code.
    if (index % 2 == 1) {
        for (int n = 0; n < 30 && !s_useFixedPathfinding; ++n) chain_row(s);
        for (int n = 0; n < rounds / 2; ++n) {
            each_search(s, armed);
            closest_row(s);
        }
    }
    s6_close(s);
}

// A map with more cells than there are records, so allocations fail.
static void s6_exhaust(int index, bool fixed) {
    Scene s;
    std::vector<Object*> armed;
    Shape shape = {196, 170, false, false, false, false, 6, 1, 6, 4, 30, 0};
    s6_open(s, index, shape, armed);
    occupy_edges(s, 40);
    if (fixed) {
        for (int n = 0; n < 40 && !s_useFixedPathfinding; ++n) chain_row(s);
    }
    static const int left[] = {0, 1, 2, 3, 4, 6, 12, 40, 200};
    for (int n : left) {
        // The pool is shared with the occupants' records.
        hoard_row(s, n);
        closest_row(s);
        safe_row(s);
        patch_rows(s);
        moveaway_row(s);
        attack_row(s, armed);
        hoard_row(s, -1);
    }
    // Long searches, to pass the cell limits.
    for (int n = 0; n < 6; ++n) each_search(s, armed);
    s6_close(s);
}

// A long strip of land that ends at water, with the goals beyond it: the cells
// far behind the unit are twice as far from the goal as the nearest one.
static void s6_corridor(int index) {
    Scene s;
    std::vector<Object*> armed;
    Shape shape = {190, 7 + below(6), false, false, false, false, 0, 0, 5, 1, 6, 130};
    s6_open(s, index, shape, armed);
    for (int n = 0; n < 14; ++n) {
        Coord3D from, to;
        from.x = between(900.0f, 1290.0f);
        from.y = between(10.0f, s.maxy - 10.0f);
        from.z = 0.0f;
        to.x = between(1650.0f, 1890.0f);
        to.y = between(10.0f, s.maxy - 10.0f);
        to.z = 0.0f;
        zones().setAllPassable();
        row("allpass");
        Placed p;
        place(s, p, from);
        ease(p);
        static const float mults[] = {0.0f, 1.0f, 25.0f};
        const float mult = mults[below(3)];
        Coord3D raw = to;
        Path* path = services()->findClosestPath(p.m.unit, p.m.set, &from, &raw, FALSE, mult, FALSE);
        tally("closest", path);
        row("closest " + mover_text(p.m) + " " + c3(from) + " " + c3(to) + " 0 " + f32(mult) + " = " +
            path_dump(path) + " " + c3(raw) + " " + num(pf()->*access_member(PfIgnore())) + " | " + state(s));
        if (path) deleteInstance(path);
        unplace(p);
    }
    s6_close(s);
}

int main(int argc, char** argv) {
    if (argc < 2) return 2;
    oracle_wp51::World world;
    g_world = &world;
    make_templates();
    s6_templates();
    world.load_ini(g_ini);
    g_terrain = new S5Terrain;
    TheTerrainLogic = g_terrain;
    Player* computer = world.team_b->getControllingPlayer();
    Player* human = world.team_a->getControllingPlayer();
    if (!computer || !human || computer == human) {
        std::fprintf(stderr, "teams without players\n");
        return 1;
    }
    computer->*access_member(PlayerKind()) = PLAYER_COMPUTER;
    human->*access_member(PlayerKind()) = PLAYER_HUMAN;
    g_state = 0x53365442u;
    for (int index = 0; index < 20; ++index) s6_scene(index);
    s6_exhaust(20, false);
    s6_corridor(21);
    s6_corridor(22);
    s6_exhaust(23, true);
    for (const auto& entry : g_tally) {
        std::fprintf(stderr, "%s %d found %d not\n", entry.first.c_str(), entry.second.first, entry.second.second);
    }

    std::FILE* out = std::fopen(argv[1], "wb");
    if (!out) return 1;
    std::fputs("{\n \"rows\": [", out);
    for (size_t r = 0; r < g_rows.size(); ++r) {
        std::fprintf(out, "%s\n  \"%s\"", r ? "," : "", g_rows[r].c_str());
    }
    std::fputs("\n ]\n}\n", out);
    std::fclose(out);
    std::fprintf(stderr, "%zu rows\n", g_rows.size());
    return 0;
}
