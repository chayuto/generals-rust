//! Gate V2 differential verification for Wave `WP-5.3-B`:
//! `PKT-258 BattleBusSlowDeathBehavior`, `PKT-278 JetSlowDeathBehavior`,
//! `PKT-299 TransportContain`, `PKT-271 GarrisonContain`, `PKT-274 HealContain`,
//! `PKT-277 InternetHackContain`, `PKT-280 MobNexusContain`, `PKT-285 ParachuteContain`
//! against `docs/golden_vectors/WP53_B.json` (`tests/cpp_harness/gen_wp53_b.cpp`).
//!
//! `PKT-289 PrisonBehavior` is compiled out of the reference library, so it only has the
//! contract test at the bottom of this file.
//!
//! The AI, physics and stealth modules are not ported yet. The stand-ins below only parse
//! their inputs and forward lifecycle events; every compared value is computed by `zh_logic`.

use std::any::Any;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use zh_common::crc::Crc;
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UnsignedInt, UpdateSleepTime,
    FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSidecar, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::modules::behavior::battle_bus_slow_death_behavior::BattleBusSlowDeathBehavior;
use zh_logic::modules::behavior::behavior_module::UpdateScheduleTracker;
use zh_logic::modules::behavior::jet_slow_death_behavior::JetSlowDeathBehavior;
use zh_logic::modules::behavior::prison_behavior::{PrisonBehavior, PrisonVisual};
use zh_logic::modules::behavior::register_surrender_behavior_modules;
use zh_logic::modules::contain::garrison_contain::GarrisonContain;
use zh_logic::modules::contain::heal_contain::HealContain;
use zh_logic::modules::contain::internet_hack_contain::InternetHackContain;
use zh_logic::modules::contain::mob_nexus_contain::MobNexusContain;
use zh_logic::modules::contain::parachute_contain::ParachuteContain;
use zh_logic::modules::contain::transport_contain::TransportContain;
use zh_logic::modules::module::{
    AiUpdateFacet, BehaviorModule, CollideFacet, EvacDisposition, ExitDoorType, ModuleData,
    ModuleType, PhysicsFacet, StealthFacet, UpdateFacet,
};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::rules::combat_types::{DamageType, DeathType, WeaponSetType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use zh_logic::scheduler::{SleepyUpdatePhase, UPDATE_SLEEP_FOREVER};
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

// -----------------------------------------------------------------------------
// Minimal JSON reader. Numbers keep their source text so comparisons are exact.
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Json {
    Scalar(String),
    Arr(Vec<Json>),
    Obj(BTreeMap<String, Json>),
}

impl Json {
    fn obj(&self) -> &BTreeMap<String, Json> {
        match self {
            Json::Obj(m) => m,
            _ => panic!("expected JSON object"),
        }
    }

    fn arr(&self) -> &[Json] {
        match self {
            Json::Arr(a) => a,
            _ => panic!("expected JSON array"),
        }
    }

    fn text(&self) -> &str {
        match self {
            Json::Scalar(s) => s,
            _ => panic!("expected JSON scalar"),
        }
    }

    fn get(&self, key: &str) -> &Json {
        self.obj()
            .get(key)
            .unwrap_or_else(|| panic!("missing JSON key `{key}`"))
    }

    fn u32(&self, key: &str) -> u32 {
        self.get(key).text().parse().expect("u32")
    }
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn peek(&mut self) -> u8 {
        while self.b[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
        self.b[self.i]
    }

    fn eat(&mut self, c: u8) {
        assert_eq!(self.peek(), c, "at byte {}", self.i);
        self.i += 1;
    }

    fn string(&mut self) -> String {
        self.eat(b'"');
        let start = self.i;
        while self.b[self.i] != b'"' {
            assert_ne!(self.b[self.i], b'\\', "escapes are not used");
            self.i += 1;
        }
        let out = String::from_utf8(self.b[start..self.i].to_vec()).expect("utf8");
        self.i += 1;
        out
    }

    fn value(&mut self) -> Json {
        match self.peek() {
            b'{' => {
                self.i += 1;
                let mut map = BTreeMap::new();
                while self.peek() != b'}' {
                    let key = self.string();
                    self.eat(b':');
                    let value = self.value();
                    map.insert(key, value);
                    if self.peek() == b',' {
                        self.i += 1;
                    }
                }
                self.i += 1;
                Json::Obj(map)
            }
            b'[' => {
                self.i += 1;
                let mut arr = Vec::new();
                while self.peek() != b']' {
                    arr.push(self.value());
                    if self.peek() == b',' {
                        self.i += 1;
                    }
                }
                self.i += 1;
                Json::Arr(arr)
            }
            b'"' => Json::Scalar(self.string()),
            _ => {
                let start = self.i;
                while self.b[self.i] == b'-' || self.b[self.i].is_ascii_digit() {
                    self.i += 1;
                }
                assert!(self.i > start, "unsupported JSON value at byte {start}");
                Json::Scalar(String::from_utf8(self.b[start..self.i].to_vec()).expect("utf8"))
            }
        }
    }
}

fn load_golden() -> Json {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/golden_vectors/WP53_B.json");
    let raw = fs::read(path).expect("read WP53_B.json");
    Parser { b: &raw, i: 0 }.value()
}

// -----------------------------------------------------------------------------
// Stand-ins for the unported AIUpdate / PhysicsBehavior / StealthUpdate modules
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct StubData {
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    facet_mask: FacetMask,
    innate_stealth: Bool,
    mass: Real,
    shock_max: [Real; 3],
}

impl StubData {
    fn new(tag_key: NameKeyType, module_name_key: NameKeyType, facet_mask: FacetMask) -> Self {
        Self {
            tag_key,
            module_name_key,
            facet_mask,
            innate_stealth: FALSE,
            mass: 1.0,
            shock_max: [0.05, 0.025, 0.025],
        }
    }
}

impl ModuleData for StubData {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }
    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }
    fn facet_mask(&self) -> FacetMask {
        self.facet_mask
    }
}

#[derive(Clone, Copy, PartialEq)]
enum StubKind {
    Ai,
    Physics,
    Stealth,
}

struct StubModule {
    kind: StubKind,
    data: StubData,
    schedule: UpdateScheduleTracker,
}

impl BehaviorModule for StubModule {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }
    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.kind == StubKind::Ai {
            sim.ai_construct(ctx.owner_slot);
        }
    }
    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        match self.kind {
            StubKind::Ai => sim.ai_init_state_machine(ctx.owner_slot),
            StubKind::Physics => {
                if let Some(o) = sim.object_mut(ctx.owner_slot) {
                    o.physics_mass = self.data.mass;
                    o.physics_shock_max_yaw = self.data.shock_max[0];
                    o.physics_shock_max_pitch = self.data.shock_max[1];
                    o.physics_shock_max_roll = self.data.shock_max[2];
                }
            }
            StubKind::Stealth => {
                if self.data.innate_stealth != FALSE {
                    sim.set_status(ctx.owner_slot, ObjectStatusType::CanStealth);
                }
            }
        }
    }
    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        self.schedule.xfer_update_header(xfer, sim)
    }
    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }
    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }
    fn as_ai(&self) -> Option<&dyn AiUpdateFacet> {
        (self.kind == StubKind::Ai).then_some(self as &dyn AiUpdateFacet)
    }
    fn as_ai_mut(&mut self) -> Option<&mut dyn AiUpdateFacet> {
        (self.kind == StubKind::Ai).then_some(self as &mut dyn AiUpdateFacet)
    }
    fn as_physics_mut(&mut self) -> Option<&mut dyn PhysicsFacet> {
        (self.kind == StubKind::Physics).then_some(self as &mut dyn PhysicsFacet)
    }
    fn as_stealth_mut(&mut self) -> Option<&mut dyn StealthFacet> {
        (self.kind == StubKind::Stealth).then_some(self as &mut dyn StealthFacet)
    }
    fn as_collide_mut(&mut self) -> Option<&mut dyn CollideFacet> {
        (self.kind == StubKind::Physics).then_some(self as &mut dyn CollideFacet)
    }
}

impl UpdateFacet for StubModule {
    fn update(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> UpdateSleepTime {
        UpdateSleepTime::FOREVER
    }
}

impl AiUpdateFacet for StubModule {}

impl PhysicsFacet for StubModule {
    fn apply_force(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _force: &Coord3D) {}
}

impl StealthFacet for StubModule {}

impl CollideFacet for StubModule {
    /// The part of `PhysicsBehavior::onCollide` the jet depends on: reject bounding volumes
    /// that do not overlap (3D when airborne, else 2D), otherwise remember the collidee.
    fn on_collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
        _loc: Option<&Coord3D>,
        _normal: Option<&Coord3D>,
    ) {
        let airborne = sim.object_is_above_terrain(ctx.owner_slot) != FALSE;
        let (Some(me), Some(them)) = (
            sim.object(ctx.owner_slot),
            other.and_then(|o| sim.object(o)),
        ) else {
            return;
        };
        let (a, b) = (me.thing.get_position(), them.thing.get_position());
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let (dist_sqr, reach) = if airborne {
            let dz = (b.z + them.geometry.get_z_delta_to_center_position())
                - (a.z + me.geometry.get_z_delta_to_center_position());
            let reach = me.geometry.get_bounding_sphere_radius()
                + them.geometry.get_bounding_sphere_radius();
            (dx * dx + dy * dy + dz * dz, reach)
        } else {
            let reach = me.geometry.get_bounding_circle_radius()
                + them.geometry.get_bounding_circle_radius();
            (dx * dx + dy * dy, reach)
        };
        let id = them.id;
        if dist_sqr <= reach * reach {
            if let Some(me) = sim.object_mut(ctx.owner_slot) {
                me.physics_last_collidee = id;
            }
        }
    }
}

const AI_MASK: FacetMask = FacetMask(0x001 | FacetMask::AI_UPDATE.0);
const PHYSICS_MASK: FacetMask = FacetMask(0x011 | FacetMask::PHYSICS_UPDATE.0);
const STEALTH_MASK: FacetMask = FacetMask(0x001 | FacetMask::STEALTH_UPDATE.0);

fn parse_stub(
    reader: &mut IniReader<'_>,
    mut data: StubData,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("InnateStealth") {
            r.parse_bool(&mut data.innate_stealth)
        } else if field.eq_ignore_ascii_case("StealthDelay") {
            r.parse_duration_unsigned_int(&mut 0)
        } else if field.eq_ignore_ascii_case("Mass") {
            r.parse_real(&mut data.mass)
        } else if field.eq_ignore_ascii_case("ShockMaxYaw") {
            r.parse_real(&mut data.shock_max[0])
        } else if field.eq_ignore_ascii_case("ShockMaxPitch") {
            r.parse_real(&mut data.shock_max[1])
        } else if field.eq_ignore_ascii_case("ShockMaxRoll") {
            r.parse_real(&mut data.shock_max[2])
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

fn create_stub(data: &dyn ModuleData, kind: StubKind) -> Box<dyn BehaviorModule> {
    Box::new(StubModule {
        kind,
        data: data.as_any().downcast_ref::<StubData>().expect("stub").clone(),
        schedule: UpdateScheduleTracker::new(),
    })
}

fn parse_ai(
    r: &mut IniReader<'_>,
    t: NameKeyType,
    n: NameKeyType,
    _c: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_stub(r, StubData::new(t, n, AI_MASK))
}

fn parse_physics(
    r: &mut IniReader<'_>,
    t: NameKeyType,
    n: NameKeyType,
    _c: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_stub(r, StubData::new(t, n, PHYSICS_MASK))
}

fn parse_stealth(
    r: &mut IniReader<'_>,
    t: NameKeyType,
    n: NameKeyType,
    _c: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_stub(r, StubData::new(t, n, STEALTH_MASK))
}

fn create_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    create_stub(data, StubKind::Ai)
}

fn create_physics(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    create_stub(data, StubKind::Physics)
}

fn create_stealth(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    create_stub(data, StubKind::Stealth)
}

// -----------------------------------------------------------------------------
// Fixture: the exact INI text the C++ harness loads
// -----------------------------------------------------------------------------

const HARNESS_INI: &str = r#"FXList FX_BB_Start
End
FXList FX_BB_Ground
End
FXList FX_Jet_Ground
End
FXList FX_Jet_Init
End
FXList FX_Jet_Sec
End
FXList FX_Jet_Hit
End
FXList FX_Jet_Final
End
ObjectCreationList OCL_BB_Start
End
ObjectCreationList OCL_BB_Ground
End
ObjectCreationList OCL_Jet_Ground
End
ObjectCreationList OCL_Jet_Init
End
ObjectCreationList OCL_Jet_Sec
End
ObjectCreationList OCL_Jet_Hit
End
ObjectCreationList OCL_Jet_Final
End
Locomotor InfLoco
  Surfaces = GROUND
  Speed = 30
  SpeedDamaged = 20
  TurnRate = 180
  TurnRateDamaged = 180
  Acceleration = 100
  Braking = 100
  Appearance = TWO_LEGS
End
Locomotor JetLoco
  Surfaces = AIR
  Speed = 60
  SpeedDamaged = 40
  TurnRate = 90
  TurnRateDamaged = 90
  Acceleration = 60
  Braking = 60
  Appearance = WINGS
  PitchStiffness = 0.2
  RollStiffness = 0.2
  PitchDamping = 0.8
  RollDamping = 0.8
End
Locomotor ParaFreefallLoco
  Surfaces = AIR
  Speed = 20
  SpeedDamaged = 20
  TurnRate = 60
  TurnRateDamaged = 60
  Acceleration = 50
  Braking = 50
  Appearance = TWO_LEGS
  PitchStiffness = 0.1
  RollStiffness = 0.1
  PitchDamping = 0.5
  RollDamping = 0.5
End
Locomotor ParaNormalLoco
  Surfaces = AIR
  Speed = 15
  SpeedDamaged = 15
  TurnRate = 60
  TurnRateDamaged = 60
  Acceleration = 50
  Braking = 50
  Appearance = TWO_LEGS
  PitchStiffness = 0.15
  RollStiffness = 0.12
  PitchDamping = 0.85
  RollDamping = 0.82
End
Weapon Wpn_Rifle
  PrimaryDamage = 10.0
  AttackRange = 100.0
  DamageType = SMALL_ARMS
  DeathType = NORMAL
  AutoReloadsClip = Yes
  ShotsPerBarrel = 1
End
Weapon Wpn_Melee
  PrimaryDamage = 10.0
  AttackRange = 5.0
  DamageType = SMALL_ARMS
  DeathType = NORMAL
  AutoReloadsClip = Yes
  ShotsPerBarrel = 1
End
Object P53B_InfArmed
  KindOf = PRELOAD SELECTABLE CAN_ATTACK INFANTRY
  TransportSlotCount = 1
  WeaponSet
    Conditions = None
    Weapon = PRIMARY Wpn_Rifle
  End
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100.0
    InitialHealth = 100.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 5.0
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL InfLoco
End
Object P53B_InfUnarmed
  KindOf = PRELOAD SELECTABLE INFANTRY
  TransportSlotCount = 1
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100.0
    InitialHealth = 100.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 5.0
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL InfLoco
End
Object P53B_InfMelee
  KindOf = PRELOAD SELECTABLE CAN_ATTACK INFANTRY
  TransportSlotCount = 1
  WeaponSet
    Conditions = None
    Weapon = PRIMARY Wpn_Melee
  End
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100.0
    InitialHealth = 100.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 5.0
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL InfLoco
End
Object P53B_InfStealth
  KindOf = PRELOAD SELECTABLE CAN_ATTACK INFANTRY STEALTH_GARRISON
  TransportSlotCount = 1
  WeaponSet
    Conditions = None
    Weapon = PRIMARY Wpn_Rifle
  End
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100.0
    InitialHealth = 100.0
  End
  Behavior = StealthUpdate ModuleTag_Stealth
    InnateStealth = Yes
    StealthDelay = 100
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL InfLoco
End
Object P53B_InfHeavy
  KindOf = PRELOAD SELECTABLE INFANTRY
  TransportSlotCount = 2
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 120.0
    InitialHealth = 120.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 10.0
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL InfLoco
End
Object P53B_InfZeroSlot
  KindOf = PRELOAD SELECTABLE INFANTRY
  TransportSlotCount = 0
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100.0
    InitialHealth = 100.0
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL InfLoco
End
Object P53B_InfNoGarrison
  KindOf = PRELOAD SELECTABLE INFANTRY NO_GARRISON
  TransportSlotCount = 1
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100.0
    InitialHealth = 100.0
  End
End
Object P53B_VehNoTrans
  KindOf = PRELOAD SELECTABLE VEHICLE
  TransportSlotCount = 1
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200.0
    InitialHealth = 200.0
  End
End
Object P53B_Shrub
  KindOf = IMMOBILE SHRUBBERY
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100.0
    InitialHealth = 100.0
  End
End
Object P53B_BattleBus
  KindOf = PRELOAD SELECTABLE VEHICLE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200.0
    InitialHealth = 200.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 10.0
    ShockMaxYaw = 0.5
    ShockMaxPitch = 0.4
    ShockMaxRoll = 0.3
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL InfLoco
  Behavior = TransportContain ModuleTag_Contain
    Slots = 4
    DamagePercentToUnits = 10%
    AllowInsideKindOf = INFANTRY
  End
  Behavior = BattleBusSlowDeathBehavior ModuleTag_SlowDeath
    ThrowForce = 12.0
    PercentDamageToPassengers = 25%
    EmptyHulkDestructionDelay = 300
    FXStartUndeath = FX_BB_Start
    OCLStartUndeath = OCL_BB_Start
    FXHitGround = FX_BB_Ground
    OCLHitGround = OCL_BB_Ground
    DestructionDelay = 200
    SinkRate = 0.5
    ProbabilityModifier = 10
  End
End
Object P53B_BattleBusNoDelay
  KindOf = PRELOAD SELECTABLE VEHICLE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200.0
    InitialHealth = 200.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 10.0
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL InfLoco
  Behavior = TransportContain ModuleTag_Contain
    Slots = 2
    AllowInsideKindOf = INFANTRY
  End
  Behavior = BattleBusSlowDeathBehavior ModuleTag_SlowDeath
    ThrowForce = 8.0
    PercentDamageToPassengers = 10%
    EmptyHulkDestructionDelay = 0
    DestructionDelay = 150
    SinkRate = 0.25
    ProbabilityModifier = 10
  End
End
Object P53B_Jet
  KindOf = PRELOAD SELECTABLE VEHICLE AIRCRAFT
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100.0
    InitialHealth = 100.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 10.0
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL JetLoco
  Behavior = JetSlowDeathBehavior ModuleTag_JetSD
    FXOnGroundDeath = FX_Jet_Ground
    OCLOnGroundDeath = OCL_Jet_Ground
    FXInitialDeath = FX_Jet_Init
    OCLInitialDeath = OCL_Jet_Init
    DelaySecondaryFromInitialDeath = 100
    FXSecondary = FX_Jet_Sec
    OCLSecondary = OCL_Jet_Sec
    FXHitGround = FX_Jet_Hit
    OCLHitGround = OCL_Jet_Hit
    DelayFinalBlowUpFromHitGround = 150
    FXFinalBlowUp = FX_Jet_Final
    OCLFinalBlowUp = OCL_Jet_Final
    RollRate = 0.25
    RollRateDelta = 95%
    PitchRate = 0.15
    FallHowFast = 40%
    DestructionDelay = 10000
    SinkRate = 0.0
    ProbabilityModifier = 10
  End
End
Object P53B_Transport
  KindOf = PRELOAD SELECTABLE VEHICLE TRANSPORT
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 300.0
    InitialHealth = 300.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 20.0
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL InfLoco
  Behavior = TransportContain ModuleTag_Contain
    Slots = 4
    InitialPayload = P53B_InfArmed 1
    HealthRegen%PerSec = 30.0
    ExitDelay = 100
    ArmedRidersUpgradeMyWeaponSet = Yes
    GoAggressiveOnExit = Yes
    OrientLikeContainerOnExit = Yes
    KeepContainerVelocityOnExit = Yes
    ScatterNearbyOnExit = No
    ResetMoodCheckTimeOnExit = Yes
    DestroyRidersWhoAreNotFreeToExit = No
    BurnedDeathToUnits = Yes
    PassengersAllowedToFire = Yes
    AllowInsideKindOf = INFANTRY
  End
End
Object P53B_TransportAirDestroy
  KindOf = PRELOAD SELECTABLE VEHICLE TRANSPORT
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 300.0
    InitialHealth = 300.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 20.0
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL InfLoco
  Behavior = TransportContain ModuleTag_Contain
    Slots = 3
    DelayExitInAir = Yes
    DestroyRidersWhoAreNotFreeToExit = Yes
    ExitPitchRate = 0.5
    AllowInsideKindOf = INFANTRY
  End
End
Object P53B_GarrisonEncl
  KindOf = PRELOAD SELECTABLE STRUCTURE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 500.0
    InitialHealth = 500.0
  End
  Behavior = GarrisonContain ModuleTag_Garr
    ContainMax = 4
    InitialRoster = P53B_InfArmed 1
    HealObjects = Yes
    TimeForFullHeal = 300
    ImmuneToClearBuildingAttacks = No
    IsEnclosingContainer = Yes
    MobileGarrison = No
  End
End
Object P53B_GarrisonMob
  KindOf = PRELOAD SELECTABLE STRUCTURE GARRISONABLE_UNTIL_DESTROYED
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 500.0
    InitialHealth = 500.0
  End
  Behavior = GarrisonContain ModuleTag_Garr
    ContainMax = 4
    HealObjects = No
    ImmuneToClearBuildingAttacks = Yes
    IsEnclosingContainer = No
    MobileGarrison = Yes
  End
End
Object P53B_HealCont
  KindOf = PRELOAD SELECTABLE STRUCTURE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 400.0
    InitialHealth = 400.0
  End
  Behavior = HealContain ModuleTag_Heal
    ContainMax = 3
    TimeForFullHeal = 200
    AllowInsideKindOf = INFANTRY
  End
End
Object P53B_NetHack
  KindOf = PRELOAD SELECTABLE STRUCTURE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 400.0
    InitialHealth = 400.0
  End
  Behavior = InternetHackContain ModuleTag_Hack
    Slots = 3
    InitialPayload = P53B_InfUnarmed 1
    AllowInsideKindOf = INFANTRY
  End
End
Object P53B_MobNexus
  KindOf = PRELOAD SELECTABLE VEHICLE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 300.0
    InitialHealth = 300.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 15.0
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL InfLoco
  Behavior = MobNexusContain ModuleTag_Mob
    Slots = 4
    InitialPayload = P53B_InfArmed 1
    HealthRegen%PerSec = 30.0
    ScatterNearbyOnExit = Yes
    OrientLikeContainerOnExit = Yes
    KeepContainerVelocityOnExit = Yes
    ExitPitchRate = 0.4
    AllowInsideKindOf = INFANTRY
  End
End
Object P53B_Parachute
  KindOf = PRELOAD PARACHUTE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 50.0
    InitialHealth = 50.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 2.0
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_FREEFALL ParaFreefallLoco
  Locomotor = SET_NORMAL ParaNormalLoco
  Behavior = ParachuteContain ModuleTag_Para
    PitchRateMax = 30.0
    RollRateMax = 30.0
    LowAltitudeDamping = 0.2
    ParachuteOpenDist = 15.0
    KillWhenLandingInWaterSlop = 10.0
    FreeFallDamagePercent = 50%
    AllowInsideKindOf = INFANTRY
  End
End
Object P53B_ParachuteZero
  KindOf = PRELOAD PARACHUTE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 50.0
    InitialHealth = 50.0
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_FREEFALL ParaFreefallLoco
  Locomotor = SET_NORMAL ParaNormalLoco
  Behavior = ParachuteContain ModuleTag_Para
    PitchRateMax = 0.0
    RollRateMax = 0.0
    LowAltitudeDamping = 0.2
    ParachuteOpenDist = 0.0
    KillWhenLandingInWaterSlop = 10.0
    FreeFallDamagePercent = 0%
    AllowInsideKindOf = INFANTRY
  End
End
"#;

/// Splits the fixture into top-level `<Kind> <Name> ... End` blocks.
fn ini_blocks(text: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let mut block = String::new();
    for line in text.lines() {
        block.push_str(line);
        block.push('\n');
        if line == "End" {
            let mut head = block.split_whitespace();
            let kind = head.next().expect("block kind").to_owned();
            let name = head.next().expect("block name").to_owned();
            out.push((kind, name, std::mem::take(&mut block)));
        }
    }
    out
}

fn build_rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "AIUpdateInterface",
        ModuleType::Behavior,
        AI_MASK,
        SleepyUpdatePhase::Initial,
        true,
        parse_ai,
        create_ai,
    );
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "PhysicsBehavior",
        ModuleType::Behavior,
        PHYSICS_MASK,
        SleepyUpdatePhase::Physics,
        false,
        parse_physics,
        create_physics,
    );
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "StealthUpdate",
        ModuleType::Behavior,
        STEALTH_MASK,
        SleepyUpdatePhase::Normal,
        false,
        parse_stealth,
        create_stealth,
    );
    register_surrender_behavior_modules(&mut rules.module_registry, &mut rules.name_keys);

    let mut objects = String::new();
    for (kind, name, block) in ini_blocks(HARNESS_INI) {
        match kind.as_str() {
            "FXList" => {
                rules.fx_list_names.insert(name);
            }
            "ObjectCreationList" => {
                rules.ocl_names.insert(name);
            }
            "Locomotor" => {
                let mut t = LocomotorTemplate::new();
                t.parse_ini_block(&block).expect("parse locomotor");
                rules.locomotor_store.add_template(t);
            }
            "Weapon" => {
                let mut t = WeaponTemplate::new();
                t.parse_ini_block(block.trim_end()).expect("parse weapon");
                rules.weapon_store.add_template(t);
            }
            "Object" => objects.push_str(&block),
            other => panic!("unexpected fixture block {other}"),
        }
    }
    load_objects(&mut rules, &objects);
    rules
}

fn load_objects(rules: &mut RulesRegistry, text: &str) {
    let mut ini = IniReader::new("object.ini", text.as_bytes(), IniLoadType::Overwrite);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut ini) {
        let line = ini.get_line_num();
        panic!(
            "load thing templates at line {line}: {e:?} ({:?})",
            text.lines().nth(line.saturating_sub(1) as usize)
        );
    }
}

// -----------------------------------------------------------------------------
// World helpers mirroring `oracle_wp51::World` and the harness functions
// -----------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Teams {
    neutral: TeamId,
    a: TeamId,
    b: TeamId,
}

/// `World::init_with_neutral_player` on an empty world (fresh object ids, frame and PRNG).
fn reset_world(sim: &mut SimContext, teams: Teams, frame: u32, logic_seed: u32) {
    sim.world = SimWorld::new();
    let w = &mut sim.world;
    for (player, team) in [teams.neutral, teams.a, teams.b].into_iter().enumerate() {
        w.team_players.insert(team, player as Int);
        w.player_default_teams.insert(player as Int, team);
    }
    w.team_is_neutral.insert(teams.neutral, TRUE);
    w.neutral_team_id = teams.neutral;
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
    w.frame = frame;
    w.rng.init_random_with_seed(logic_seed);
}

/// `World::spawn_object`: create at the origin, then `setPosition` / `setOrientation`.
fn spawn(
    sim: &mut SimContext,
    name: &str,
    team: TeamId,
    pos: (Real, Real, Real),
    angle: Real,
) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    let slot = sim.spawn_object_with_status(
        tid,
        team,
        Coord3D::default(),
        0.0,
        ObjectStatusMaskType::new(),
    );
    sim.set_object_position(slot, &Coord3D::new(pos.0, pos.1, pos.2));
    sim.set_object_orientation(slot, angle);
    slot
}

fn module_named(sim: &mut SimContext, owner: ObjectSlot, name: &str) -> ModuleSlot {
    let key = sim.rules.name_keys.name_to_key(name);
    let obj = sim.world.objects.get(owner).expect("object");
    obj.behaviors
        .iter()
        .map(|&(ms, _)| ms)
        .find(|&ms| {
            sim.world
                .modules
                .payload(ms)
                .is_some_and(|p| p.module_name_key() == key)
        })
        .unwrap_or_else(|| panic!("missing module {name}"))
}

fn module_ref<T: 'static>(sim: &SimContext, slot: ModuleSlot) -> &T {
    sim.world
        .modules
        .payload(slot)
        .and_then(|p| p.as_any())
        .and_then(|a| a.downcast_ref::<T>())
        .expect("module type")
}

fn priority(sim: &SimContext, slot: ModuleSlot) -> u32 {
    sim.module_schedule(slot).expect("schedule").priority()
}

/// The harness's `step_single_update`.
fn step(sim: &mut SimContext, slot: ModuleSlot, frame: UnsignedInt) {
    let owner = sim.world.modules.meta(slot).owner_slot;
    if sim.world.objects.get(owner).is_none_or(|o| o.is_destroyed()) {
        return;
    }
    sim.world.frame = frame;
    if sim.module_schedule(slot).expect("schedule").next_call_frame() > frame {
        return;
    }
    let disabled = sim.world.objects.get(owner).expect("owner").disabled_mask.clone();
    let sleep = sim
        .dispatch_module(slot, |m, s, c| {
            let update = m.as_update_mut().expect("update facet");
            if update.disabled_types_to_process().test_for_all(&disabled) == FALSE {
                return UpdateSleepTime::NONE;
            }
            let sleep = update.update(s, c);
            if sleep.0 < 1 {
                UpdateSleepTime::NONE
            } else {
                sleep
            }
        })
        .expect("dispatch update");
    let next = frame.saturating_add(sleep.0).min(UPDATE_SLEEP_FOREVER);
    sim.world
        .modules
        .meta_mut(slot)
        .schedule
        .set_next_call_frame(next);
    let index = sim.world.modules.meta(slot).schedule.index_in_logic();
    if index >= 0 {
        sim.world
            .sleepy_updates
            .rebalance(&mut sim.world.modules, index as usize);
    }
}

fn hurt(sim: &mut SimContext, slot: ObjectSlot, damage_type: DamageType, amount: Real) {
    let d = damage(damage_type, amount);
    sim.attempt_damage_on(slot, &mut d.clone());
}

fn damage(damage_type: DamageType, amount: Real) -> DamageInfo {
    let mut d = DamageInfo::default();
    d.input.damage_type = damage_type;
    d.input.death_type = DeathType::Normal;
    d.input.amount = amount;
    d
}

fn health_bits(sim: &SimContext, slot: ObjectSlot) -> u32 {
    let body = sim.object(slot).expect("object").body_slot.expect("body");
    match sim.module_sidecar(body) {
        ModuleSidecar::Body { health, .. } => health.to_bits(),
        other => panic!("not a body sidecar: {other:?}"),
    }
}

fn has_status(sim: &SimContext, slot: ObjectSlot, status: ObjectStatusType) -> bool {
    sim.object(slot)
        .is_some_and(|o| o.status.test(status.as_i32()) != FALSE)
}

fn has_condition(sim: &SimContext, slot: ObjectSlot, flag: ModelConditionFlagType) -> bool {
    sim.object(slot)
        .is_some_and(|o| o.model_condition.test(flag as Int) != FALSE)
}

fn is_destroyed(sim: &SimContext, slot: ObjectSlot) -> bool {
    sim.object(slot).is_none_or(|o| o.is_destroyed())
}

fn object_id(sim: &SimContext, slot: ObjectSlot) -> ObjectId {
    sim.object(slot).expect("object").id
}

fn valid(sim: &SimContext, container: ObjectSlot, rider: ObjectSlot) -> bool {
    sim.is_valid_container_for(container, rider, TRUE) != FALSE
}

fn rng_crc(sim: &SimContext) -> u32 {
    sim.world.rng.get_game_logic_random_seed_crc()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn save_module(sim: &mut SimContext, slot: ModuleSlot) -> Vec<u8> {
    let (mut payload, _ctx, generation) =
        sim.world.modules.take_payload(slot).expect("take payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    sim.world.modules.restore_payload(slot, generation, payload);
    xs.as_bytes().to_vec()
}

/// Actual outputs of one vector, keyed like the JSON `out` object.
#[derive(Default)]
struct Out(BTreeMap<String, String>);

impl Out {
    fn put(&mut self, key: &str, value: impl std::fmt::Display) {
        assert!(
            self.0.insert(key.to_owned(), value.to_string()).is_none(),
            "duplicate output {key}"
        );
    }

    fn flag(&mut self, key: &str, value: bool) {
        self.put(key, u32::from(value));
    }

    /// The harness's `capture_snapshot` + `append_capture`.
    fn capture(&mut self, sim: &mut SimContext, slot: ModuleSlot, prefix: &str, force_hex: bool) {
        let mut xc = XferCrc::new();
        sim.world.modules.payload(slot).expect("payload").crc(&mut xc);
        let bytes = save_module(sim, slot);
        let mut save_crc = Crc::new();
        save_crc.compute_crc(&bytes, bytes.len() as Int);

        self.put(&format!("{prefix}_crc"), xc.get_crc());
        self.put(&format!("{prefix}_xfer_len"), bytes.len());
        self.put(&format!("{prefix}_xfer_fnv"), format!("{:x}", fnv1a64(&bytes)));
        self.put(&format!("{prefix}_xfer_crc"), save_crc.get());
        if force_hex || bytes.len() <= 256 {
            self.put(&format!("{prefix}_xfer_hex"), hex(&bytes));
        }
    }
}

// -----------------------------------------------------------------------------
// The eight harness sections
// -----------------------------------------------------------------------------

struct Case<'a> {
    inp: &'a Json,
    teams: Teams,
    frame: u32,
    logic_seed: u32,
    force_hex: bool,
}

impl Case<'_> {
    fn reset(&self, sim: &mut SimContext) {
        reset_world(sim, self.teams, self.frame, self.logic_seed);
    }

    fn mode(&self, key: &str) -> u32 {
        self.inp.u32(key)
    }
}

fn battle_bus(sim: &mut SimContext, case: &Case<'_>, out: &mut Out) {
    case.reset(sim);
    let (f0, mode) = (case.frame, case.mode("bb_mode"));
    let template = if mode == 1 {
        "P53B_BattleBusNoDelay"
    } else {
        "P53B_BattleBus"
    };
    let bus = spawn(sim, template, case.teams.a, (100.0, 100.0, 10.0), 0.0);
    let rider = spawn(sim, "P53B_InfArmed", case.teams.a, (100.0, 100.0, 10.0), 0.0);
    sim.contain_add(bus, rider);
    let slot = module_named(sim, bus, "BattleBusSlowDeathBehavior");

    if mode == 2 {
        hurt(sim, bus, DamageType::Explosion, 500.0);
        step(sim, slot, f0 + 1);
    } else {
        let d = damage(DamageType::Explosion, 200.0);
        sim.dispatch_module(slot, |m, s, c| {
            m.as_slow_death_mut().unwrap().begin_slow_death(s, c, &d);
        })
        .unwrap();
        step(sim, slot, f0 + 5);
        step(sim, slot, f0 + 10);
        sim.set_object_position(bus, &Coord3D::new(100.0, 100.0, 0.0));
        step(sim, slot, f0 + 11);
    }

    out.flag("bb_mid_sl", has_condition(sim, bus, ModelConditionFlagType::SecondLife));
    out.flag(
        "bb_mid_held",
        sim.object(bus).unwrap().is_disabled_by_type(DisabledType::Held) != FALSE,
    );
    out.put("bb_rider_hp", health_bits(sim, rider));
    out.put("bb_mid_pri", priority(sim, slot));
    out.put("bb_mid_rng", rng_crc(sim));
    out.capture(sim, slot, "bb_mid", case.force_hex);

    if mode == 0 {
        step(sim, slot, f0 + 12);
        sim.contain_remove(bus, rider, FALSE);
        step(sim, slot, f0 + 27);
        step(sim, slot, f0 + 42);
    }
    let module = module_ref::<BattleBusSlowDeathBehavior>(sim, slot);
    out.flag("bb_post_real", module.is_real_death != FALSE);
    out.flag("bb_post_first", module.is_in_first_death != FALSE);
    out.put("bb_post_gf", module.ground_check_frame);
    out.put("bb_post_pf", module.penalty_death_frame);
    out.put("bb_post_pri", priority(sim, slot));
    out.capture(sim, slot, "bb_post", case.force_hex);
}

fn jet(sim: &mut SimContext, case: &Case<'_>, out: &mut Out) {
    case.reset(sim);
    let (f0, mode) = (case.frame, case.mode("jet_mode"));
    let z = if mode == 0 { 0.0 } else { 80.0 };
    let plane = spawn(sim, "P53B_Jet", case.teams.a, (200.0, 200.0, z), 0.0);
    if mode == 1 {
        sim.set_status(plane, ObjectStatusType::DeckHeightOffset);
    }
    let shrub = (mode == 3)
        .then(|| spawn(sim, "P53B_Shrub", case.teams.neutral, (200.0, 200.0, 0.0), 0.0));
    let slot = module_named(sim, plane, "JetSlowDeathBehavior");
    hurt(sim, plane, DamageType::Explosion, 300.0);

    let destroyed_on_die = is_destroyed(sim, plane);
    out.flag("jet_die", destroyed_on_die);
    out.flag(
        "jet_deck_clr",
        !has_status(sim, plane, ObjectStatusType::DeckHeightOffset),
    );
    let module = module_ref::<JetSlowDeathBehavior>(sim, slot);
    out.put("jet_roll0", module.roll_rate.to_bits());
    out.put("jet_df0", module.timer_death_frame);
    out.capture(sim, slot, "jet_mid", case.force_hex);

    if !destroyed_on_die {
        step(sim, slot, f0 + 1);
        step(sim, slot, f0 + 4);
        if let Some(shrub) = shrub {
            sim.dispatch_on_collide(plane, Some(shrub), None, None);
        } else {
            sim.set_object_position(plane, &Coord3D::new(200.0, 200.0, 0.0));
        }
        step(sim, slot, f0 + 5);
    }
    let module = module_ref::<JetSlowDeathBehavior>(sim, slot);
    out.put("jet_gf_mid", module.timer_on_ground_frame);
    out.put("jet_df_mid", module.timer_death_frame);
    out.put("jet_roll_mid", module.roll_rate.to_bits());
    out.capture(sim, slot, "jet_post", case.force_hex);
    if !is_destroyed(sim, plane) {
        step(sim, slot, f0 + 12);
    }
    out.flag("jet_dest_fin", is_destroyed(sim, plane));
    out.put("jet_rng", rng_crc(sim));
}

fn transport(sim: &mut SimContext, case: &Case<'_>, out: &mut Out) {
    case.reset(sim);
    let (f0, mode) = (case.frame, case.mode("tc_mode"));
    let z = if mode == 2 { 20.0 } else { 0.0 };
    let template = if mode == 3 {
        "P53B_TransportAirDestroy"
    } else {
        "P53B_Transport"
    };
    let team_a = case.teams.a;
    let tc = spawn(sim, template, team_a, (300.0, 300.0, z), 0.75);
    let slot = module_named(sim, tc, "TransportContain");
    sim.physics_apply_force(tc, &Coord3D::new(20.0, 0.0, 0.0));

    step(sim, slot, f0 + 1);
    out.flag(
        "tc_pay",
        module_ref::<TransportContain>(sim, slot).state.payload_created != FALSE,
    );
    out.put("tc_init_cnt", sim.contain_count(tc));
    out.flag("tc_loaded", has_condition(sim, tc, ModelConditionFlagType::Loaded));
    out.flag(
        "tc_wpn_upg",
        sim.object(tc)
            .unwrap()
            .weapon_set_flags
            .test(WeaponSetType::PlayerUpgrade.as_i32())
            != FALSE,
    );

    let enemy = spawn(sim, "P53B_InfArmed", case.teams.b, (310.0, 300.0, 0.0), 0.0);
    let zero = spawn(sim, "P53B_InfZeroSlot", team_a, (312.0, 300.0, 0.0), 0.0);
    let vehicle = spawn(sim, "P53B_VehNoTrans", team_a, (314.0, 300.0, 0.0), 0.0);
    let heavy = spawn(sim, "P53B_InfHeavy", team_a, (316.0, 300.0, 0.0), 0.0);
    let melee = spawn(sim, "P53B_InfMelee", team_a, (318.0, 300.0, 0.0), 0.0);
    let para = spawn(sim, "P53B_Parachute", team_a, (320.0, 300.0, 30.0), 0.0);
    let para_rider = spawn(sim, "P53B_InfUnarmed", team_a, (320.0, 300.0, 30.0), 0.0);
    sim.contain_add(para, para_rider);

    let mask = [enemy, zero, vehicle, heavy, para]
        .into_iter()
        .enumerate()
        .fold(0u32, |m, (bit, r)| m | (u32::from(valid(sim, tc, r)) << bit));
    out.put("tc_valid", mask);

    hurt(sim, heavy, DamageType::SmallArms, 40.0);
    for rider in [heavy, melee] {
        if valid(sim, tc, rider) {
            sim.contain_add(tc, rider);
        }
    }
    out.put(
        "tc_extra",
        module_ref::<TransportContain>(sim, slot).state.extra_slots_in_use,
    );
    let heavy_id = object_id(sim, heavy);
    let vehicle_id = object_id(sim, vehicle);
    out.put(
        "tc_h_fire",
        sim.contain_is_passenger_allowed_to_fire(tc, heavy_id),
    );
    out.put(
        "tc_v_fire",
        sim.contain_is_passenger_allowed_to_fire(tc, vehicle_id),
    );

    step(sim, slot, f0 + 2);
    out.put("tc_h_hp", health_bits(sim, heavy));
    out.capture(sim, slot, "tc_mid", case.force_hex);

    let tmpl_id = sim
        .rules
        .thing_factory
        .find_template_id("P53B_InfHeavy")
        .expect("template id");
    let door = sim
        .dispatch_module(slot, |m, s, c| {
            m.as_exit_mut()
                .unwrap()
                .reserve_door_for_exit(s, c, tmpl_id, Some(heavy))
        })
        .unwrap();
    if door != ExitDoorType::DoorNoneAvailable {
        sim.exit_object_via_door(tc, heavy, door);
    }
    out.put("tc_door", door as i32);
    let busy = sim
        .dispatch_module(slot, |m, s, c| m.as_exit().unwrap().is_exit_busy(s, c))
        .unwrap();
    out.put("tc_busy", busy);
    out.put(
        "tc_nb_f",
        module_ref::<TransportContain>(sim, slot).state.frame_exit_not_busy,
    );
    out.put("tc_h_fall", sim.object(heavy).unwrap().physics_allow_to_fall);

    if mode == 1 {
        let (a, b) = (case.teams.a, case.teams.b);
        sim.dispatch_module(slot, |m, s, c| m.on_capture(s, c, a, b))
            .unwrap();
    } else if mode == 3 {
        hurt(sim, tc, DamageType::Explosion, 500.0);
    }
    out.put("tc_post_cnt", sim.contain_count(tc));
    out.put("tc_rng", rng_crc(sim));
}

fn garrison(sim: &mut SimContext, case: &Case<'_>, out: &mut Out) {
    case.reset(sim);
    let (f0, mode) = (case.frame, case.mode("gc_mode"));
    let template = if mode == 2 {
        "P53B_GarrisonMob"
    } else {
        "P53B_GarrisonEncl"
    };
    let team_a = case.teams.a;
    let gc = spawn(sim, template, case.teams.neutral, (400.0, 400.0, 0.0), 0.5);
    let slot = module_named(sim, gc, "GarrisonContain");
    out.put("gc_ros", sim.contain_count(gc));

    sim.contain_remove_all(gc, FALSE);
    let stealth = spawn(sim, "P53B_InfStealth", team_a, (410.0, 400.0, 0.0), 0.0);
    let infantry = spawn(sim, "P53B_InfArmed", team_a, (412.0, 400.0, 0.0), 0.0);
    let no_garrison = spawn(sim, "P53B_InfNoGarrison", team_a, (414.0, 400.0, 0.0), 0.0);
    out.flag("gc_nogarr", valid(sim, gc, no_garrison));

    hurt(sim, stealth, DamageType::SmallArms, 40.0);
    sim.contain_add(gc, stealth);
    out.put(
        "gc_hide_s",
        module_ref::<GarrisonContain>(sim, slot).hide_garrisoned_state_from_nonallies,
    );
    out.put("gc_team", sim.object(gc).unwrap().team_id.0);
    let module = module_ref::<GarrisonContain>(sim, slot);
    out.put("gc_orig_team", module.original_team.map_or(0, |t| t.0));
    out.put("gc_pts_init", module.points_initialized);
    out.flag("gc_atk", has_status(sim, gc, ObjectStatusType::CanAttack));

    sim.contain_add(gc, infantry);
    out.put(
        "gc_hide_r",
        module_ref::<GarrisonContain>(sim, slot).hide_garrisoned_state_from_nonallies,
    );
    step(sim, slot, f0 + 1);
    out.put("gc_st_hp", health_bits(sim, stealth));
    out.capture(sim, slot, "gc_mid", case.force_hex);

    let disposition = match mode {
        0 => EvacDisposition::BurstFromCenter,
        1 => EvacDisposition::ToLeft,
        _ => EvacDisposition::ToRight,
    };
    sim.dispatch_module(slot, |m, _, _| {
        m.as_contain_mut().unwrap().set_evac_disposition(disposition);
    })
    .unwrap();
    if mode < 2 {
        sim.exit_object_via_door(gc, infantry, ExitDoorType::Door1);
    } else {
        sim.contain_remove_all(gc, TRUE);
    }

    out.put("gc_rally", module_ref::<GarrisonContain>(sim, slot).rally_valid);
    out.put("gc_post_cnt", sim.contain_count(gc));
    hurt(sim, gc, DamageType::Explosion, 360.0);
    out.flag(
        "gc_valid_rd",
        valid(sim, gc, no_garrison) || valid(sim, gc, infantry),
    );
    out.capture(sim, slot, "gc_post", case.force_hex);
    out.put("gc_rng", rng_crc(sim));
}

fn heal(sim: &mut SimContext, case: &Case<'_>, out: &mut Out) {
    case.reset(sim);
    let f0 = case.frame;
    let hc = spawn(sim, "P53B_HealCont", case.teams.a, (500.0, 500.0, 0.0), 0.0);
    let slot = module_named(sim, hc, "HealContain");
    let infantry = spawn(sim, "P53B_InfArmed", case.teams.a, (510.0, 500.0, 0.0), 0.0);
    hurt(sim, infantry, DamageType::SmallArms, 50.0);
    sim.contain_add(hc, infantry);

    step(sim, slot, f0 + 2);
    out.put("hc_mid_hp", health_bits(sim, infantry));
    out.put("hc_mid_cnt", sim.contain_count(hc));
    out.capture(sim, slot, "hc_mid", case.force_hex);

    let last = if case.mode("hc_sel") == 0 {
        f0 + 5
    } else {
        f0 + 6
    };
    for f in f0 + 3..=last {
        step(sim, slot, f);
    }
    out.put("hc_post_hp", health_bits(sim, infantry));
    out.put("hc_post_cnt", sim.contain_count(hc));
}

fn internet_hack(sim: &mut SimContext, case: &Case<'_>, out: &mut Out) {
    case.reset(sim);
    let center = spawn(sim, "P53B_NetHack", case.teams.a, (600.0, 600.0, 0.0), 0.0);
    let slot = module_named(sim, center, "InternetHackContain");
    step(sim, slot, case.frame + 1);
    let hacker = spawn(sim, "P53B_InfUnarmed", case.teams.a, (610.0, 600.0, 0.0), 0.0);
    sim.contain_add(center, hacker);
    out.put("ih_cnt", sim.contain_count(center));
    out.capture(sim, slot, "ih", case.force_hex);
}

fn mob_nexus(sim: &mut SimContext, case: &Case<'_>, out: &mut Out) {
    case.reset(sim);
    let team_a = case.teams.a;
    let mob = spawn(sim, "P53B_MobNexus", team_a, (700.0, 700.0, 0.0), 0.25);
    let slot = module_named(sim, mob, "MobNexusContain");
    sim.physics_apply_force(mob, &Coord3D::new(15.0, 0.0, 0.0));
    out.put("mn_init_cnt", sim.contain_count(mob));

    let ally = spawn(sim, "P53B_InfStealth", team_a, (710.0, 700.0, 0.0), 0.0);
    let enemy = spawn(sim, "P53B_InfArmed", case.teams.b, (712.0, 700.0, 0.0), 0.0);
    let neutral = spawn(sim, "P53B_InfArmed", case.teams.neutral, (714.0, 700.0, 0.0), 0.0);
    let heavy = spawn(sim, "P53B_InfHeavy", team_a, (716.0, 700.0, 0.0), 0.0);
    let para = spawn(sim, "P53B_Parachute", team_a, (718.0, 700.0, 30.0), 0.0);
    let para_rider = spawn(sim, "P53B_InfUnarmed", team_a, (718.0, 700.0, 30.0), 0.0);
    sim.contain_add(para, para_rider);

    let mask = [ally, enemy, neutral, heavy, para]
        .into_iter()
        .enumerate()
        .fold(0u32, |m, (bit, r)| m | (u32::from(valid(sim, mob, r)) << bit));
    out.put("mn_valid", mask);

    hurt(sim, ally, DamageType::SmallArms, 30.0);
    sim.contain_add(mob, ally);
    sim.contain_add(mob, heavy);
    step(sim, slot, case.frame + 1);
    out.put("mn_ally_hp", health_bits(sim, ally));
    out.put(
        "mn_extra_mid",
        module_ref::<MobNexusContain>(sim, slot).extra_slots_in_use,
    );
    out.capture(sim, slot, "mn_mid", case.force_hex);

    let expose = case.mode("mn_exp") as Bool;
    let evacuated = sim
        .dispatch_module(slot, |m, s, c| {
            m.as_transport_passenger_mut()
                .unwrap()
                .try_to_evacuate(s, c, expose)
        })
        .unwrap();
    out.put("mn_evac", evacuated);
    out.put("mn_post_cnt", sim.contain_count(mob));
    out.put(
        "mn_extra_post",
        module_ref::<MobNexusContain>(sim, slot).extra_slots_in_use,
    );
    out.flag("mn_st_det", has_status(sim, ally, ObjectStatusType::Detected));
    out.put("mn_rng", rng_crc(sim));
}

fn parachute(sim: &mut SimContext, case: &Case<'_>, out: &mut Out) {
    case.reset(sim);
    let (f0, mode) = (case.frame, case.mode("pc_mode"));
    let template = if mode == 3 {
        "P53B_ParachuteZero"
    } else {
        "P53B_Parachute"
    };
    let z0 = if mode == 2 { 10.0 } else { 40.0 };
    let team_a = case.teams.a;
    let chute = spawn(sim, template, team_a, (800.0, 800.0, z0), 0.25);
    let slot = module_named(sim, chute, "ParachuteContain");
    let rider = spawn(sim, "P53B_InfArmed", team_a, (800.0, 800.0, z0), 0.0);
    let rider2 = spawn(sim, "P53B_InfUnarmed", team_a, (800.0, 800.0, z0), 0.0);
    let zero = spawn(sim, "P53B_InfZeroSlot", team_a, (800.0, 800.0, z0), 0.0);

    out.flag("pc_v_zero", valid(sim, chute, zero));
    sim.contain_add(chute, rider);
    out.flag("pc_v_sec", valid(sim, chute, rider2));

    if mode == 1 {
        let dest = Coord3D::new(825.0, 815.0, 0.0);
        sim.dispatch_module(slot, |m, _, _| {
            m.as_contain_mut().unwrap().set_override_destination(&dest);
        })
        .unwrap();
    }

    step(sim, slot, f0 + 1);
    let drop_z = if mode == 2 { 5.0 } else { 15.0 };
    sim.set_object_position(chute, &Coord3D::new(800.0, 800.0, drop_z));
    step(sim, slot, f0 + 2);
    step(sim, slot, f0 + 3);

    let module = module_ref::<ParachuteContain>(sim, slot);
    out.put("pc_open", module.opened);
    out.put("pc_ovr", module.is_landing_override_set);
    out.put("pc_pitch", module.pitch.to_bits());
    out.put("pc_roll", module.roll.to_bits());
    out.put("pc_prate", module.pitch_rate.to_bits());
    out.put("pc_rrate", module.roll_rate.to_bits());
    out.put("pc_sz", module.start_z.to_bits());
    out.flag("pc_nocoll", has_status(sim, chute, ObjectStatusType::NoCollisions));
    out.capture(sim, slot, "pc_mid", case.force_hex);

    if mode == 2 {
        sim.set_object_position(chute, &Coord3D::new(800.0, 800.0, 60.0));
        hurt(sim, chute, DamageType::Explosion, 200.0);
    } else {
        let ground = Coord3D::new(800.0, 800.0, 0.0);
        sim.set_object_position(chute, &ground);
        let normal = Coord3D::new(0.0, 0.0, 1.0);
        sim.dispatch_module(slot, |m, s, c| {
            m.as_collide_mut()
                .unwrap()
                .on_collide(s, c, None, Some(&ground), Some(&normal));
        })
        .unwrap();
    }
    out.put("pc_post_cnt", sim.contain_count(chute));
    out.put("pc_rhp", health_bits(sim, rider));
    out.put("pc_rfall", sim.object(rider).unwrap().physics_allow_to_fall);
    out.put("pc_dead", sim.object(chute).unwrap().is_effectively_dead());
    out.put("pc_rng", rng_crc(sim));
}

const SECTIONS: [fn(&mut SimContext, &Case<'_>, &mut Out); 8] = [
    battle_bus,
    jet,
    transport,
    garrison,
    heal,
    internet_hack,
    mob_nexus,
    parachute,
];

/// `seed0_diagnostics` key -> `out` key of vector 0.
const DIAGNOSTIC_ALIASES: [(&str, &str); 15] = [
    ("bb_mid_second_life", "bb_mid_sl"),
    ("bb_mid_held", "bb_mid_held"),
    ("bb_rider_hp_bits", "bb_rider_hp"),
    ("bb_post_real", "bb_post_real"),
    ("jet_dest_on_die", "jet_die"),
    ("jet_dest_final", "jet_dest_fin"),
    ("tc_init_count", "tc_init_cnt"),
    ("tc_valid_mask", "tc_valid"),
    ("gc_team_after_garr", "gc_team"),
    ("gc_orig_team_id", "gc_orig_team"),
    ("hc_mid_hp_bits", "hc_mid_hp"),
    ("hc_post_cnt", "hc_post_cnt"),
    ("mn_valid_mask", "mn_valid"),
    ("pc_opened", "pc_open"),
    ("pc_post_cnt", "pc_post_cnt"),
];

#[test]
fn golden_wp53_b_all_1000_seeds() {
    let doc = load_golden();
    assert_eq!(doc.get("packet").text(), "WP-5.3-B");
    let diag = doc.get("seed0_diagnostics");
    let vectors = doc.get("vectors").arr();
    assert_eq!(doc.u32("vector_count"), 1000);
    assert_eq!(vectors.len(), 1000);
    // Every diagnostic is checked: three team ids against the inputs, the rest against seed 0.
    assert_eq!(diag.obj().len(), DIAGNOSTIC_ALIASES.len() + 3);

    let mut sim = SimContext::new();
    sim.rules = build_rules();

    // Mismatches are collected so one run reports every failing field.
    let mut failures: BTreeMap<String, (u32, u32, String)> = BTreeMap::new();
    for (index, vector) in vectors.iter().enumerate() {
        let seed = vector.u32("seed");
        assert_eq!(seed as usize, index);
        let inp = vector.get("in");
        let case = Case {
            inp,
            teams: Teams {
                neutral: TeamId(inp.u32("team_n") as i32),
                a: TeamId(inp.u32("team_a") as i32),
                b: TeamId(inp.u32("team_b") as i32),
            },
            frame: inp.u32("frame"),
            logic_seed: inp.u32("logic_seed"),
            force_hex: seed <= 8,
        };

        let mut out = Out::default();
        for section in SECTIONS {
            section(&mut sim, &case, &mut out);
        }

        let expected = vector.get("out").obj();
        for (key, value) in expected {
            let actual = out.0.get(key).map_or("<not produced>", String::as_str);
            if actual != value.text() {
                let entry = failures.entry(key.clone()).or_insert_with(|| {
                    (0, seed, format!("expected {} got {actual}", value.text()))
                });
                entry.0 += 1;
            }
        }
        for key in out.0.keys() {
            assert!(expected.contains_key(key), "seed {seed}: unexpected output {key}");
        }

        if seed == 0 {
            for (name, input) in [
                ("team_neutral_id", "team_n"),
                ("team_a_id", "team_a"),
                ("team_b_id", "team_b"),
            ] {
                assert_eq!(diag.get(name).text(), inp.get(input).text(), "{name}");
            }
            for (name, key) in DIAGNOSTIC_ALIASES {
                assert_eq!(out.0[key], diag.get(name).text(), "seed0 diagnostic {name}");
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} fields mismatch (field: count, first seed, detail):\n{}",
        failures.len(),
        failures
            .iter()
            .map(|(k, (n, s, d))| format!("{k}: {n} first {s}: {}", &d[..d.len().min(7800)]))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// -----------------------------------------------------------------------------
// Contract tests (not covered by the vectors)
// -----------------------------------------------------------------------------

const TEAMS: Teams = Teams {
    neutral: TeamId(3),
    a: TeamId(1),
    b: TeamId(2),
};

const PRISON_INI: &str = "\
Object P53B_Prison\n\
  KindOf = STRUCTURE SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 500\n    InitialHealth = 500\n  End\n\
  Behavior = PrisonBehavior ModuleTag_Prison\n\
    ContainMax = 3\n\
    ShowPrisoners = Yes\n\
    YardBonePrefix = YARD\n\
  End\n\
End\n";

/// Replaces the module in `slot` by `fresh`, loads `slot`'s saved bytes into it and saves again.
fn reload(
    sim: &mut SimContext,
    slot: ModuleSlot,
    fresh: Box<dyn BehaviorModule>,
) -> (Vec<u8>, Vec<u8>) {
    let before = save_module(sim, slot);
    let (_old, _ctx, generation) = sim.world.modules.take_payload(slot).expect("take payload");
    let mut fresh = fresh;
    let mut xl = XferLoad::new(&before);
    fresh.xfer(&mut xl, sim).expect("xfer load");
    sim.world.modules.restore_payload(slot, generation, fresh);
    sim.dispatch_module(slot, |m, s, c| m.load_post_process(s, c))
        .expect("post process");
    let after = save_module(sim, slot);
    (before, after)
}

#[test]
fn wp53_b_xfer_load_round_trips() {
    let mut sim = SimContext::new();
    sim.rules = build_rules();
    reset_world(&mut sim, TEAMS, 40, 7);

    macro_rules! round_trip {
        ($template:expr, $name:expr, $ty:ident, $riders:expr) => {{
            let owner = spawn(&mut sim, $template, TEAMS.a, (400.0, 400.0, 0.0), 0.5);
            for rider_template in $riders {
                let rider = spawn(&mut sim, rider_template, TEAMS.a, (410.0, 400.0, 0.0), 0.0);
                sim.contain_add(owner, rider);
            }
            let slot = module_named(&mut sim, owner, $name);
            let data = module_ref::<$ty>(&sim, slot).data.clone();
            let (before, after) = reload(&mut sim, slot, Box::new($ty::new(data)));
            assert_eq!(hex(&after), hex(&before), "{}", $name);
        }};
    }

    let none: [&str; 0] = [];
    round_trip!("P53B_Jet", "JetSlowDeathBehavior", JetSlowDeathBehavior, none);
    round_trip!(
        "P53B_BattleBus",
        "BattleBusSlowDeathBehavior",
        BattleBusSlowDeathBehavior,
        none
    );
    round_trip!(
        "P53B_Transport",
        "TransportContain",
        TransportContain,
        ["P53B_InfHeavy"]
    );
    round_trip!(
        "P53B_NetHack",
        "InternetHackContain",
        InternetHackContain,
        ["P53B_InfUnarmed"]
    );
    round_trip!("P53B_HealCont", "HealContain", HealContain, ["P53B_InfArmed"]);
    round_trip!(
        "P53B_MobNexus",
        "MobNexusContain",
        MobNexusContain,
        ["P53B_InfHeavy"]
    );
    round_trip!(
        "P53B_Parachute",
        "ParachuteContain",
        ParachuteContain,
        ["P53B_InfArmed"]
    );
    // The mobile garrison has no initial roster, so the reloaded list starts empty.
    round_trip!(
        "P53B_GarrisonMob",
        "GarrisonContain",
        GarrisonContain,
        ["P53B_InfStealth", "P53B_InfArmed"]
    );
}

#[test]
fn prison_behavior_surrender_contract() {
    // `ALLOW_SURRENDER` is off in the shipped game: the default registry cannot create it.
    let default_rules = RulesRegistry::new();
    assert!(default_rules
        .module_registry
        .find_descriptor("PrisonBehavior", ModuleType::Behavior)
        .is_none_or(|d| d.create_module.is_none()));

    let mut rules = build_rules();
    load_objects(&mut rules, PRISON_INI);
    let mut sim = SimContext::new();
    sim.rules = rules;
    reset_world(&mut sim, TEAMS, 40, 11);

    let prison = spawn(&mut sim, "P53B_Prison", TEAMS.a, (100.0, 100.0, 0.0), 0.0);
    let first = spawn(&mut sim, "P53B_InfArmed", TEAMS.a, (110.0, 100.0, 0.0), 0.0);
    let second = spawn(&mut sim, "P53B_InfUnarmed", TEAMS.a, (112.0, 100.0, 0.0), 0.0);
    let slot = module_named(&mut sim, prison, "PrisonBehavior");

    // Each shown prisoner draws one orientation from the logic stream.
    let mut expected_rng = sim.world.rng.clone();
    for _ in 0..2 {
        expected_rng.get_game_logic_random_value_real(0.0, zh_common::math::TWO_PI);
    }
    sim.contain_add(prison, first);
    sim.contain_add(prison, second);
    assert_eq!(rng_crc(&sim), expected_rng.get_game_logic_random_seed_crc());

    let held = |sim: &SimContext, o: ObjectSlot| {
        sim.object(o).unwrap().is_disabled_by_type(DisabledType::Held) != FALSE
    };
    assert!(held(&sim, first) && held(&sim, second));
    let (first_id, second_id) = (object_id(&sim, first), object_id(&sim, second));
    let visual = |object_id| PrisonVisual {
        object_id,
        drawable_id: 0,
    };
    assert_eq!(
        module_ref::<PrisonBehavior>(&sim, slot).visuals,
        [visual(second_id), visual(first_id)]
    );

    // Tail of the snapshot: u16 count, then (object id, drawable id) from the list head.
    let bytes = save_module(&mut sim, slot);
    let mut tail = 2u16.to_le_bytes().to_vec();
    for id in [second_id, first_id] {
        tail.extend(id.0.to_le_bytes());
        tail.extend(0u32.to_le_bytes());
    }
    assert!(bytes.ends_with(&tail), "{}", hex(&bytes));

    // Loading pushes each entry on the head, so the list comes back reversed (as in C++).
    let data = module_ref::<PrisonBehavior>(&sim, slot).data.clone();
    let (before, after) = reload(&mut sim, slot, Box::new(PrisonBehavior::new(data)));
    assert_eq!(before.len(), after.len());
    assert_eq!(before[..before.len() - tail.len()], after[..after.len() - tail.len()]);
    assert_eq!(
        module_ref::<PrisonBehavior>(&sim, slot).visuals,
        [visual(first_id), visual(second_id)]
    );

    sim.contain_remove(prison, first, FALSE);
    assert!(!held(&sim, first) && held(&sim, second));
    assert_eq!(sim.contain_count(prison), 1);
    assert_eq!(
        module_ref::<PrisonBehavior>(&sim, slot).visuals,
        [visual(second_id)]
    );
}
