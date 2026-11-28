//! `PKT-390` `Object` lifecycle gaps: weapon set updates, battle plan bonuses on a new
//! object, mines on construction and on defection, capture and the pathfind map.
//! Vectors: `tests/cpp_harness/gen_object_lifecycle.cpp`. The scene below repeats the
//! harness step by step.

use std::collections::BTreeMap;

use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{Int, ObjectId, Real, Relationship, TeamId, FALSE, TRUE};
use zh_common::xfer::{Snapshot, XferSave};
use zh_logic::arena::{ModuleSidecar, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::pathfinder::PathfindLayerEnum;
use zh_logic::object::weapon_set::{WeaponLockType, WeaponSlotType};
use zh_logic::rules::combat_types::{DamageType, WeaponSetType};
use zh_logic::rules::ObjectStatusType;
use zh_logic::sim::{SimApi, SimContext, SimView, SimWorld};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/PKT-390_lifecycle.txt");

const WEAPONS: [(&str, &str); 3] = [
    ("WA", "PrimaryDamage = 5.0\n  AttackRange = 100.0\n  ClipSize = 4\n  ClipReloadTime = 1000\n  DelayBetweenShots = 200"),
    ("WB", "PrimaryDamage = 7.0\n  AttackRange = 50.0\n  DamageType = FLAME\n  AntiAirborneVehicle = Yes"),
    ("WC", "PrimaryDamage = 9.0\n  AttackRange = 150.0\n  ClipSize = 2\n  ClipReloadTime = 500\n  DamageType = ARMOR_PIERCING"),
];

const OBJECTS: &str = "\
Object WSUnit
  KindOf = VEHICLE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  WeaponSet
    Conditions = None
    Weapon = PRIMARY WA
    Weapon = SECONDARY WB
  End
  WeaponSet
    Conditions = VETERAN
    ShareWeaponReloadTime = Yes
    Weapon = PRIMARY WC
    Weapon = TERTIARY WB
  End
  WeaponSet
    Conditions = ELITE
    WeaponLockSharedAcrossSets = Yes
    Weapon = PRIMARY WA
    Weapon = SECONDARY WC
  End
End
Object PlanVeh
  KindOf = VEHICLE
  VisionRange = 100
  ShroudClearingRange = 120
  Geometry = CYLINDER
  GeometryMajorRadius = 5
  GeometryHeight = 5
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
End
Object PlanInf
  KindOf = INFANTRY
  VisionRange = 100
  ShroudClearingRange = 120
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
End
Object Bld
  KindOf = STRUCTURE
  Geometry = BOX
  GeometryMajorRadius = 20
  GeometryMinorRadius = 20
  GeometryHeight = 20
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 500
    InitialHealth = 500
  End
End
Object Mine
  KindOf = MINE IMMOBILE
  Geometry = CYLINDER
  GeometryMajorRadius = 3
  GeometryHeight = 2
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
End
Object Rock
  KindOf = IMMOBILE
  Geometry = CYLINDER
  GeometryMajorRadius = 3
  GeometryHeight = 2
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
End
Object Walker
  KindOf = INFANTRY
  TransportSlotCount = 1
  Geometry = CYLINDER
  GeometryMajorRadius = 5
  GeometryHeight = 10
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL LifeLegs
End
Object Truck
  KindOf = VEHICLE
  Geometry = CYLINDER
  GeometryMajorRadius = 5
  GeometryHeight = 10
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = TransportContain ModuleTag_Contain
    Slots = 4
    AllowInsideKindOf = INFANTRY
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL LifeLegs
End
Object Tech
  KindOf = STRUCTURE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 500
    InitialHealth = 500
  End
End
Object Dozer
  KindOf = VEHICLE DOZER
  Geometry = CYLINDER
  GeometryMajorRadius = 5
  GeometryHeight = 10
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = DozerAIUpdate ModuleTag_AI
    RepairHealthPercentPerSecond = 2%
    BoredTime = 5000
    BoredRange = 200
  End
  Locomotor = SET_NORMAL LifeLegs
End
";

/// Players 0 (neutral), 1 and 2 with their default teams, and a second team of player 1.
const TEAM_N: TeamId = TeamId(1);
const TEAM_A: TeamId = TeamId(2);
const TEAM_B: TeamId = TeamId(3);

struct Scene {
    sim: SimContext,
    names: BTreeMap<Int, String>,
    rows: Vec<String>,
}

fn extent() -> Region3D {
    Region3D {
        lo: Coord3D::new(0.0, 0.0, 0.0),
        hi: Coord3D::new(1000.0, 1000.0, 100.0),
    }
}

impl Scene {
    fn new() -> Self {
        let mut sim = SimContext::new();
        for (name, body) in WEAPONS {
            let mut weapon = WeaponTemplate::new();
            weapon
                .parse_ini_block(&format!("Weapon {name}\n  {body}\nEnd\n"))
                .expect("weapon");
            weapon.name = name.to_owned();
            sim.rules.weapon_store.add_template(weapon);
        }
        let mut legs = LocomotorTemplate::new();
        legs.parse_ini_block(
            "Locomotor LifeLegs\n  Surfaces = GROUND\n  Appearance = TWO_LEGS\n  Speed = 30\nEnd\n",
        )
        .expect("locomotor");
        legs.name = "LifeLegs".to_owned();
        sim.rules.locomotor_store.add_template(legs);
        let mut reader = IniReader::new("object.ini", OBJECTS.as_bytes(), IniLoadType::Overwrite);
        sim.rules
            .load_thing_templates_from_ini(&mut reader)
            .expect("objects");
        sim.rules.thing_factory.post_process_load();

        sim.world = SimWorld::new();
        sim.init_partition(20.0, extent());
        sim.new_pathfind_map(&extent(), 12.0);
        assert!(sim
            .world
            .pathfinder
            .as_mut()
            .expect("map")
            .begin_path_queue(&extent(), 100));
        for (player, team) in [TEAM_N, TEAM_A, TEAM_B].into_iter().enumerate() {
            sim.world.player_default_teams.insert(player as Int, team);
            sim.add_player_team(team, player as Int);
        }
        sim.world.neutral_team_id = TEAM_N;
        sim.world.team_is_neutral.insert(TEAM_N, TRUE);
        sim.world.local_player_index = 5;
        for i in 0..3 {
            for j in 0..3 {
                let relation = if i == j {
                    Relationship::Allies
                } else if i == 0 || j == 0 {
                    Relationship::Neutral
                } else {
                    Relationship::Enemies
                };
                sim.world.set_player_relation(i, j, relation);
            }
        }
        Self {
            sim,
            names: BTreeMap::new(),
            rows: Vec::new(),
        }
    }

    fn make(&mut self, name: &str, template: &str, team: TeamId, x: Real, y: Real) -> ObjectSlot {
        let id = self
            .sim
            .rules
            .thing_factory
            .find_template_id(template)
            .unwrap_or_else(|| panic!("template {template}"));
        // As the reference: made at the origin, then positioned.
        let slot = self.sim.spawn_object(id, team, Coord3D::default(), 0.0);
        self.sim.set_object_position(slot, &Coord3D::new(x, y, 0.0));
        let object = self.sim.world.objects.id_of(slot).expect("id");
        self.names.insert(object.0, name.to_owned());
        slot
    }

    fn row(&mut self, text: String) {
        self.rows.push(text);
    }

    // ---- weapon sets

    fn weapons_row(&mut self, op: &str, o: ObjectSlot) {
        let mut x = XferSave::new();
        self.sim
            .world
            .objects
            .get_mut(o)
            .expect("object")
            .weapon_set
            .xfer(&mut x)
            .expect("xfer");
        let hex: String = x.into_bytes().iter().map(|b| format!("{b:02x}")).collect();
        self.row(format!("weapons {op} {hex}"));
    }

    fn lock(&mut self, o: ObjectSlot, slot: WeaponSlotType, lock: WeaponLockType) {
        let set = &mut self.sim.world.objects.get_mut(o).expect("object").weapon_set;
        set.set_weapon_lock(slot, lock);
    }

    fn weapon_sets(&mut self) {
        use WeaponSetType::{Elite, Hero, Veteran};
        self.sim.world.frame = 7;
        let o = self.make("ws", "WSUnit", TEAM_A, 50.0, 50.0);
        self.weapons_row("spawn", o);
        self.sim.world.frame = 11;
        self.lock(o, WeaponSlotType::Secondary, WeaponLockType::LockedPermanently);
        self.weapons_row("lock2", o);
        self.sim.world.frame = 20;
        self.sim.set_weapon_set_flag(o, Veteran, TRUE);
        self.weapons_row("+vet", o);
        self.sim.world.frame = 25;
        self.sim.set_weapon_set_flag(o, Veteran, TRUE);
        self.weapons_row("+vet-again", o);
        self.lock(o, WeaponSlotType::Tertiary, WeaponLockType::LockedTemporarily);
        self.sim.world.frame = 31;
        self.sim.set_weapon_set_flag(o, Veteran, FALSE);
        self.weapons_row("-vet", o);
        self.lock(o, WeaponSlotType::Secondary, WeaponLockType::LockedPermanently);
        self.sim.world.frame = 42;
        self.sim.set_weapon_set_flag(o, Elite, TRUE);
        self.weapons_row("+elite", o);
        self.sim.world.frame = 53;
        self.sim.set_weapon_set_flag(o, Veteran, TRUE);
        self.weapons_row("+vet-on-elite", o);
        self.sim.world.frame = 64;
        self.sim.set_weapon_set_flag(o, Hero, TRUE);
        self.weapons_row("+hero", o);
        self.sim.world.frame = 75;
        self.sim.set_weapon_set_flag(o, Elite, FALSE);
        self.weapons_row("-elite", o);
        self.sim.world.frame = 86;
        self.sim.set_weapon_set_flag(o, Veteran, FALSE);
        self.weapons_row("-vet-hero-left", o);
    }

    // ---- mines on construction

    fn mine_row(&mut self, op: &str, o: ObjectSlot) {
        let object = self.sim.world.objects.get(o).expect("object");
        let (id, destroyed) = (object.id, object.is_destroyed());
        let dead = object.is_effectively_dead() != FALSE;
        let health = self.sim.object_health(o).to_bits();
        let name = self.name_of(id);
        self.row(format!(
            "mine {op} {name} {} {} {health}",
            u8::from(destroyed),
            u8::from(dead)
        ));
    }

    fn construction_mines(&mut self) {
        use ObjectStatusType::UnderConstruction;
        self.sim.world.frame = 200;
        for round in 0..3 {
            let x = 200.0 + 100.0 * round as Real;
            let mines: Vec<ObjectSlot> = [
                ("enemy", "Mine", TEAM_B, x + 5.0, 205.0),
                ("own", "Mine", TEAM_A, x - 5.0, 205.0),
                ("neutral", "Mine", TEAM_N, x, 195.0),
                ("edge", "Mine", TEAM_B, x + 22.0, 200.0),
                ("far", "Mine", TEAM_B, x + 40.0, 200.0),
                ("rock", "Rock", TEAM_B, x, 205.0),
            ]
            .into_iter()
            .map(|(name, template, team, x, y)| {
                self.make(&format!("{name}{round}"), template, team, x, y)
            })
            .collect();
            let bld = self.make(&format!("bld{round}"), "Bld", TEAM_A, x, 200.0);
            if round == 2 {
                for &m in &mines {
                    self.mine_row(&format!("placed{round}"), m);
                }
            }
            self.sim.update_partition();
            self.sim.set_status(bld, UnderConstruction);
            if round == 0 {
                for &m in &mines {
                    self.mine_row("set", m);
                }
            } else if round == 1 {
                let late = [
                    self.make("late-enemy", "Mine", TEAM_B, x + 8.0, 195.0),
                    self.make("late-own", "Mine", TEAM_A, x - 8.0, 195.0),
                ];
                self.sim.update_partition();
                self.sim.set_status(bld, UnderConstruction);
                for m in late {
                    self.mine_row("same", m);
                }
                self.sim.clear_status(bld, UnderConstruction);
                for m in late {
                    self.mine_row("cleared", m);
                }
            } else {
                for &m in &mines {
                    self.mine_row("set2", m);
                }
            }
            self.sim.process_destroy_list();
        }
    }

    // ---- pathfind map

    fn name_of(&self, id: ObjectId) -> String {
        if id == ObjectId::INVALID {
            return "-".to_owned();
        }
        self.names
            .get(&id.0)
            .cloned()
            .unwrap_or_else(|| format!("?{}", id.0))
    }

    fn cells_row(&mut self, op: &str) {
        let mut out = format!("cells {op}");
        let pf = self.sim.world.pathfinder.as_ref().expect("map");
        for x in 60..=90 {
            for y in 60..=90 {
                let Some(cell) = pf.get_cell(PathfindLayerEnum::Ground, x, y) else {
                    continue;
                };
                let (goal, pos) = (pf.goal_unit(cell), pf.unit_present(cell));
                if goal != ObjectId::INVALID || pos != ObjectId::INVALID {
                    out += &format!(" {x}.{y}.{}.{}", self.name_of(goal), self.name_of(pos));
                }
            }
        }
        self.row(out);
    }

    fn reserve(&mut self, o: ObjectSlot, x: Real, y: Real) {
        let at = Coord3D::new(x, y, 0.0);
        self.sim
            .pathfinder_update_goal(o, &at, PathfindLayerEnum::Ground);
        self.sim.update_pathfind_position(o);
    }

    fn pathfind(&mut self) {
        self.sim.world.frame = 500;
        let gone = self.make("gone", "Walker", TEAM_A, 655.0, 655.0);
        self.reserve(gone, 705.0, 655.0);
        self.cells_row("reserved");
        self.sim.destroy_object(gone);
        self.cells_row("destroyed");
        self.sim.process_destroy_list();
        self.cells_row("flushed");

        let rider = self.make("rider", "Walker", TEAM_A, 655.0, 755.0);
        let truck = self.make("truck", "Truck", TEAM_A, 805.0, 805.0);
        self.reserve(rider, 705.0, 755.0);
        self.cells_row("rider");
        self.sim.contain_add(truck, rider);
        self.cells_row("contained");
        self.sim.contain_remove(truck, rider, FALSE);
        self.cells_row("removed");
    }
}

fn check(prefix: &str, rows: &[String]) {
    let want: Vec<&str> = VECTORS
        .lines()
        .filter(|l| l.split(' ').next() == Some(prefix))
        .collect();
    let got: Vec<&str> = rows
        .iter()
        .map(String::as_str)
        .filter(|l| l.split(' ').next() == Some(prefix))
        .collect();
    assert!(!want.is_empty(), "no {prefix} vectors");
    for (i, (g, w)) in got.iter().zip(&want).enumerate() {
        assert_eq!(g, w, "{prefix} row {i}");
    }
    assert_eq!(got.len(), want.len(), "{prefix} row count");
}

#[test]
fn pathfind_map_loses_destroyed_and_contained_units() {
    let mut scene = Scene::new();
    scene.pathfind();
    // Object ids differ between this scene and the reference's, names do not.
    check("cells", &scene.rows);
}

#[test]
fn construction_clears_the_mines_under_the_object() {
    let mut scene = Scene::new();
    scene.construction_mines();
    check("mine", &scene.rows);
}

#[test]
fn weapon_set_follows_the_flags() {
    let mut scene = Scene::new();
    scene.weapon_sets();
    check("weapons", &scene.rows);
}
