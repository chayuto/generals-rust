//! Gate V2-Golden for targeting and mood (`docs/AI_PORT_PLAN.md`, slice S12):
//! `zh_logic::ai::ai` and `zh_logic::ai::mood` over random crowds.
//!
//! Oracle: `tests/cpp_harness/gen_s12_targeting.cpp`
//! Golden vectors: `docs/golden_vectors/S12_Targeting.json`
//!
//! Every object is a real arena object in the partition grid; its body, contain
//! and stealth modules are stand-ins that answer what the reference's modules
//! answered. What targeting asks of unported code comes from the tape.

use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UnsignedInt, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};
use zh_logic::ai::ai::*;
use zh_logic::ai::mood::*;
use zh_logic::arena::{FacetMask, ModuleContext, ModuleDataId, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::modules::module::{
    AttitudeType, BodyFacet, ContainFacet, StealthFacet, StealthParams,
};
use zh_logic::modules::BehaviorModule;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::object::weapon_set::{CanAttackResult, WeaponTemplateSet};
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::rules::combat_types::DamageType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::SleepyUpdatePhase;
use zh_logic::sim::{
    AttackAbility, RulesRegistry, SimApi, SimContext, SimView, SimWorld,
    PRIVATE_STATUS_EFFECTIVELY_DEAD, PRIVATE_STATUS_OFF_MAP,
};

const JSON: &str = include_str!("../../../docs/golden_vectors/S12_Targeting.json");

// ---------------------------------------------------------------------------
// The vectors: integers, strings, arrays and objects.
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum J {
    N(i64),
    S(String),
    A(Vec<J>),
    O(Vec<(String, J)>),
}

impl J {
    fn parse(s: &[u8], at: &mut usize) -> J {
        let open = s[*at];
        *at += 1;
        match open {
            b'[' | b'{' => {
                let mut items = Vec::new();
                let mut fields = Vec::new();
                let close = if open == b'[' { b']' } else { b'}' };
                while s[*at] != close {
                    if open == b'{' {
                        let J::S(key) = J::parse(s, at) else {
                            panic!("key")
                        };
                        assert_eq!(s[*at], b':');
                        *at += 1;
                        fields.push((key, J::parse(s, at)));
                    } else {
                        items.push(J::parse(s, at));
                    }
                    if s[*at] == b',' {
                        *at += 1;
                    }
                }
                *at += 1;
                if open == b'[' {
                    J::A(items)
                } else {
                    J::O(fields)
                }
            }
            b'"' => {
                let from = *at;
                while s[*at] != b'"' {
                    *at += 1;
                }
                *at += 1;
                J::S(String::from_utf8(s[from..*at - 1].to_vec()).expect("utf8"))
            }
            _ => {
                let from = *at - 1;
                while s[*at].is_ascii_digit() {
                    *at += 1;
                }
                let text = std::str::from_utf8(&s[from..*at]).expect("utf8");
                J::N(text.parse().expect("number"))
            }
        }
    }

    fn get(&self, key: &str) -> &J {
        let J::O(fields) = self else {
            panic!("not an object")
        };
        &fields
            .iter()
            .find(|f| f.0 == key)
            .unwrap_or_else(|| panic!("{key}"))
            .1
    }

    fn list(&self) -> &[J] {
        let J::A(items) = self else {
            panic!("not an array: {self:?}")
        };
        items
    }

    fn n(&self) -> i64 {
        let J::N(n) = self else {
            panic!("not a number: {self:?}")
        };
        *n
    }

    fn nums(&self) -> Vec<i64> {
        self.list().iter().map(J::n).collect()
    }
}

fn real(bits: i64) -> Real {
    Real::from_bits(bits as u32)
}

fn flag(v: i64) -> Bool {
    Bool::from(v != 0)
}

// ---------------------------------------------------------------------------
// Stand-ins
// ---------------------------------------------------------------------------

struct BodyProbe(DamageInfo);

impl BodyFacet for BodyProbe {
    fn attempt_damage(&mut self, _: &mut dyn SimApi, _: ModuleContext, _: &mut DamageInfo) {}
    fn get_health(&self) -> Real {
        1.0
    }
    fn get_last_damage_info(&self) -> Option<&DamageInfo> {
        Some(&self.0)
    }
    fn get_last_damage_timestamp(&self) -> Option<UnsignedInt> {
        Some(0)
    }
}

#[derive(Default)]
struct ContainProbe {
    count: UnsignedInt,
    stealth_units: UnsignedInt,
    garrisonable: Bool,
    passengers_fire: Bool,
    apparent: [Int; 3],
    items: Vec<ObjectId>,
}

impl ContainFacet for ContainProbe {
    fn get_contain_count(&self, _: &dyn SimView, _: ModuleContext) -> UnsignedInt {
        self.count
    }
    fn is_garrisonable(&self, _: &dyn SimView, _: ModuleContext) -> Bool {
        self.garrisonable
    }
    fn get_contained_items<'a>(&'a self, _: &'a dyn SimView, _: ModuleContext) -> &'a [ObjectId] {
        &self.items
    }
    fn add_to_contain(&mut self, _: &mut dyn SimApi, _: ModuleContext, _: ObjectSlot) {}
    fn remove_from_contain(
        &mut self,
        _: &mut dyn SimApi,
        _: ModuleContext,
        _: ObjectSlot,
        _: Bool,
    ) {
    }
    fn apparent_controlling_player_index(
        &self,
        _: &dyn SimView,
        _: ModuleContext,
        observer: Int,
    ) -> Option<Int> {
        Some(self.apparent[observer as usize]).filter(|&p| p >= 0)
    }
    fn is_passenger_allowed_to_fire(
        &self,
        _: &dyn SimView,
        _: ModuleContext,
        id: ObjectId,
    ) -> Bool {
        assert_eq!(id, ObjectId::INVALID);
        self.passengers_fire
    }
    fn get_stealth_units_contained(&self) -> UnsignedInt {
        self.stealth_units
    }
}

struct StealthProbe(StealthParams);

impl StealthFacet for StealthProbe {
    fn stealth_params(&self) -> Option<StealthParams> {
        Some(self.0)
    }
}

macro_rules! probe_module {
    ($probe:ty, $as:ident, $facet:ty) => {
        impl BehaviorModule for $probe {
            fn module_name_key(&self) -> NameKeyType {
                NameKeyType::default()
            }
            fn xfer(&mut self, _: &mut dyn Xfer, _: &dyn SimView) -> Result<(), XferError> {
                Ok(())
            }
            fn $as(&self) -> Option<&$facet> {
                Some(self)
            }
        }
    };
}

probe_module!(BodyProbe, as_body, dyn BodyFacet);
probe_module!(ContainProbe, as_contain, dyn ContainFacet);
probe_module!(StealthProbe, as_stealth, dyn StealthFacet);

/// The harness's line of sight: a hash of the end points. The vectors also count
/// the calls, which is not compared: unported code behind the tape asks too.
struct LosTerrain;

impl ThingTerrainAccess for LosTerrain {
    fn get_ground_height(&self, _: Real, _: Real, _: Option<&mut Coord3D>) -> Real {
        0.0
    }
    fn align_on_terrain(
        &self,
        _: Real,
        _: &Coord3D,
        _: Bool,
        _: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        PathfindLayerEnum::Ground
    }
    fn is_underwater(&self, _: Real, _: Real, _: Option<&mut Real>, _: Option<&mut Real>) -> Bool {
        FALSE
    }
    fn is_clear_line_of_sight(&self, pos: &Coord3D, other: &Coord3D) -> Bool {
        let mut h: u32 = 2_166_136_261;
        for v in [pos.x, pos.y, pos.z, other.x, other.y, other.z] {
            h ^= v.to_bits();
            h = h.wrapping_mul(16_777_619);
        }
        Bool::from((h >> 7) % 4 != 0)
    }
}

struct Tape {
    objects: Vec<ObjectSlot>,
    me: usize,
    attack: Vec<i64>,
    within: Vec<i64>,
    blocked: Vec<i64>,
    fog: Vec<i64>,
    team: i64,
}

impl Tape {
    fn index(&self, object: ObjectSlot) -> usize {
        self.objects
            .iter()
            .position(|&o| o == object)
            .expect("object")
    }

    fn pair(&self, me: ObjectSlot, other: ObjectSlot) -> usize {
        assert_eq!(self.index(me), self.me, "asked for another unit");
        self.index(other)
    }
}

impl TargetingSeams for Tape {
    fn able_to_attack_specific_object(
        &mut self,
        _: &dyn SimView,
        me: ObjectSlot,
        other: ObjectSlot,
    ) -> CanAttackResult {
        match self.attack[self.pair(me, other)] {
            0 => CanAttackResult::NotPossible,
            1 => CanAttackResult::InvalidShot,
            2 => CanAttackResult::PossibleAfterMoving,
            3 => CanAttackResult::Possible,
            r => panic!("result {r}"),
        }
    }
    fn within_attack_range(&mut self, _: &dyn SimView, me: ObjectSlot, other: ObjectSlot) -> Bool {
        flag(self.within[self.pair(me, other)])
    }
    fn view_blocked_by_obstacle(
        &mut self,
        _: &dyn SimView,
        me: ObjectSlot,
        other: ObjectSlot,
    ) -> Bool {
        flag(self.blocked[self.pair(me, other)])
    }
    fn is_clear_of_fog(&mut self, sim: &dyn SimView, other: ObjectSlot, player: Int) -> Bool {
        let mine = sim.object(self.objects[self.me]).expect("me").team_id;
        assert_eq!(player, sim.player_index(mine));
        flag(self.fog[self.index(other)])
    }
    fn team_target_object(&mut self, _: &dyn SimView, me: ObjectSlot) -> Option<ObjectSlot> {
        assert_eq!(self.index(me), self.me);
        usize::try_from(self.team).ok().map(|i| self.objects[i])
    }
}

// ---------------------------------------------------------------------------
// World
// ---------------------------------------------------------------------------

const STATUS: [ObjectStatusType; 6] = [
    ObjectStatusType::Stealthed,
    ObjectStatusType::Detected,
    ObjectStatusType::Disguised,
    ObjectStatusType::Repulsor,
    ObjectStatusType::IsUsingAbility,
    ObjectStatusType::NoAttack,
];

/// The harness's weapons, as far as their ranges go.
const WEAPONS: [&str; 3] = [
    "Weapon S12_Gun\n  AttackRange = 120\nEnd",
    "Weapon S12_Short\n  AttackRange = 45\n  MinimumAttackRange = 10\nEnd",
    "Weapon S12_AA\n  AttackRange = 90\nEnd",
];

/// The weapon slots of each template, as indices into [`WEAPONS`].
fn weapons_of(name: &str) -> &'static [usize] {
    match name {
        "S12_Tank" | "S12_Defense" | "S12_Humvee" | "S12_Inert" | "S12_BunkerDefense" => &[0],
        "S12_Rifle" | "S12_Stinger" | "S12_Spy" | "S12_Sneak" => &[1],
        "S12_Flyer" => &[2, 0],
        _ => &[],
    }
}

fn arm(sim: &mut SimContext, object: ObjectSlot, name: &str) {
    let mut set = WeaponTemplateSet::default();
    for (slot, &weapon) in weapons_of(name).iter().enumerate() {
        let mut template = WeaponTemplate::new();
        template.parse_ini_block(WEAPONS[weapon]).expect("weapon");
        set.templates[slot] = Some(template);
    }
    let now = sim.world.frame;
    let record = sim.world.objects.get_mut(object).expect("object");
    record.weapon_set.init_from_template_set(&set, now);
}

fn build(sim: &mut SimContext, crowd: &J, names: &[&str]) -> Vec<ObjectSlot> {
    sim.world = SimWorld::new();
    sim.world.frame = crowd.get("frame").n() as UnsignedInt;
    let human = crowd.get("human").nums();
    let relations = crowd.get("rel").nums();
    for (p, &is_human) in human.iter().enumerate() {
        let team = TeamId(p as Int);
        sim.world.team_players.insert(team, p as Int);
        sim.world.player_default_teams.insert(p as Int, team);
        sim.world.set_team_human(team, flag(is_human));
        for q in 0..3 {
            if p != q {
                let r = match relations[p * 3 + q] {
                    0 => Relationship::Enemies,
                    1 => Relationship::Neutral,
                    2 => Relationship::Allies,
                    r => panic!("relationship {r}"),
                };
                sim.world.set_player_relation(p as Int, q as Int, r);
            }
        }
    }
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );

    let records: Vec<Vec<i64>> = crowd.get("objects").list().iter().map(J::nums).collect();
    let mut objects = Vec::new();
    for r in &records {
        let template = sim
            .rules
            .thing_factory
            .find_template_id(names[r[0] as usize])
            .expect("template");
        let o = sim.spawn_object(template, TeamId(r[1] as Int), Coord3D::default(), 0.0);
        let pos = Coord3D::new(real(r[2]), real(r[3]), real(r[4]));
        sim.set_object_position(o, &pos);
        arm(sim, o, names[r[0] as usize]);
        objects.push(o);
    }
    let ids: Vec<ObjectId> = objects
        .iter()
        .map(|&o| sim.world.objects.id_of(o).expect("id"))
        .collect();

    for (i, r) in records.iter().enumerate() {
        let container = usize::try_from(r[8]).ok().map(|c| objects[c]);
        let record = sim.world.objects.get_mut(objects[i]).expect("object");
        for (bit, status) in STATUS.iter().enumerate() {
            if r[5] & (1 << bit) != 0 {
                record.status.set(status.as_i32(), 1);
            }
        }
        record.private_status = 0;
        if r[6] != 0 {
            record.private_status |= PRIVATE_STATUS_EFFECTIVELY_DEAD;
        }
        if r[7] != 0 {
            record.private_status |= PRIVATE_STATUS_OFF_MAP;
        }
        record.contained_by = container;
        assert_eq!(record.vision_range.to_bits(), r[11] as u32, "vision range");
        if r[9] == 0 {
            sim.set_in_partition(objects[i], FALSE);
        }
        sim.world.attack_abilities.insert(
            ids[i],
            AttackAbility {
                able: flag(r[10]),
                specific_object: CanAttackResult::NotPossible,
                weapon_against_target: CanAttackResult::NotPossible,
            },
        );
    }

    let attach = |sim: &mut SimContext, i: usize, mask, module| {
        sim.world.attach_module(
            objects[i],
            NameKeyType::default(),
            ModuleDataId::default(),
            mask,
            SleepyUpdatePhase::Normal,
            module,
        );
    };
    for (i, d) in crowd.get("damage").list().iter().enumerate() {
        let d = d.nums();
        let mut info = DamageInfo::default();
        info.input.damage_type = DamageType::from_i32(d[0] as Int).expect("damage type");
        info.input.source_id = usize::try_from(d[1]).map_or(ObjectId::INVALID, |s| ids[s]);
        attach(
            sim,
            i,
            FacetMask::BODY,
            Box::new(BodyProbe(info)) as Box<dyn BehaviorModule>,
        );
    }
    for c in crowd.get("contain").list() {
        let c = c.nums();
        let probe = ContainProbe {
            count: c[1] as UnsignedInt,
            stealth_units: c[2] as UnsignedInt,
            garrisonable: flag(c[3]),
            passengers_fire: flag(c[4]),
            apparent: [c[5] as Int, c[6] as Int, c[7] as Int],
            items: c[8..].iter().map(|&item| ids[item as usize]).collect(),
        };
        attach(sim, c[0] as usize, FacetMask::CONTAIN, Box::new(probe));
    }
    for s in crowd.get("stealth").list() {
        let s = s.nums();
        let probe = StealthProbe(StealthParams {
            stealth_delay: 0,
            stealth_level: 0,
            order_idle_enemies_on_reveal: FALSE,
            disguised_template: (s[1] != 0).then_some(Default::default()),
            disguised_player_index: s[2] as Int,
            granted_by_special_power: flag(s[3]),
            temporary_grant: FALSE,
        });
        attach(
            sim,
            s[0] as usize,
            FacetMask::STEALTH_UPDATE,
            Box::new(probe),
        );
    }
    sim.update_partition();
    objects
}

fn ai_data(values: &[i64]) -> AiData {
    AiData {
        guard_inner_modifier_ai: real(values[0]),
        guard_outer_modifier_ai: real(values[1]),
        guard_inner_modifier_human: real(values[2]),
        guard_outer_modifier_human: real(values[3]),
        alert_range_modifier: real(values[4]),
        aggressive_range_modifier: real(values[5]),
        attack_priority_distance_modifier: real(values[6]),
        force_idle_frames_count: values[7] as UnsignedInt,
        enable_repulsors: flag(values[8]),
        attack_uses_line_of_sight: flag(values[9]),
        attack_ignore_insignificant_buildings: flag(values[10]),
        ..AiData::default()
    }
}

fn attitude(value: i64) -> AttitudeType {
    match value {
        -2 => AttitudeType::Sleep,
        -1 => AttitudeType::Passive,
        0 => AttitudeType::Normal,
        1 => AttitudeType::Alert,
        2 => AttitudeType::Aggressive,
        3 => AttitudeType::Invalid,
        a => panic!("attitude {a}"),
    }
}

fn index_of(objects: &[ObjectSlot], found: Option<ObjectSlot>) -> i64 {
    found.map_or(-1, |o| {
        objects.iter().position(|&x| x == o).expect("found") as i64
    })
}

// ---------------------------------------------------------------------------
// The test
// ---------------------------------------------------------------------------

/// `TAiData::TAiData`.
#[test]
fn ai_data_starts_as_the_constructor_leaves_it() {
    let data = AiData::default();
    for modifier in [
        data.guard_inner_modifier_ai,
        data.guard_outer_modifier_ai,
        data.guard_inner_modifier_human,
        data.guard_outer_modifier_human,
        data.alert_range_modifier,
        data.aggressive_range_modifier,
        data.attack_priority_distance_modifier,
    ] {
        assert_eq!(modifier.to_bits(), 0);
    }
    assert_eq!(data.force_idle_frames_count, 1);
    assert_eq!(data.enable_repulsors, FALSE);
    assert_eq!(data.attack_uses_line_of_sight, TRUE);
    assert_eq!(data.attack_ignore_insignificant_buildings, FALSE);
}

#[test]
fn targeting_and_mood_match_the_reference() {
    let lines: Vec<J> = JSON
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with('{'))
        .map(|l| J::parse(l.as_bytes(), &mut 0))
        .collect();
    let (header, crowds) = lines.split_first().expect("header");
    assert_eq!(crowds.len(), 1000);

    let J::S(hex) = header.get("ini") else {
        panic!("ini")
    };
    let ini: Vec<u8> = (0..hex.len() / 2)
        .map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).expect("hex"))
        .collect();
    let J::S(names) = header.get("names") else {
        panic!("names")
    };
    let names: Vec<&str> = names.split(' ').collect();

    let mut sim = SimContext::new();
    let mut rules = RulesRegistry::new();
    let mut reader = IniReader::new("s12.ini", &ini, IniLoadType::Overwrite);
    rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    sim.rules = rules;
    sim.terrain = Box::new(LosTerrain);

    let mut counts = [0_usize; 8];
    for crowd in crowds {
        let n = crowd.get("n").n();
        let objects = build(&mut sim, crowd, &names);
        let data = ai_data(&crowd.get("ai").nums());
        let prio = crowd.get("prio").list();
        let priorities = AttackPriorityInfo {
            name: "S12".to_owned(),
            default_priority: prio[0].n() as Int,
            priorities: prio[1..]
                .iter()
                .map(|p| {
                    let p = p.nums();
                    (names[p[0] as usize].to_owned(), p[1] as Int)
                })
                .collect(),
        };

        for asker in crowd.get("askers").list() {
            let me_index = asker.get("me").n() as usize;
            let me = objects[me_index];
            let mut tape = Tape {
                objects: objects.clone(),
                me: me_index,
                attack: asker.get("attack").nums(),
                within: asker.get("within").nums(),
                blocked: asker.get("blocked").nums(),
                fog: asker.get("fog").nums(),
                team: asker.get("team").n(),
            };
            assert_eq!(
                largest_weapon_range(&sim, me).to_bits(),
                asker.get("largest").n() as u32,
                "crowd {n} unit {me_index}: largest weapon range"
            );
            let u = asker.get("unit").nums();
            let mut state = MoodState::default();
            let unit = (!u.is_empty()).then(|| {
                state.next_mood_check_time = u[9] as UnsignedInt;
                state.randomly_offset_mood_check = flag(u[10]);
                MoodUnit {
                    has_state_machine: flag(u[0]),
                    attitude: attitude(asker.get("attitude").n()),
                    valid_surfaces: u[2] as Int,
                    has_primary_turret: flag(u[3]),
                    is_attacking: flag(u[4]),
                    auto_acquire_enemies_when_idle: u[6] as UnsignedInt,
                    mood_attack_check_rate: u[7] as UnsignedInt,
                    attack_info: (u[8] == 2).then_some(&priorities),
                }
            });
            if let Some(unit) = &unit {
                assert_eq!(
                    mood_matrix_value(&sim, me, unit),
                    u[1] as UnsignedInt,
                    "crowd {n} unit {me_index}: mood matrix"
                );
            }
            let is_idle = u.get(5).copied().unwrap_or(0);

            for (q, query) in asker.get("queries").list().iter().enumerate() {
                let f = query.list();
                let v = |i: usize| f[i].n();
                let at = format!("crowd {n} unit {me_index} query {q} kind {}", v(0));
                counts[v(0) as usize] += 1;
                match v(0) {
                    0 => {
                        let reject = f[5].nums();
                        let mut seen = Vec::new();
                        let mut filter = |o: ObjectSlot| {
                            let i = index_of(&objects, Some(o));
                            seen.push(i);
                            !reject.contains(&i)
                        };
                        let found = find_closest_enemy(
                            &sim,
                            &mut tape,
                            &data,
                            me,
                            real(v(1)),
                            v(2) as UnsignedInt,
                            (v(3) == 2).then_some(&priorities),
                            (v(4) != 0)
                                .then_some(&mut filter as &mut dyn FnMut(ObjectSlot) -> bool),
                        );
                        assert_eq!(seen, f[8].nums(), "{at}: filtered objects");
                        assert_eq!(index_of(&objects, found), v(6), "{at}: enemy");
                    }
                    1 => {
                        let found =
                            find_closest_ally(&sim, &mut tape, me, real(v(1)), v(2) as UnsignedInt);
                        assert_eq!(index_of(&objects, found), v(3), "{at}: ally");
                    }
                    2 => {
                        let found = find_closest_repulsor(&sim, &data, me, real(v(1)));
                        assert_eq!(index_of(&objects, found), v(2), "{at}: repulsor");
                    }
                    3 => {
                        let mood = unit.as_ref().map(|u| mood_matrix_value(&sim, me, u));
                        let range =
                            adjusted_vision_range(&sim, &mut tape, &data, me, mood, v(1) as Int);
                        assert_eq!(range.to_bits(), v(2) as u32, "{at}: factors {}", v(1));
                    }
                    4 => {
                        let unit = unit.as_ref().expect("unit");
                        assert_eq!(i64::from(mood_matrix_value(&sim, me, unit)), v(1), "{at}");
                        use MoodMatrixAction as Action;
                        let actions = [
                            Action::Idle,
                            Action::Move,
                            Action::Attack,
                            Action::AttackMove,
                        ];
                        for (i, action) in actions.into_iter().enumerate() {
                            let got = mood_matrix_action_adjustment(&sim, me, unit, action);
                            assert_eq!(i64::from(got), v(2 + i), "{at}: {action:?}");
                        }
                        let stealthed = can_auto_acquire_while_stealthed(&sim, me, unit);
                        assert_eq!(stealthed, flag(v(6)), "{at}: stealthed");
                    }
                    5 => {
                        let arg = v(2) as UnsignedInt;
                        let now = sim.world.frame;
                        match v(1) {
                            0 => state.wake_up_and_attempt_to_target(flag(is_idle), now),
                            1 => state.reset_next_mood_check_time(now, &data),
                            2 => state.set_next_mood_check_time(arg),
                            _ => panic!("{at}: operation"),
                        }
                        assert_eq!(i64::from(state.next_mood_check_time), v(3), "{at}: next");
                        assert_eq!(state.randomly_offset_mood_check, flag(v(4)), "{at}: offset");
                    }
                    6 => {
                        let unit = unit.as_ref().expect("unit");
                        sim.world.rng.init_random_with_seed(v(3) as UnsignedInt);
                        let found = next_mood_target(
                            &mut sim,
                            &mut tape,
                            &data,
                            me,
                            unit,
                            &mut state,
                            flag(v(1)),
                            flag(v(2)),
                        );
                        let at = format!("{at} (by ai {}, idle {})", v(1), v(2));
                        assert_eq!(index_of(&objects, found), v(4), "{at}: target");
                        assert_eq!(i64::from(state.next_mood_check_time), v(5), "{at}: next");
                        assert_eq!(state.randomly_offset_mood_check, flag(v(6)), "{at}: offset");
                        let crc = sim.world.rng.get_game_logic_random_seed_crc();
                        assert_eq!(i64::from(crc), v(7), "{at}: random seed");
                    }
                    7 => {
                        state.next_mood_check_time = v(1) as UnsignedInt;
                        state.randomly_offset_mood_check = flag(v(2));
                    }
                    k => panic!("{at}: kind {k}"),
                }
            }
        }
    }
    assert!(counts[0] > 9000 && counts[6] > 15000, "{counts:?}");
}
