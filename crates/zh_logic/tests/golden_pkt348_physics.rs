// SPDX-License-Identifier: GPL-3.0-or-later
//! `PKT-348` `PhysicsBehavior` against the reference (`tests/cpp_harness/gen_pkt348_physics.cpp`).

use std::any::Any;
use std::collections::BTreeMap;
use std::fmt::Write as _;

use zh_common::golden_json::get_str;
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::{Coord3D, Matrix3D};
use zh_common::types::{Bool, NameKeyType, ObjectId, Real, Relationship, TeamId, FALSE};
use zh_common::xfer::{Xfer, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleDataId, ModuleSlot, ObjectSlot};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::combat::locomotor_set::LocomotorSetType;
use zh_logic::modules::module::{
    AiUpdateFacet, BehaviorModule, ModuleData, ModuleType, ProjectileFacet,
};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::modules::update::physics_update::{self as pb, physics, physics_mut};
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::{SleepyUpdatePhase, UPDATE_SLEEP_FOREVER};
use zh_logic::sim::{
    RulesRegistry, SimApi, SimContext, SimView, SimWorld, PRIVATE_STATUS_CAPTURED,
    PRIVATE_STATUS_OFF_MAP,
};

const NAMES: [&str; 17] = [
    "P348_Base",
    "P348_Bouncy",
    "P348_Heavy",
    "P348_Wall",
    "P348_Rock",
    "P348_Drone",
    "P348_Rester",
    "P348_Crusher",
    "P348_Crushee",
    "P348_Carrier",
    "P348_Walker",
    "P348_Climber",
    "P348_Flinger",
    "P348_Shell",
    "P348_Bunker",
    "P348_Plain",
    "P348_Mortal",
];

fn object_ini(
    name: &str,
    kind: &str,
    hp: &str,
    geometry: &str,
    extra: &str,
    physics: Option<&str>,
) -> String {
    let mut s = format!(
        "Object {name}\n  KindOf = PRELOAD {kind}\n{geometry}  Body = ActiveBody ModuleTag_Body\n    MaxHealth = {hp}\n    InitialHealth = {hp}\n  End\n{extra}"
    );
    if let Some(p) = physics {
        let _ = write!(s, "  Behavior = PhysicsBehavior ModuleTag_Phys\n{p}  End\n");
    }
    s.push_str("End\n");
    s
}

fn boxg(major: &str, minor: &str, height: &str) -> String {
    format!("  Geometry = BOX\n  GeometryMajorRadius = {major}\n  GeometryMinorRadius = {minor}\n  GeometryHeight = {height}\n")
}

fn cyl(radius: &str, height: &str) -> String {
    format!(
        "  Geometry = CYLINDER\n  GeometryMajorRadius = {radius}\n  GeometryHeight = {height}\n"
    )
}

/// The harness's `fixture_ini`, object blocks only.
fn objects_ini() -> String {
    let ai = "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n";
    let walker =
        format!("{ai}  Locomotor = SET_NORMAL P348Walk\n  Locomotor = SET_TAXIING P348Walk\n");
    let climber =
        format!("{ai}  Locomotor = SET_NORMAL P348Climb\n  Locomotor = SET_TAXIING P348Climb\n");
    [
        object_ini("P348_Base", "VEHICLE", "1000000", &boxg("10", "5", "8"), "", Some("")),
        object_ini(
            "P348_Bouncy", "INFANTRY", "1000000", &cyl("4", "10"), "",
            Some("    Mass = 5\n    AllowBouncing = Yes\n    ForwardFriction = 3\n    LateralFriction = 12\n    ZFriction = 20\n    AerodynamicFriction = 1.5\n    CenterOfMassOffset = 1.5\n    MinFallHeightForDamage = 5\n    FallHeightDamageFactor = 0.5\n    PitchRollYawFactor = 0.7\n    ShockResistance = 0.3\n    ShockMaxYaw = 0.4\n    ShockMaxPitch = 0.3\n    ShockMaxRoll = 0.2\n"),
        ),
        object_ini(
            "P348_Heavy", "VEHICLE", "1000000", &boxg("14", "8", "12"), "  CrushableLevel = 3\n",
            Some("    Mass = 50\n    AllowCollideForce = No\n    AerodynamicFriction = 45\n    ForwardFriction = 0.01\n    LateralFriction = 90\n    CenterOfMassOffset = -2\n    PitchRollYawFactor = 3.5\n    ShockResistance = 2.0\n    VehicleCrashesIntoBuildingWeaponTemplate = P348OtherCrash\n    VehicleCrashesIntoNonBuildingWeaponTemplate = VehicleCrashesIntoBuildingWeapon\n"),
        ),
        object_ini("P348_Wall", "STRUCTURE IMMOBILE", "1000000", &boxg("15", "15", "30"), "", None),
        object_ini("P348_Rock", "IMMOBILE", "1000000", &cyl("9", "14"), "", None),
        object_ini("P348_Drone", "DRONE", "500", &cyl("3", "4"), "", Some("    KillWhenRestingOnGround = Yes\n    CenterOfMassOffset = 0.5\n")),
        object_ini(
            "P348_Rester", "VEHICLE", "500", &boxg("6", "6", "4"), "",
            Some("    KillWhenRestingOnGround = Yes\n    AllowBouncing = Yes\n    Mass = 3\n    VehicleCrashesIntoBuildingWeaponTemplate = None\n    VehicleCrashesIntoNonBuildingWeaponTemplate = P348OtherCrash\n"),
        ),
        object_ini(
            "P348_Crusher", "VEHICLE", "1000000", &boxg("12", "8", "8"),
            "  CrusherLevel = 3\n  CrushableLevel = 2\n", Some("    Mass = 20\n"),
        ),
        object_ini(
            "P348_Crushee", "VEHICLE", "5000000", &boxg("12", "6", "6"),
            "  CrusherLevel = 0\n  CrushableLevel = 1\n", Some("    Mass = 4\n"),
        ),
        object_ini(
            "P348_Carrier", "VEHICLE", "1000000", &boxg("16", "10", "10"),
            "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 10\n  End\n",
            Some("    Mass = 30\n    ShockResistance = 0.5\n"),
        ),
        object_ini("P348_Walker", "INFANTRY", "800", &cyl("4", "10"), &walker, Some("    Mass = 2\n    AllowBouncing = Yes\n")),
        object_ini(
            "P348_Climber", "INFANTRY", "800", &cyl("4", "10"), &climber,
            Some("    Mass = 2\n    AllowBouncing = Yes\n    LateralFriction = 6\n"),
        ),
        object_ini(
            "P348_Flinger", "INFANTRY", "12", &cyl("4", "10"),
            "  Behavior = SlowDeathBehavior ModuleTag_Death\n    DeathTypes = ALL\n    FlingForce = 8\n    FlingForceVariance = 4\n    FlingPitch = 60\n    FlingPitchVariance = 15\n    DestructionDelay = 3000\n  End\n",
            Some("    Mass = 3\n    MinFallHeightForDamage = 2\n    FallHeightDamageFactor = 2.0\n"),
        ),
        object_ini("P348_Shell", "PROJECTILE", "1000000", &cyl("2", "2"), "", Some("    Mass = 0.5\n    MinFallHeightForDamage = 1\n")),
        object_ini("P348_Bunker", "STRUCTURE IMMOBILE", "1000000", &boxg("12", "12", "16"), "", Some("    Mass = 500\n")),
        object_ini("P348_Plain", "VEHICLE", "1000000", &boxg("8", "8", "8"), "", None),
        object_ini(
            "P348_Mortal", "VEHICLE", "30", &boxg("7", "4", "6"),
            "  Behavior = DestroyDie ModuleTag_Destroy\n  End\n",
            Some("    Mass = 4\n    MinFallHeightForDamage = 3\n    AllowBouncing = Yes\n"),
        ),
    ]
    .concat()
}

// ---------------------------------------------------------------------------
// Stand-ins for the unported AI and projectile updates
// ---------------------------------------------------------------------------

struct StubData {
    tag_key: NameKeyType,
    name_key: NameKeyType,
}

impl ModuleData for StubData {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }
    fn module_name_key(&self) -> NameKeyType {
        self.name_key
    }
    fn facet_mask(&self) -> FacetMask {
        FacetMask(0x001 | FacetMask::AI_UPDATE.0)
    }
}

struct StubAi {
    key: NameKeyType,
    ignored: ObjectId,
}

impl AiUpdateFacet for StubAi {
    fn get_ignored_obstacle_id(&self) -> ObjectId {
        self.ignored
    }
}

impl BehaviorModule for StubAi {
    fn module_name_key(&self) -> NameKeyType {
        self.key
    }
    fn xfer(&mut self, _x: &mut dyn Xfer, _s: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
    fn as_ai(&self) -> Option<&dyn AiUpdateFacet> {
        Some(self)
    }
    fn as_ai_mut(&mut self) -> Option<&mut dyn AiUpdateFacet> {
        Some(self)
    }
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
}

fn parse_ai(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    name_key: NameKeyType,
    _c: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(StubData { tag_key, name_key }))
}

fn create_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(StubAi {
        key: data.module_name_key(),
        ignored: ObjectId::INVALID,
    })
}

#[derive(Default)]
struct StubProjectile {
    handled: Bool,
    calls: u32,
}

impl ProjectileFacet for StubProjectile {
    fn projectile_handle_collision(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _other: Option<ObjectSlot>,
    ) -> Bool {
        self.calls += 1;
        self.handled
    }
}

impl BehaviorModule for StubProjectile {
    fn module_name_key(&self) -> NameKeyType {
        NameKeyType(907)
    }
    fn xfer(&mut self, _x: &mut dyn Xfer, _s: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
    fn as_projectile_mut(&mut self) -> Option<&mut dyn ProjectileFacet> {
        Some(self)
    }
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
}

// ---------------------------------------------------------------------------
// World
// ---------------------------------------------------------------------------

struct Terrain {
    ground: Real,
    cliff: bool,
    water: bool,
}

impl ThingTerrainAccess for Terrain {
    fn get_ground_height(&self, _x: Real, _y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        self.ground
    }
    fn align_on_terrain(
        &self,
        angle: Real,
        pos: &Coord3D,
        _stick: Bool,
        mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        zh_logic::object::thing::make_align_to_normal_matrix(
            angle,
            pos,
            &Coord3D::new(0.0, 0.0, 1.0),
            mtx,
        );
        PathfindLayerEnum::Ground
    }
    fn is_cliff_cell(&self, x: Real, _y: Real) -> Bool {
        Bool::from(self.cliff && x > 520.0)
    }
    fn is_underwater(
        &self,
        _x: Real,
        y: Real,
        water_z: Option<&mut Real>,
        terrain_z: Option<&mut Real>,
    ) -> Bool {
        if let Some(w) = water_z {
            *w = 0.0;
        }
        if let Some(t) = terrain_z {
            *t = self.ground;
        }
        Bool::from(self.water && y > 520.0)
    }
}

fn rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "AIUpdateInterface",
        ModuleType::Behavior,
        FacetMask(0x001 | FacetMask::AI_UPDATE.0),
        SleepyUpdatePhase::Initial,
        true,
        parse_ai,
        create_ai,
    );
    for name in ["VehicleCrashesIntoBuildingWeapon", "P348OtherCrash"] {
        let mut w = WeaponTemplate::new();
        w.parse_ini_block(&format!(
            "Weapon {name}\n  PrimaryDamage = 1.0\n  PrimaryDamageRadius = 0.0\n  AttackRange = 100.0\n  DelayBetweenShots = 1000\n  ClipSize = 0\nEnd"
        ))
        .expect("weapon");
        rules.weapon_store.add_template(w);
    }
    for (name, surfaces) in [("P348Walk", "GROUND"), ("P348Climb", "GROUND WATER CLIFF")] {
        let mut l = LocomotorTemplate::new();
        l.parse_ini_block(&format!(
            "Locomotor {name}\n  Surfaces = {surfaces}\n  Speed = 30\nEnd\n"
        ))
        .expect("locomotor");
        rules.locomotor_store.add_template(l);
    }
    let text = objects_ini();
    let mut ini = IniReader::new("object.ini", text.as_bytes(), IniLoadType::Overwrite);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut ini) {
        panic!("fixture line {}: {e:?}", ini.get_line_num());
    }
    rules
}

fn float(tok: &str) -> Real {
    Real::from_bits(u32::from_str_radix(tok, 16).expect("float bits"))
}

fn coord(tok: &str) -> Coord3D {
    let mut it = tok.split(',').map(float);
    Coord3D::new(it.next().unwrap(), it.next().unwrap(), it.next().unwrap())
}

fn hx(v: Real) -> String {
    let bits = if v.is_nan() { 0x7fc0_0000 } else { v.to_bits() };
    format!("{bits:08x}")
}

struct Ent {
    slot: ObjectSlot,
    id: ObjectId,
    module: Option<ModuleSlot>,
    ret: u32,
    last: String,
}

struct Run {
    sim: SimContext,
    ents: Vec<Ent>,
}

impl Run {
    fn index_of(&self, id: ObjectId) -> i32 {
        if id == ObjectId::INVALID {
            return -1;
        }
        self.ents
            .iter()
            .position(|e| e.id == id)
            .map_or(-2, |i| i as i32)
    }

    fn destroyed(&self, i: usize) -> bool {
        self.sim
            .world
            .objects
            .get(self.ents[i].slot)
            .is_none_or(|o| o.is_destroyed())
    }

    fn save(&mut self, ms: ModuleSlot) -> Vec<u8> {
        let (mut payload, _, generation) =
            self.sim.world.modules.take_payload(ms).expect("payload");
        let mut xs = XferSave::new();
        payload.xfer(&mut xs, &self.sim).expect("save");
        self.sim
            .world
            .modules
            .restore_payload(ms, generation, payload);
        xs.as_bytes().to_vec()
    }

    /// The save bytes, with NaN members made canonical.
    fn save_hex(&mut self, ms: ModuleSlot) -> String {
        let mut bytes = self.save(ms);
        let start = bytes.len() - 88;
        for off in [0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 60, 76, 80, 84] {
            let at = start + off;
            let v = f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
            if v.is_nan() {
                bytes[at..at + 4].copy_from_slice(&0x7fc0_0000_u32.to_le_bytes());
            }
        }
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    fn projectile_calls(&mut self, i: usize) -> u32 {
        let slot = self.ents[i].slot;
        let Some(ms) = self
            .sim
            .world
            .objects
            .get(slot)
            .and_then(|o| o.projectile_slot)
        else {
            return 0;
        };
        self.stub_projectile(ms).calls
    }

    fn stub_projectile(&mut self, ms: ModuleSlot) -> &mut StubProjectile {
        self.sim
            .world
            .modules
            .payload_mut(ms)
            .and_then(|m| m.as_any_mut())
            .and_then(|a| a.downcast_mut::<StubProjectile>())
            .expect("projectile stub")
    }

    fn ent_row(&mut self, i: usize) -> String {
        let slot = self.ents[i].slot;
        let health = self.sim.object_health(slot);
        let obj = self.sim.world.objects.get(slot).expect("object");
        let m = obj.thing.get_transform_matrix();
        let mut s = m
            .rows
            .iter()
            .flatten()
            .map(|&v| hx(v))
            .collect::<Vec<_>>()
            .join(",");
        let model = |f: ModelConditionFlagType| obj.model_condition.test(f.as_i32()) != 0;
        let flags = [
            obj.is_destroyed(),
            obj.is_effectively_dead() != FALSE,
            obj.is_disabled_by_type(DisabledType::Freefall) != FALSE,
            obj.is_disabled_by_type(DisabledType::Unmanned) != FALSE,
            model(ModelConditionFlagType::Freefall),
            model(ModelConditionFlagType::Splatted),
            model(ModelConditionFlagType::Stunned),
            model(ModelConditionFlagType::StunnedFlailing),
            obj.private_status & PRIVATE_STATUS_OFF_MAP != 0,
            obj.private_status & PRIVATE_STATUS_CAPTURED != 0,
        ];
        let bits: String = flags.iter().map(|&f| if f { '1' } else { '0' }).collect();
        let _ = write!(s, " {} {bits}{}", hx(health), obj.team_id.0);
        if let Some(ms) = self.ents[i].module {
            let collidee = physics(&self.sim, slot).expect("physics").last_collidee;
            let save = self.save_hex(ms);
            let next = self
                .sim
                .module_schedule(ms)
                .expect("schedule")
                .next_call_frame();
            let calls = self.projectile_calls(i);
            let _ = write!(
                s,
                " {save} {next} {} {} {calls}",
                self.ents[i].ret,
                self.index_of(collidee)
            );
        }
        s
    }

    fn row(&mut self, query: &str) -> String {
        let mut out = String::new();
        for i in 0..self.ents.len() {
            let r = self.ent_row(i);
            if r != self.ents[i].last {
                let _ = write!(out, "{i}={r};");
                self.ents[i].last = r;
            }
        }
        let fired: String = self
            .sim
            .world
            .weapon_fires
            .iter()
            .map(
                |(_, name, ..)| {
                    if name == "P348OtherCrash" {
                        'N'
                    } else {
                        'B'
                    }
                },
            )
            .collect();
        let _ = write!(
            out,
            "|{} {fired}",
            self.sim.world.rng.get_game_logic_random_seed_crc()
        );
        if !query.is_empty() {
            let _ = write!(out, " {query}");
        }
        out
    }

    fn step_all(&mut self, frame: u32) {
        self.sim.world.frame = frame;
        for i in 0..self.ents.len() {
            let Some(m) = self.ents[i].module else {
                continue;
            };
            if self.destroyed(i) {
                continue;
            }
            let next =
                |sim: &SimContext| sim.module_schedule(m).expect("schedule").next_call_frame();
            if next(&self.sim) > frame {
                continue;
            }
            let sleep = self
                .sim
                .dispatch_module(m, |b, s, c| b.as_update_mut().expect("update").update(s, c))
                .expect("dispatch");
            self.ents[i].ret = sleep.0;
            let next_frame = frame
                .saturating_add(sleep.0.max(1))
                .min(UPDATE_SLEEP_FOREVER);
            self.sim
                .world
                .modules
                .meta_mut(m)
                .schedule
                .set_next_call_frame(next_frame);
            let index = self.sim.world.modules.meta(m).schedule.index_in_logic();
            if index >= 0 {
                self.sim
                    .world
                    .sleepy_updates
                    .rebalance(&mut self.sim.world.modules, index as usize);
            }
        }
    }

    fn stub_ai(&mut self, i: usize) -> &mut StubAi {
        let ms = self
            .sim
            .world
            .objects
            .get(self.ents[i].slot)
            .and_then(|o| o.ai_slot)
            .expect("ai");
        self.sim
            .world
            .modules
            .payload_mut(ms)
            .and_then(|m| m.as_any_mut())
            .and_then(|a| a.downcast_mut::<StubAi>())
            .expect("ai stub")
    }

    fn attach_projectile(&mut self, i: usize, handled: Bool, on: Bool) {
        let slot = self.ents[i].slot;
        let existing = self
            .sim
            .world
            .objects
            .get(slot)
            .and_then(|o| o.projectile_slot);
        let ms = existing.unwrap_or_else(|| {
            self.sim.world.attach_module(
                slot,
                NameKeyType(907),
                ModuleDataId::INVALID,
                FacetMask::PROJECTILE_UPDATE,
                SleepyUpdatePhase::Normal,
                Box::new(StubProjectile::default()),
            )
        });
        self.stub_projectile(ms).handled = handled;
        if let Some(p) = physics_mut(&mut self.sim, slot) {
            p.has_projectile_update = on;
        }
    }

    /// Applies one recorded operation; returns the query result, if one was asked for.
    fn apply(&mut self, op: &str, frame: &mut u32) -> String {
        let mut tok: Vec<&str> = op.split(' ').collect();
        let query = tok
            .last()
            .and_then(|t| t.strip_prefix('?'))
            .map(|t| t.parse::<usize>().expect("query index"));
        if query.is_some() {
            tok.pop();
        }
        let num = |i: usize| tok[i].parse::<i64>().expect("number");
        let idx = |i: usize| num(i) as usize;
        match tok[0] {
            "S" => {
                for _ in 0..num(1) {
                    *frame += 1;
                    self.step_all(*frame);
                }
            }
            "G" => {}
            "CR" => {
                self.sim.world.frame = *frame;
                if !self.destroyed(0) {
                    let slot = self.ents[0].slot;
                    self.sim.set_object_position(slot, &coord(tok[1]));
                    self.sim.set_object_orientation(slot, float(tok[2]));
                    // The harness records the velocity it ended up with.
                    if let Some(p) = physics_mut(&mut self.sim, slot) {
                        p.vel = coord(tok[3]);
                    }
                }
            }
            "N" => {
                let (c, r) = (self.ents[idx(1)].slot, self.ents[idx(2)].slot);
                self.sim.contain_add(c, r);
            }
            "H" => {
                let slot = self.ents[idx(1)].slot;
                let kind = if num(2) != 0 {
                    DisabledType::Held
                } else {
                    DisabledType::Unmanned
                };
                if !self.destroyed(idx(1)) {
                    if num(3) != 0 {
                        self.sim.set_disabled(slot, kind);
                    } else {
                        self.sim.clear_disabled(slot, kind);
                    }
                }
            }
            "Z" => {
                let slot = self.ents[idx(1)].slot;
                if !self.destroyed(idx(1)) {
                    self.sim.set_crushed(
                        slot,
                        Some(Bool::from(num(2) != 0)),
                        Some(Bool::from(num(3) != 0)),
                    );
                }
            }
            "X" => {
                let slot = self.ents[idx(1)].slot;
                let set = if num(2) != 0 {
                    LocomotorSetType::Taxiing
                } else {
                    LocomotorSetType::Normal
                };
                self.sim.object_mut(slot).expect("object").ai_locomotor_set = Some(set);
                self.sim.world.rng.init_random_with_seed(num(3) as u32);
            }
            "IG" => {
                let id = self.ents[idx(2)].id;
                self.stub_ai(idx(1)).ignored = id;
            }
            _ => self.apply_to_object(&tok),
        }
        let Some(a) = query else {
            return String::new();
        };
        if self.destroyed(a) || self.ents[a].module.is_none() {
            return String::new();
        }
        let slot = self.ents[a].slot;
        let magnitude = pb::velocity_magnitude(&self.sim, slot);
        let (fwd2, fwd3) = (
            pb::forward_speed_2d(&self.sim, slot),
            pb::forward_speed_3d(&self.sim, slot),
        );
        let mass = self.sim.physics_mass(slot);
        let motive = u8::from(pb::is_motive(&self.sim, slot));
        let com = self.sim.physics_center_of_mass_offset(slot);
        let p = physics(&self.sim, slot).expect("physics");
        format!(
            "Q{a} {} {} {} {} {motive} {} {}{}",
            hx(magnitude),
            hx(fwd2),
            hx(fwd3),
            hx(mass),
            hx(com),
            u8::from(p.flag(pb::IS_STUNNED)),
            u8::from(p.flag(pb::ALLOW_TO_FALL))
        )
    }

    /// The operations skipped on a destroyed object.
    fn apply_to_object(&mut self, tok: &[&str]) {
        let a: usize = tok[1].parse().expect("object");
        if self.destroyed(a) {
            // `U` may still reach its second object.
            if tok[0] == "U" {
                self.set_status_of(tok, 4);
            }
            return;
        }
        let slot = self.ents[a].slot;
        let num = |i: usize| tok[i].parse::<i64>().expect("number");
        let other =
            |run: &Self, i: usize| usize::try_from(num(i)).ok().map(|k| (k, run.ents[k].slot));
        let on = |i: usize| Bool::from(num(i) != 0);
        match tok[0] {
            "F" => pb::apply_force(&mut self.sim, slot, &coord(tok[2])),
            "M" => pb::apply_motive_force(&mut self.sim, slot, &coord(tok[2])),
            "K" => pb::apply_shock(&mut self.sim, slot, &coord(tok[2])),
            "R" => pb::apply_random_rotation(&mut self.sim, slot),
            "V" => {
                if let Some(p) = physics_mut(&mut self.sim, slot) {
                    p.add_velocity(&coord(tok[2]));
                }
            }
            "Y" => {
                let rate = float(tok[3]);
                let p = physics_mut(&mut self.sim, slot).expect("physics");
                match num(2) {
                    0 => p.set_yaw_rate(rate),
                    1 => p.set_pitch_rate(rate),
                    _ => p.set_roll_rate(rate),
                }
            }
            "B" => {
                let flag = [
                    pb::ALLOW_TO_FALL,
                    pb::STICK_TO_GROUND,
                    pb::ALLOW_BOUNCE,
                    pb::ALLOW_COLLIDE_FORCE,
                    pb::APPLY_FRICTION2D_WHEN_AIRBORNE,
                    pb::IMMUNE_TO_FALLING_DAMAGE,
                    pb::IS_STUNNED,
                    pb::IS_IN_FREEFALL,
                ][num(2) as usize];
                physics_mut(&mut self.sim, slot)
                    .expect("physics")
                    .set_flag(flag, on(3));
            }
            "E" => {
                let p = physics_mut(&mut self.sim, slot).expect("physics");
                if num(2) != 0 {
                    p.extra_friction = float(tok[3]);
                } else {
                    p.extra_bounciness = float(tok[3]);
                }
            }
            "C" => {
                let p = physics_mut(&mut self.sim, slot).expect("physics");
                if num(2) != 0 {
                    p.scrub_velocity_z(float(tok[3]));
                } else {
                    p.scrub_velocity_2d(float(tok[3]));
                }
            }
            "D" => pb::reset_dynamic_physics(&mut self.sim, slot),
            "P" => self.sim.set_object_position(slot, &coord(tok[2])),
            "O" => pb::set_angles(
                &mut self.sim,
                slot,
                float(tok[2]),
                float(tok[3]),
                float(tok[4]),
            ),
            "L" => {
                let loc = *self.sim.object(slot).expect("object").thing.get_position();
                match other(self, 2) {
                    None => pb::on_collide(&mut self.sim, slot, None, Some(&loc), None),
                    Some((k, o)) => {
                        if !self.destroyed(k) {
                            pb::on_collide(&mut self.sim, slot, Some(o), Some(&loc), None);
                        }
                    }
                }
            }
            "T" => {
                let (_, o) = other(self, 2).expect("target");
                pb::transfer_velocity_to(&mut self.sim, slot, o);
            }
            "I" => {
                let id = other(self, 2).map_or(ObjectId::INVALID, |(k, _)| self.ents[k].id);
                physics_mut(&mut self.sim, slot)
                    .expect("physics")
                    .ignore_collisions_with = id;
            }
            "W" => physics_mut(&mut self.sim, slot).expect("physics").mass = float(tok[2]),
            "A" => self.sim.physics_clear_acceleration(slot),
            "J" => physics_mut(&mut self.sim, slot).expect("physics").turning = num(2) as i32,
            "U" => {
                self.set_status_of(tok, 1);
                self.set_status_of(tok, 4);
            }
            "PU" => self.attach_projectile(a, on(2), on(3)),
            "LD" => {
                let bytes: Vec<u8> = (0..tok[2].len() / 2)
                    .map(|i| u8::from_str_radix(&tok[2][i * 2..i * 2 + 2], 16).expect("hex"))
                    .collect();
                let ms = self.ents[a].module.expect("module");
                let (mut payload, _, generation) =
                    self.sim.world.modules.take_payload(ms).expect("payload");
                let mut xl = XferLoad::new(&bytes);
                payload.xfer(&mut xl, &self.sim).expect("load");
                self.sim
                    .world
                    .modules
                    .restore_payload(ms, generation, payload);
                self.sim
                    .dispatch_module(ms, |m, s, c| m.load_post_process(s, c))
                    .expect("post process");
            }
            "AO" => {
                let (k, o) = other(self, 2).expect("target");
                if !self.destroyed(k) {
                    pb::add_overlap(&mut self.sim, slot, o);
                }
            }
            "LY" => {
                let layer = if num(2) != 0 {
                    PathfindLayerEnum::Bridge1
                } else {
                    PathfindLayerEnum::Ground
                };
                self.sim.set_object_layer(slot, layer);
            }
            other => panic!("unknown op {other}"),
        }
    }

    /// `U a which on b`: the status of object `tok[at]`, if it names one.
    fn set_status_of(&mut self, tok: &[&str], at: usize) {
        let Ok(i) = tok[at].parse::<usize>() else {
            return;
        };
        if self.destroyed(i) {
            return;
        }
        let status = [
            ObjectStatusType::Braking,
            ObjectStatusType::Parachuting,
            ObjectStatusType::DeckHeightOffset,
        ][tok[2].parse::<usize>().expect("status")];
        let slot = self.ents[i].slot;
        if tok[3] != "0" {
            self.sim.set_status(slot, status);
        } else {
            self.sim.clear_status(slot, status);
        }
    }
}

fn strings(line: &str, start: &str, end: &str) -> Vec<String> {
    let rest = &line[line.find(start).expect(start) + start.len()..];
    let body = &rest[..rest.find(end).expect(end)];
    body.trim_matches('"')
        .split("\",\"")
        .map(str::to_owned)
        .collect()
}

#[test]
fn physics_behavior_matches_reference() {
    let json = include_str!("../../../docs/golden_vectors/PKT348_Physics.json");
    let lines: Vec<&str> = json
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with("{\"v\":"))
        .collect();
    assert_eq!(lines.len(), 1200);
    let mut run = Run {
        sim: SimContext::new(),
        ents: Vec::new(),
    };
    run.sim.rules = rules();
    let mut failures = BTreeMap::new();
    let mut checked = 0;
    for (v, line) in lines.iter().enumerate() {
        let setup: Vec<&str> = get_str(line, "setup").expect("setup").split(' ').collect();
        let ops = strings(line, "\"ops\":[", "],\"rows\"");
        let rows = strings(line, "\"rows\":[", "]}");
        assert_eq!(rows.len(), ops.len() + 1, "vector {v}");

        let gravity = float(setup[3]);
        let seed: u32 = setup[7].parse().unwrap();
        let mut frame: u32 = setup[8].parse().unwrap();
        run.sim.world = SimWorld::new();
        run.sim.world.frame = frame;
        run.sim
            .world.set_player_relation(1, 2, Relationship::Enemies);
        run.sim
            .world.set_player_relation(2, 1, Relationship::Enemies);
        run.sim.terrain = Box::new(Terrain {
            ground: float(setup[0]),
            cliff: setup[1] != "0",
            water: setup[2] != "0",
        });
        let data = &mut run.sim.rules.global_data;
        data.gravity = -1.0;
        data.ground_stiffness = float(setup[4]);
        data.structure_stiffness = float(setup[5]);
        data.default_structure_rubble_height = float(setup[6]);
        run.ents.clear();
        for o in setup[9..].chunks(4) {
            let name = NAMES[o[0].parse::<usize>().unwrap()];
            let team = TeamId(if o[1] == "0" { 1 } else { 2 });
            let tid = run
                .sim
                .rules
                .thing_factory
                .find_template_id(name)
                .expect(name);
            let slot = run.sim.spawn_object(tid, team, coord(o[2]), float(o[3]));
            let obj = run.sim.world.objects.get_mut(slot).expect("object");
            let id = obj.id;
            let module = obj.physics.as_ref().map(|p| p.mod_slot);
            if obj.ai_slot.is_some() {
                obj.ai_locomotor_set = Some(LocomotorSetType::Normal);
            }
            run.ents.push(Ent {
                slot,
                id,
                module,
                ret: 0,
                last: String::new(),
            });
        }
        run.sim.rules.global_data.gravity = gravity;
        run.sim.world.rng.init_random_with_seed(seed);

        for (k, want) in rows.iter().enumerate() {
            let query = if k == 0 {
                String::new()
            } else {
                run.apply(&ops[k - 1], &mut frame)
            };
            let got = run.row(&query);
            checked += 1;
            if got != *want {
                let op = if k == 0 { "spawn" } else { ops[k - 1].as_str() };
                failures.insert(
                    v,
                    format!("row {k} after `{op}`\n  setup {setup:?}\n  got  {got}\n  want {want}"),
                );
                break;
            }
        }
    }
    let first: Vec<String> = failures
        .iter()
        .take(2)
        .map(|(v, m)| format!("vector {v} {m}"))
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {} vectors differ ({checked} rows checked): {:?}\n{}",
        failures.len(),
        lines.len(),
        failures.keys().take(20).collect::<Vec<_>>(),
        first.join("\n")
    );
    assert!(checked > 15_000, "{checked}");
}
