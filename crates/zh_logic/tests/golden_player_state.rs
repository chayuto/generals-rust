//! Money, energy and upgrades belong to the player, whichever of its teams acts.
//! Oracle: `tests/cpp_harness/gen_player_state.cpp`
//! (`docs/golden_vectors/PlayerState.json`).
//!
//! Every vector gives two players several teams (and several instances of one
//! prototype), then spawns, moves and destroys objects, grants and removes
//! upgrades, moves money and power, queues research and hands teams over. Each
//! row is compared whole. The `ObjectCreationUpgrade` records give the order in
//! which `Player::onUpgradeCompleted` reached the objects.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{ObjectId, Relationship, TeamId, FALSE, TRUE};
use zh_common::xfer::{Snapshot, XferSave};
use zh_logic::arena::{ModuleSlot, ObjectSlot};
use zh_logic::economy::energy::Energy;
use zh_logic::rules::command_set::{CommandButton, CommandButtonType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/PlayerState.json");

fn header(name: &str) -> String {
    let prefix = format!("\"{name}\":\"");
    let line = VECTORS
        .lines()
        .find(|l| l.starts_with(&prefix))
        .expect("header line");
    line[prefix.len()..line.rfind('"').expect("quote")].replace("\\n", "\n")
}

fn vector_lines() -> impl Iterator<Item = &'static str> {
    VECTORS.lines().filter(|l| l.starts_with("{\"v\":"))
}

struct Fixture {
    upgrades: String,
    ocl: String,
    objects: String,
    names: Vec<String>,
    kinds: Vec<String>,
    sets: Vec<(String, Vec<String>)>,
}

impl Fixture {
    fn load() -> Self {
        let list = |text: String| text.split(',').map(str::to_owned).collect::<Vec<_>>();
        let sets = header("sets")
            .split_terminator(';')
            .map(|set| {
                let (name, upgrades) = set.split_once(':').expect("set");
                (name.to_owned(), list(upgrades.to_owned()))
            })
            .collect();
        Self {
            upgrades: header("upgrades"),
            ocl: header("ocl"),
            objects: header("objects"),
            names: list(header("names")),
            kinds: list(header("kinds")),
            sets,
        }
    }

    fn rules(&self) -> RulesRegistry {
        let reader = |text| IniReader::new("player.ini", text, IniLoadType::Overwrite);
        let mut rules = RulesRegistry::new();
        rules
            .upgrade_center
            .load_from_ini(&mut reader(self.upgrades.as_bytes()), &mut rules.name_keys)
            .expect("upgrades");
        for line in self.ocl.lines() {
            if let Some(name) = line.strip_prefix("ObjectCreationList ") {
                rules.ocl_names.insert(name.to_owned());
            }
        }
        let mut r = reader(self.objects.as_bytes());
        if let Err(e) = rules.load_thing_templates_from_ini(&mut r) {
            panic!("thing templates, line {}: {e:?}", r.get_line_num());
        }
        rules.thing_factory.post_process_load();
        for (name, upgrades) in &self.sets {
            let buttons = upgrades
                .iter()
                .map(|u| CommandButton {
                    command: CommandButtonType::Other,
                    object: String::new(),
                    upgrade: u.clone(),
                })
                .collect();
            rules.command_sets.insert(name, buttons);
        }
        rules
    }
}

/// The reference's team labels (see `Trial` in the harness).
fn team_id(label: usize) -> TeamId {
    const FIXED: [i32; 6] = [10, 11, 12, 23, 24, 25];
    TeamId(*FIXED.get(label).unwrap_or(&(20 + label as i32)))
}

/// Teams that are in no list walk as the default team and then by id. A script that
/// uses that order and never changes a list needs no registration.
fn lists_are_implicit(line: &str) -> bool {
    get_str(line, "order_a") == Some("134")
        && get_str(line, "order_b") == Some("25")
        && ops(line).iter().all(|&[code, ..]| code != 14 && code != 15)
}

fn new_sim(fixture: &Fixture, line: &str) -> SimContext {
    let mut sim = SimContext::new();
    sim.rules = fixture.rules();
    sim.world = SimWorld::new();
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
    sim.world.frame = get_u32(line, "frame").expect("frame");
    let w = &mut sim.world;
    for player in 0..3 {
        w.player_default_teams
            .insert(player, team_id(player as usize));
        w.player_is_human.insert(team_id(player as usize), FALSE);
    }
    w.team_players.insert(team_id(0), 0);
    w.team_is_neutral.insert(team_id(0), TRUE);
    w.neutral_team_id = team_id(0);
    w.local_player_index = 5;
    for i in 0..3 {
        for j in 0..3 {
            let rel = if i == j {
                Relationship::Allies
            } else if i == 0 || j == 0 {
                Relationship::Neutral
            } else {
                Relationship::Enemies
            };
            w.player_relationships.insert((i, j), rel);
        }
    }
    for (key, player) in [("order_a", 1), ("order_b", 2)] {
        for digit in get_str(line, key).expect(key).chars() {
            let label = digit.to_digit(10).expect("label") as usize;
            if lists_are_implicit(line) {
                sim.world.team_players.insert(team_id(label), player);
            } else {
                sim.add_player_team(team_id(label), player);
            }
        }
    }
    // `Money::deposit`, which does nothing for zero.
    for (player, key) in [(0, "m0"), (1, "m1"), (2, "m2")] {
        let money = get_u32(line, key).expect(key) as i32;
        let _ = sim.add_player_money(team_id(player), money);
    }
    sim
}

struct Trial<'a> {
    fixture: &'a Fixture,
    sim: SimContext,
    teams: usize,
    objects: Vec<Option<ObjectSlot>>,
    kinds: Vec<usize>,
    index_of: BTreeMap<ObjectId, usize>,
}

impl Trial<'_> {
    fn mask_bits(
        &mut self,
        from: usize,
        to: usize,
        has: impl Fn(&SimContext, usize) -> bool,
    ) -> String {
        (from..to)
            .map(|i| if has(&self.sim, i) { '1' } else { '0' })
            .collect()
    }

    fn markers(&mut self) -> String {
        let mut out = String::new();
        for (ocl, creator, _, _) in std::mem::take(&mut self.sim.world.spawned_ocls) {
            let number = ocl.strip_prefix("OCL_G").expect("ocl name");
            let _ = write!(out, "{number}:{},", self.index_of[&creator]);
        }
        out
    }

    fn player_row(&mut self, player: i32) -> String {
        let w = &self.sim.world;
        let money = w.players.player(player).map_or(0, |p| p.money.count_money());
        let mut energy = w.players.player(player).map(|p| &p.energy).cloned().unwrap_or_else(|| {
            let mut fresh = Energy::new();
            fresh.init(Some(player));
            fresh
        });
        let frame = w.frame;
        let mut xs = XferSave::new();
        energy.xfer(&mut xs).expect("energy save");
        let save: String = xs.as_bytes().iter().map(|b| format!("{b:02x}")).collect();
        let done = w.players.player(player).map(|p| p.upgrades_completed.clone()).unwrap_or_default();
        let pending = w.players.player(player).map(|p| p.upgrades_in_progress.clone()).unwrap_or_default();
        let score = w.players.player(player).map(|p| &p.score_keeper);
        let (earned, spent) = score.map_or((0, 0), |s| (s.total_money_earned, s.total_money_spent));
        let production = energy.get_production(frame);
        let consumption = energy.get_consumption();
        let sufficient = energy.has_sufficient_power(frame);
        let mut bits = [String::new(), String::new()];
        for name in &self.fixture.names[..9] {
            let mask = self
                .sim
                .rules
                .upgrade_center
                .find_upgrade(name, &mut self.sim.rules.name_keys)
                .expect("upgrade")
                .upgrade_mask()
                .clone();
            for (out, owned) in bits.iter_mut().zip([&done, &pending]) {
                out.push(if owned.test_for_any(&mask) != FALSE {
                    '1'
                } else {
                    '0'
                });
            }
        }
        let [done, pending] = bits;
        format!("{money} {production} {consumption} {sufficient} {save} {done} {pending} {earned} {spent}")
    }

    /// What every team of a player is told must be the player's own state.
    fn check_team_views(&mut self, context: &str) {
        for label in 0..self.teams {
            let team = team_id(label);
            let player = self.sim.player_index(team);
            let w = &self.sim.world;
            let money = w.players.player(player).map_or(0, |p| p.money.count_money());
            let production = w.players.player(player).map(|p| &p.energy)
                .map_or(0, |e| e.get_production(w.frame));
            let done = w.players.player(player).map(|p| p.upgrades_completed.clone()).unwrap_or_default();
            let pending = w.players.player(player).map(|p| p.upgrades_in_progress.clone()).unwrap_or_default();
            assert_eq!(
                self.sim.player_money(team),
                money,
                "{context} money, team {label}"
            );
            assert_eq!(
                self.sim.player_energy_production(team),
                production,
                "{context} power, team {label}"
            );
            for name in &self.fixture.names[..9] {
                let mask = self
                    .sim
                    .rules
                    .upgrade_center
                    .find_upgrade(name, &mut self.sim.rules.name_keys)
                    .expect("upgrade")
                    .upgrade_mask()
                    .clone();
                assert_eq!(
                    self.sim.player_has_upgrade(team, &mask),
                    done.test_for_any(&mask),
                    "{context} {name}, team {label}"
                );
                assert_eq!(
                    self.sim.player_has_upgrade_in_production(team, &mask),
                    pending.test_for_all(&mask),
                    "{context} {name} pending, team {label}"
                );
            }
        }
    }

    fn object_row(&mut self, slot: ObjectSlot, asked: usize) -> String {
        let key = self
            .sim
            .rules
            .name_keys
            .name_to_key(&self.fixture.names[asked]);
        let names = self.fixture.names[9..].to_vec();
        let masks: Vec<_> = names
            .iter()
            .map(|n| {
                self.sim
                    .rules
                    .upgrade_center
                    .find_upgrade(n, &mut self.sim.rules.name_keys)
                    .expect("upgrade")
                    .upgrade_mask()
                    .clone()
            })
            .collect();
        let own = self.mask_bits(0, masks.len(), |sim, i| {
            sim.object_has_upgrade(slot, &masks[i]) != FALSE
        });
        let sim = &self.sim;
        let o = sim.world.objects.get(slot).expect("object");
        let team = (0..self.teams)
            .find(|&l| team_id(l) == o.team_id)
            .expect("team label");
        let mut upgraded = String::new();
        let mut queued = 0;
        for &(ms, _) in &o.behaviors {
            let module = sim.world.modules.payload(ms).expect("payload");
            if let Some(u) = module.as_upgrade() {
                upgraded.push(if u.is_already_upgraded() != FALSE {
                    '1'
                } else {
                    '0'
                });
            }
            if let Some(p) = module.as_production() {
                queued = p.production_count();
            }
        }
        format!(
            "{team} {upgraded} {own} {} {} {} {} {queued} {}",
            sim.object_max_health(slot) as i32,
            o.weapon_bonus_condition,
            sim.object_command_set_string(slot),
            u8::from(o.is_disabled() != FALSE),
            sim.affected_by_upgrade(slot, key),
        )
    }

    fn row(&mut self, asked: usize, ret: u32) -> String {
        let mut out = format!("{ret}|{}|", self.markers());
        for label in 0..self.teams {
            let _ = write!(out, "{}", self.sim.player_index(team_id(label)));
        }
        for player in 0..3 {
            out.push('|');
            let row = self.player_row(player);
            out.push_str(&row);
        }
        out.push('|');
        for index in 0..self.objects.len() {
            if let Some(slot) = self.objects[index] {
                let row = self.object_row(slot, asked);
                let _ = write!(out, "{index} {row};");
            }
        }
        out
    }

    fn module_tagged(&mut self, slot: ObjectSlot, tag: &str) -> ModuleSlot {
        let key = self.sim.rules.name_keys.name_to_key(tag);
        let object = self.sim.world.objects.get(slot).expect("object");
        object
            .behaviors
            .iter()
            .map(|&(ms, _)| ms)
            .find(|&ms| self.sim.world.modules.meta(ms).tag_key == key)
            .expect("tagged module")
    }

    fn run_updates(&mut self) {
        for index in 0..self.objects.len() {
            let Some(slot) = self.objects[index] else {
                continue;
            };
            let tag = match self.fixture.kinds[self.kinds[index]].as_str() {
                "G_Lab" => "ModuleTag_Production",
                "G_Bank" => "ModuleTag_Deposit",
                _ => continue,
            };
            let module = self.module_tagged(slot, tag);
            self.sim
                .dispatch_module(module, |m, sim, ctx| {
                    let _ = m.as_update_mut().expect("update").update(sim, ctx);
                })
                .expect("dispatch");
        }
    }

    fn apply(&mut self, code: u32, a: usize, b: usize) -> u32 {
        let mut ret = 0;
        let slot = |objects: &[Option<ObjectSlot>]| objects[a].expect("live object");
        match code {
            0 => {
                let sim = &mut self.sim;
                let name = &self.fixture.kinds[b];
                let tmpl = sim.rules.thing_factory.find_template_id(name).expect(name);
                let id = ObjectId(sim.world.next_object_id);
                self.index_of.insert(id, self.objects.len());
                let o = sim.factory_new_object(tmpl, team_id(a));
                assert_eq!(sim.world.objects.id_of(o), Some(id), "object id");
                self.objects.push(Some(o));
                self.kinds.push(b);
            }
            1 => {
                let o = slot(&self.objects);
                self.sim.destroy_object(o);
                self.sim.process_destroy_list();
                self.objects[a] = None;
            }
            2 => self.sim.set_object_team(slot(&self.objects), team_id(b)),
            3 => self
                .sim
                .grant_player_upgrade(team_id(a), &self.fixture.names[b]),
            4 | 5 => {
                let key = self.sim.rules.name_keys.name_to_key(&self.fixture.names[b]);
                if code == 4 {
                    self.sim.player_add_upgrade_in_production(team_id(a), key);
                } else {
                    self.sim.player_remove_upgrade(team_id(a), key);
                }
            }
            6 => self
                .sim
                .grant_upgrade(slot(&self.objects), &self.fixture.names[b], FALSE),
            7 => self
                .sim
                .remove_upgrade(slot(&self.objects), &self.fixture.names[b]),
            8 => {
                let _ = self.sim.add_player_money(team_id(a), b as i32);
            }
            9 => ret = self.sim.withdraw_player_money(team_id(a), b as u32),
            10 => self
                .sim
                .set_disabled(slot(&self.objects), DisabledType::Hacked),
            11 => {
                let o = slot(&self.objects);
                ret = u32::from(self.sim.clear_disabled(o, DisabledType::Hacked) == TRUE);
            }
            12 => self.sim.sabotage_power(team_id(a), b as u32),
            13 => {
                for _ in 0..a {
                    self.sim.world.frame += 1;
                    self.run_updates();
                }
            }
            14 => self.sim.set_team_controlling_player(team_id(a), b as i32),
            15 => {
                assert_eq!(a, self.teams, "new team label");
                self.sim.add_team_instance(team_id(a), team_id(b));
                self.teams += 1;
            }
            16 | 17 => {
                let o = slot(&self.objects);
                let key = self.sim.rules.name_keys.name_to_key(&self.fixture.names[b]);
                let module = self.module_tagged(o, "ModuleTag_Production");
                ret = self
                    .sim
                    .dispatch_module(module, |m, sim, ctx| {
                        let p = m.as_production_mut().expect("production");
                        if code == 16 {
                            u32::from(p.queue_upgrade(sim, ctx, key) != FALSE)
                        } else {
                            p.cancel_upgrade(sim, ctx, key);
                            0
                        }
                    })
                    .expect("dispatch");
            }
            18 => self.sim.dispatch_on_build_complete(slot(&self.objects)),
            other => panic!("op {other}"),
        }
        ret
    }
}

/// `[code,a,b,asked]` records.
fn ops(line: &str) -> Vec<[usize; 4]> {
    get_str(line, "ops")
        .expect("ops")
        .split_terminator(']')
        .map(|op| {
            let fields: Vec<usize> = op[1..]
                .split(',')
                .map(|f| f.parse().expect("op field"))
                .collect();
            fields.try_into().expect("four fields")
        })
        .collect()
}

#[test]
fn player_state_oracle_shape() {
    assert_eq!(get_u32(VECTORS, "vector_count"), Some(1000));
    assert_eq!(vector_lines().count(), 1000);
    let fixture = Fixture::load();
    assert_eq!(
        (fixture.names.len(), fixture.kinds.len(), fixture.sets.len()),
        (12, 6, 3)
    );
}

#[test]
fn player_state_matches_reference() {
    let fixture = Fixture::load();
    let mut rows_checked = 0;
    for line in vector_lines() {
        let v = get_u32(line, "v").expect("v");
        let mut trial = Trial {
            fixture: &fixture,
            sim: new_sim(&fixture, line),
            teams: 6,
            objects: Vec::new(),
            kinds: Vec::new(),
            index_of: BTreeMap::new(),
        };
        let rows: Vec<&str> = get_str(line, "rows").expect("rows").split('~').collect();
        let ops = ops(line);
        assert_eq!(rows.len(), ops.len() + 1, "vector {v}");
        assert_eq!(trial.row(0, 0), rows[0], "vector {v} setup");
        for (k, (&[code, a, b, asked], want)) in ops.iter().zip(&rows[1..]).enumerate() {
            let ret = trial.apply(code as u32, a, b);
            let context = format!("vector {v} op {k} [{code},{a},{b},{asked}]");
            assert_eq!(trial.row(asked, ret), *want, "{context}");
            trial.check_team_views(&context);
            rows_checked += 1;
        }
    }
    assert!(rows_checked >= 1000, "only {rows_checked} rows");
    let implicit = vector_lines().filter(|l| lists_are_implicit(l)).count();
    assert!(implicit >= 10, "only {implicit} implicit-list scripts");
}

/// A null `Team*` has no controlling player, so it reaches nobody's state (the
/// reference would dereference null).
#[test]
fn no_team_means_no_player_state() {
    let fixture = Fixture::load();
    let mut sim = new_sim(&fixture, vector_lines().next().expect("a vector"));
    sim.set_player_cash(0, 100);
    let none = TeamId::INVALID;
    let key = sim.rules.name_keys.name_to_key("PU_Ord0");
    sim.grant_player_upgrade(none, "PU_Ord0");
    sim.player_add_upgrade_in_production(none, key);
    let _ = sim.add_player_money(none, 5);
    sim.sabotage_power(none, 5);
    assert_eq!(sim.withdraw_player_money(none, 50), 0);
    assert_eq!(sim.player_money(none), 0);
    assert_eq!(sim.money_of(0), 100);
    // An index outside the list holds nothing.
    assert_eq!(sim.money_of(99), 0);
    assert_eq!(sim.withdraw_from_player(99, 5), 0);
    for index in 0..16 {
        let player = sim.world.players.player(index).expect("player");
        assert_eq!(player.upgrades_completed, Default::default());
        assert_eq!(player.upgrades_in_progress, Default::default());
        assert_eq!(player.energy.get_power_sabotaged_till_frame(), 0);
        assert_eq!(player.energy.get_production(sim.world.frame), 0);
    }
}
