//! The state a `Player` holds for all of its teams, and the power brown-out, against
//! the reference (`tests/cpp_harness/gen_player_holdings.cpp`). Every row after
//! every operation is compared exactly.

use std::fmt::Write as _;

use zh_common::core::dict::Dict;
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{Relationship, TeamId, FALSE};
use zh_logic::arena::{FacetMask, ModuleSlot, ObjectSlot};
use zh_logic::rules::disabled_types::{DisabledType, DISABLED_COUNT};
use zh_logic::rules::kind_of::{KindOfMaskType, KindOfType};
use zh_logic::rules::science::{ScienceTextLookup, ScienceType};
use zh_logic::sim::{SimApi, SimContext, SimView, SimWorld};

const GOLDEN: &str = include_str!("../../../docs/golden_vectors/PlayerHoldings.json");

const KINDS: [&str; 7] = [
    "G_Plant", "G_Eater", "G_Gun", "G_Radar", "G_Proof", "G_Centre", "G_Solar",
];
const SCIENCES: [&str; 3] = ["SCI_A", "SCI_B", "SCI_C"];
const POWERS: [&str; 4] = ["SP_Plain", "SP_Sci", "SP_Shared", "SP_SharedSci"];
const UPGRADES: [&str; 2] = ["PU_Radar", "PU_Power"];
const PERCENTS: [f32; 5] = [-0.1, 0.25, -0.33, 0.0, -0.0];
const HANDICAPS: [f32; 4] = [1.0, 0.75, 1.5, 0.4];
const HANDICAP_KEYS: [&str; 4] = [
    "HANDICAP_BUILDCOST_GENERIC",
    "HANDICAP_BUILDCOST_BUILDINGS",
    "HANDICAP_BUILDTIME_GENERIC",
    "HANDICAP_BUILDTIME_BUILDINGS",
];
/// The harness's team labels: the default teams of players 1 and 2, then a second
/// team of each.
const TEAMS: [(i32, i32); 4] = [(1, 1), (2, 2), (3, 1), (4, 2)];

struct Vector {
    base: u32,
    ops: Vec<[i64; 5]>,
    rows: Vec<String>,
}

fn vectors() -> Vec<Vector> {
    let mut out: Vec<Vector> = Vec::new();
    for line in GOLDEN.lines() {
        if line.starts_with("{\"setup\"") {
            let ops = line.split_once("\"ops\":[").expect("ops").1;
            let ops = ops.split_once("],\"rows\"").expect("rows").0;
            let ops = ops
                .split(',')
                .map(|op| {
                    let mut words = op.trim_matches('"').split(' ');
                    std::array::from_fn(|_| words.next().expect("word").parse().expect("number"))
                })
                .collect();
            out.push(Vector {
                base: get_str(line, "setup").expect("setup").parse().expect("base"),
                ops,
                rows: Vec::new(),
            });
        } else if let Some(row) = line.strip_prefix('"') {
            let end = row.find('"').expect("end of row");
            // Lines before the first vector are the headers.
            if let Some(vector) = out.last_mut() {
                vector.rows.push(row[..end].to_owned());
            }
        }
    }
    out
}

fn header(name: &str) -> String {
    let line = GOLDEN
        .lines()
        .find(|l| l.starts_with(&format!("\"{name}\":")))
        .expect(name);
    get_str(line, name).expect(name).replace("\\n", "\n")
}

fn reader(text: &str) -> IniReader<'_> {
    IniReader::new("holdings.ini", text.as_bytes(), IniLoadType::Overwrite)
}

struct NoText;

impl ScienceTextLookup for NoText {
    fn fetch_label(&mut self, _label: &str) -> String {
        String::new()
    }
}

fn kind_mask(which: usize) -> KindOfMaskType {
    let mut mask = KindOfMaskType::default();
    if which == 0 || which == 3 {
        mask.set(KindOfType::Structure as i32, 1);
    }
    if which == 1 {
        mask.set(KindOfType::Vehicle as i32, 1);
    }
    if which == 2 || which == 3 {
        mask.set(KindOfType::Powered as i32, 1);
    }
    mask
}

struct Trial {
    sim: SimContext,
    objects: Vec<Option<ObjectSlot>>,
}

impl Trial {
    fn new(base: u32) -> Self {
        let mut sim = SimContext::new();
        let rules = &mut sim.rules;
        rules
            .science_store
            .load_from_ini(
                &mut reader(&header("sciences")),
                &mut rules.name_keys,
                &mut NoText,
            )
            .expect("sciences");
        rules
            .special_power_store
            .load_from_ini(
                &mut reader(&header("powers")),
                &rules.science_store,
                &mut rules.name_keys,
            )
            .expect("powers");
        rules
            .upgrade_center
            .load_from_ini(&mut reader(&header("upgrades")), &mut rules.name_keys)
            .expect("upgrades");
        rules
            .load_thing_templates_from_ini(&mut reader(&header("objects")))
            .expect("objects");
        rules.thing_factory.post_process_load();
        rules.global_data.low_energy_penalty_modifier = 0.4;
        rules.global_data.min_low_energy_production_speed = 0.2;
        rules.global_data.max_low_energy_production_speed = 0.8;
        rules.global_data.multiple_factory = 0.85;
        sim.world = SimWorld::new();
        sim.world.frame = base;
        sim.init_partition(
            20.0,
            Region3D {
                lo: Coord3D::new(0.0, 0.0, 0.0),
                hi: Coord3D::new(1000.0, 1000.0, 100.0),
            },
        );
        sim.world.local_player_index = 5;
        for player in [1, 2] {
            sim.world
                .player_default_teams
                .insert(player, TeamId(player));
        }
        for (team, player) in TEAMS {
            sim.add_player_team(TeamId(team), player);
        }
        for i in 1..3 {
            for j in 1..3 {
                let relation = if i == j {
                    Relationship::Allies
                } else {
                    Relationship::Enemies
                };
                sim.world.player_relationships.insert((i, j), relation);
            }
        }
        Self {
            sim,
            objects: Vec::new(),
        }
    }

    fn object(&self, at: i64) -> ObjectSlot {
        self.objects[at as usize].expect("object")
    }

    fn power_module(&self, object: i64, nth: i64) -> ModuleSlot {
        let record = self.sim.world.objects.get(self.object(object)).expect("object");
        record
            .behaviors
            .iter()
            .filter(|(_, facets)| facets.contains(FacetMask::SPECIAL_POWER))
            .nth(nth as usize)
            .expect("special power")
            .0
    }

    fn science(&mut self, which: i64) -> ScienceType {
        self.sim
            .rules
            .science_store
            .get_science_from_internal_name(SCIENCES[which as usize], &mut self.sim.rules.name_keys)
    }

    fn power_id(&self, which: i64) -> u32 {
        self.sim
            .rules
            .special_power_store
            .find_special_power_template(POWERS[which as usize])
            .expect("power")
            .id()
    }

    /// One operation of the harness; the codes are listed there.
    fn run(&mut self, [code, a, b, c, frame]: [i64; 5]) -> u32 {
        self.sim.world.frame = frame as u32;
        let team = TeamId(TEAMS.get(a as usize).map_or(0, |t| t.0));
        let mut ret = 0;
        match code {
            0 => {
                let template = self
                    .sim
                    .rules
                    .thing_factory
                    .find_template_id(KINDS[b as usize])
                    .expect("template");
                let slot = self.sim.factory_new_object(template, team);
                self.objects.push(Some(slot));
            }
            1 => {
                self.sim.destroy_object(self.object(a));
                self.sim.process_destroy_list();
                self.objects[a as usize] = None;
            }
            2 => self
                .sim
                .set_object_team(self.object(a), TeamId(TEAMS[b as usize].0)),
            3 => self.sim.grant_player_upgrade(team, UPGRADES[b as usize]),
            4 => self.sim.add_radar(team, u8::from(b != 0)),
            5 => self.sim.remove_radar(team, u8::from(b != 0)),
            6 => self.sim.sabotage_power(team, b as u32),
            7 => {
                for player in 0..3 {
                    self.sim.expire_power_sabotage(player);
                }
            }
            8 => {
                let o = self.object(a);
                if b != 0 {
                    self.sim.set_disabled(o, DisabledType::Hacked);
                } else {
                    ret = u32::from(self.sim.clear_disabled(o, DisabledType::Hacked) != FALSE);
                }
            }
            9 => self
                .sim
                .add_cost_modifier(team, &kind_mask(b as usize), PERCENTS[c as usize]),
            10 => {
                if b >= 0 {
                    self.sim.remove_cost_modifier(
                        team,
                        &kind_mask(b as usize),
                        PERCENTS[c as usize],
                    );
                }
            }
            11 => {
                let player = self.sim.player_of_team_mut(team).expect("player");
                player.can_build_units = b & 1 != 0;
                player.can_build_base = b & 2 != 0;
            }
            12 => {
                let mut dict = Dict::new();
                let key = self
                    .sim
                    .rules
                    .name_keys
                    .name_to_key(HANDICAP_KEYS[b as usize]);
                dict.set_real(key, HANDICAPS[c as usize]);
                let mut handicap = self.sim.player_of_team(team).expect("player").handicap.clone();
                handicap.read_from_dict(&dict, &mut self.sim.rules.name_keys);
                self.sim.player_of_team_mut(team).expect("player").handicap = handicap;
            }
            13 => {
                let key = self
                    .sim
                    .rules
                    .name_keys
                    .name_to_key(KINDS[b as usize % KINDS.len()]);
                let player = self.sim.player_of_team_mut(team).expect("player");
                let changes = if (b as usize) < KINDS.len() {
                    &mut player.production_cost_changes
                } else {
                    &mut player.production_time_changes
                };
                changes.insert(key, PERCENTS[c as usize]);
            }
            14 => {
                let science = self.science(b);
                ret = u32::from(self.sim.grant_player_science(team, science) != FALSE);
            }
            15 => self.sim.apply_cash_bounty_percent(team, PERCENTS[b as usize]),
            16 => {
                let attacker = self.sim.world.objects.id_of(self.object(b)).expect("id");
                self.sim.record_attacked_by(self.object(a), attacker);
            }
            17..=22 => {
                let module = self.power_module(a, b);
                ret = self
                    .sim
                    .dispatch_module(module, |m, sim, ctx| {
                        let power = m.as_special_power_mut().expect("special power");
                        match code {
                            17 => u32::from(power.is_ready(sim, ctx) != FALSE),
                            18 => power.get_ready_frame(sim, ctx),
                            19 => power.get_percent_ready(sim, ctx).to_bits(),
                            20 => {
                                power.start_power_recharge(sim, ctx);
                                0
                            }
                            21 => {
                                power.on_special_power_creation(sim, ctx);
                                0
                            }
                            _ => {
                                power.pause_countdown(sim, ctx, u8::from(c != 0));
                                0
                            }
                        }
                    })
                    .expect("dispatch");
            }
            23 => {
                let id = self.power_id(b);
                let at = (frame + c) as u32;
                self.sim
                    .express_shared_special_power_ready_frame(team, id, at);
            }
            24 => {
                let id = self.power_id(b);
                // The reference's call returns nothing, and reading would not
                // change a timer that now exists.
                let _ = self.sim.reset_shared_special_power_ready_frame(team, id);
            }
            25 => {
                let id = self.power_id(b);
                ret = self.sim.shared_special_power_ready_frame(team, id);
            }
            other => panic!("unknown op {other}"),
        }
        ret
    }

    fn player_row(&mut self, index: i32, out: &mut String) {
        let sciences: Vec<ScienceType> = (0..SCIENCES.len() as i64)
            .map(|i| self.science(i))
            .collect();
        let frame = self.sim.world.frame;
        let player = self.sim.world.players.player(index).expect("player");
        let energy = self
            .sim
            .world
            .player_energy
            .get(&index)
            .cloned()
            .unwrap_or_default();
        let _ = write!(
            out,
            "{},{},{},{} {},{},{},{} t",
            player.radar_count,
            player.disable_proof_radar_count,
            u8::from(player.radar_disabled),
            u8::from(player.has_radar()),
            energy.get_production(frame),
            energy.get_consumption(),
            u8::from(energy.has_sufficient_power(frame) != FALSE),
            energy.get_power_sabotaged_till_frame(),
        );
        for timer in &player.special_power_timers {
            let _ = write!(out, "{}:{};", timer.template_id, timer.ready_frame);
        }
        out.push_str(" s");
        for held in &player.sciences {
            for (i, science) in sciences.iter().enumerate() {
                if science == held {
                    let _ = write!(out, "{i}");
                }
            }
        }
        out.push_str(" c");
        for change in &player.production_changes {
            for i in 0..4 {
                if change.kind_of == kind_mask(i) {
                    let _ = write!(out, "{i}");
                }
            }
            let _ = write!(out, ":{:08x}:{};", change.percent.to_bits(), change.refs);
        }
        let _ = write!(
            out,
            " b{:08x} {}{} ",
            player.cash_bounty_percent.to_bits(),
            u8::from(player.can_build_units),
            u8::from(player.can_build_base),
        );
        for by in &player.attacked_by[..3] {
            out.push(if *by { '1' } else { '0' });
        }
        let _ = write!(out, ",{}", player.attacked_frame);
    }

    fn row(&mut self) -> String {
        let mut out = String::new();
        for player in 0..3 {
            self.player_row(player, &mut out);
            out.push('|');
        }
        for (team, _) in TEAMS {
            for name in KINDS {
                let template = self
                    .sim
                    .rules
                    .thing_factory
                    .find_template_id(name)
                    .expect("template");
                let team = TeamId(team);
                let _ = write!(
                    out,
                    "{},{},{};",
                    self.sim.unit_cost_to_build(team, template),
                    self.sim.unit_time_to_build(team, template),
                    u8::from(self.sim.player_allowed_to_build(team, template) != FALSE),
                );
            }
            out.push('|');
        }
        for (i, slot) in self.objects.iter().enumerate() {
            let Some(slot) = *slot else {
                continue;
            };
            let record = self.sim.world.objects.get(slot).expect("object");
            let mask = (0..DISABLED_COUNT as i32)
                .filter(|&d| record.disabled_mask.test(d) != FALSE)
                .fold(0u32, |mask, d| mask | 1 << d);
            let label = TEAMS
                .iter()
                .position(|&(team, _)| TeamId(team) == record.team_id)
                .expect("team label");
            let _ = write!(out, "{i} {label} {mask};");
        }
        out
    }
}

#[test]
fn power_ids_match_reference() {
    let trial = Trial::new(1);
    let ids: Vec<String> = (0..POWERS.len() as i64)
        .map(|i| trial.power_id(i).to_string())
        .collect();
    assert_eq!(ids.join(","), header("power_ids"));
}

#[test]
fn player_holdings_match_reference() {
    let vectors = vectors();
    let line = GOLDEN.lines().next().expect("first line");
    assert_eq!(
        vectors.len(),
        get_u32(line, "vector_count").expect("count") as usize
    );
    let mut rows = 0;
    for (v, vector) in vectors.iter().enumerate() {
        let mut trial = Trial::new(vector.base);
        assert_eq!(vector.rows.len(), vector.ops.len() + 1, "vector {v}");
        assert_eq!(trial.row(), vector.rows[0], "vector {v} setup");
        for (k, (&op, expected)) in vector.ops.iter().zip(&vector.rows[1..]).enumerate() {
            let ret = trial.run(op);
            let got = format!("{ret}!{}", trial.row());
            assert_eq!(got, *expected, "vector {v} op {k} {op:?}");
            rows += 1;
        }
    }
    assert!(rows > 5000, "{rows}");
}
