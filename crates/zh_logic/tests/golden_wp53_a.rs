//! Gate V2 differential verification for Wave `WP-5.3-A`:
//! `PKT-259 BehaviorModule`, `PKT-265 ContainModule`, `PKT-282 OpenContain`,
//! `PKT-295 SlowDeathBehavior`, `PKT-276 InstantDeathBehavior`, `PKT-257 AutoHealBehavior`,
//! `PKT-268 FireWeaponWhenDamagedBehavior`, `PKT-269 FireWeaponWhenDeadBehavior`,
//! `PKT-273 GrantStealthBehavior`.

use std::any::Any;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{
    Bool, Int, NameKeyType, Real, Relationship, TeamId, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSidecar, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::modules::behavior::auto_heal_behavior::AutoHealBehavior;
use zh_logic::modules::behavior::behavior_module::UpdateScheduleTracker;
use zh_logic::modules::behavior::grant_stealth_behavior::GrantStealthBehavior;
use zh_logic::modules::behavior::register_behavior_modules;
use zh_logic::modules::behavior::slow_death_behavior::SlowDeathBehavior;
use zh_logic::modules::contain::register_contain_modules;
use zh_logic::modules::module::{
    AiUpdateFacet, BehaviorModule, ModuleData, ModuleType, ObjectEnterExitType, 
    StealthFacet, UpdateFacet,
};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use zh_logic::rules::upgrade::UpgradeMaskType;
use zh_logic::scheduler::{SleepyUpdatePhase, UPDATE_SLEEP_FOREVER};
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, PRIVATE_STATUS_OFF_MAP};

// -----------------------------------------------------------------------------
// Minimal Deterministic JSON Parser for `docs/golden_vectors/WP53_A.json`
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum JsonVal {
    Null,
    Bool(bool),
    Num(i64, u64),
    Str(String),
    Arr(Vec<JsonVal>),
    Obj(BTreeMap<String, JsonVal>),
}

impl JsonVal {
    fn as_obj(&self) -> &BTreeMap<String, JsonVal> {
        match self {
            JsonVal::Obj(m) => m,
            _ => panic!("expected JSON object, got {self:?}"),
        }
    }

    fn as_arr(&self) -> &[JsonVal] {
        match self {
            JsonVal::Arr(a) => a,
            _ => panic!("expected JSON array, got {self:?}"),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            JsonVal::Str(s) => s,
            _ => panic!("expected JSON string, got {self:?}"),
        }
    }

    fn as_u32(&self) -> u32 {
        match self {
            JsonVal::Num(_, u) => *u as u32,
            _ => panic!("expected JSON u32, got {self:?}"),
        }
    }

    fn as_i32(&self) -> i32 {
        match self {
            JsonVal::Num(i, _) => *i as i32,
            _ => panic!("expected JSON i32, got {self:?}"),
        }
    }

    fn get(&self, key: &str) -> &JsonVal {
        self.as_obj()
            .get(key)
            .unwrap_or_else(|| panic!("missing JSON key `{key}`"))
    }
}

struct JsonParser<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> JsonParser<'a> {
    fn skip_ws(&mut self) {
        while self.i < self.b.len() && self.b[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }

    fn parse(&mut self) -> JsonVal {
        self.skip_ws();
        match self.b[self.i] {
            b'{' => {
                self.i += 1;
                let mut map = BTreeMap::new();
                self.skip_ws();
                if self.b[self.i] == b'}' {
                    self.i += 1;
                    return JsonVal::Obj(map);
                }
                loop {
                    let key = self.parse_string();
                    self.skip_ws();
                    assert_eq!(self.b[self.i], b':');
                    self.i += 1;
                    let val = self.parse();
                    map.insert(key, val);
                    self.skip_ws();
                    if self.b[self.i] == b'}' {
                        self.i += 1;
                        return JsonVal::Obj(map);
                    }
                    assert_eq!(self.b[self.i], b',');
                    self.i += 1;
                }
            }
            b'[' => {
                self.i += 1;
                let mut arr = Vec::new();
                self.skip_ws();
                if self.b[self.i] == b']' {
                    self.i += 1;
                    return JsonVal::Arr(arr);
                }
                loop {
                    arr.push(self.parse());
                    self.skip_ws();
                    if self.b[self.i] == b']' {
                        self.i += 1;
                        return JsonVal::Arr(arr);
                    }
                    assert_eq!(self.b[self.i], b',');
                    self.i += 1;
                }
            }
            b'"' => JsonVal::Str(self.parse_string()),
            b't' => {
                self.i += 4;
                JsonVal::Bool(true)
            }
            b'f' => {
                self.i += 5;
                JsonVal::Bool(false)
            }
            b'n' => {
                self.i += 4;
                JsonVal::Null
            }
            b'-' | b'0'..=b'9' => {
                let start = self.i;
                if self.b[self.i] == b'-' {
                    self.i += 1;
                }
                while self.i < self.b.len() && self.b[self.i].is_ascii_digit() {
                    self.i += 1;
                }
                let s = std::str::from_utf8(&self.b[start..self.i]).unwrap();
                let iv = s.parse::<i64>().unwrap_or(0);
                let uv = s.parse::<u64>().unwrap_or(iv as u64);
                JsonVal::Num(iv, uv)
            }
            other => panic!("unexpected byte `{}` at {}", other as char, self.i),
        }
    }

    fn parse_string(&mut self) -> String {
        self.skip_ws();
        assert_eq!(self.b[self.i], b'"');
        self.i += 1;
        let mut out = String::new();
        while self.b[self.i] != b'"' {
            if self.b[self.i] == b'\\' {
                self.i += 1;
                match self.b[self.i] {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    c => out.push(c as char),
                }
                self.i += 1;
            } else {
                out.push(self.b[self.i] as char);
                self.i += 1;
            }
        }
        self.i += 1;
        out
    }
}

fn load_golden_json() -> JsonVal {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/golden_vectors/WP53_A.json");
    let raw = fs::read(path).expect("read WP53_A.json");
    JsonParser { b: &raw, i: 0 }.parse()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(hex.len() / 2);
    let nib = |b: u8| -> u8 {
        match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => 10 + (b - b'a'),
            b'A'..=b'F' => 10 + (b - b'A'),
            _ => 0,
        }
    };
    let bs = hex.as_bytes();
    for pair in bs.chunks_exact(2) {
        out.push((nib(pair[0]) << 4) | nib(pair[1]));
    }
    out
}

// -----------------------------------------------------------------------------
// Lightweight Test Facet Stubs (`AIUpdateInterface`, `StealthUpdate`, `PhysicsBehavior`)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct TestModuleData {
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    facet_mask: FacetMask,
    innate_stealth: Bool,
    granted_by_sp: Bool,
}

impl ModuleData for TestModuleData {
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

struct TestAIUpdate {
    data: TestModuleData,
    ai_dead: Bool,
    enter_target: Option<ObjectSlot>,
    schedule: UpdateScheduleTracker,
}

impl BehaviorModule for TestAIUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }
    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }
    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }
    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        self.schedule.xfer_update_header(xfer, sim)
    }
    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
    }
    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }
    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }
    fn as_ai(&self) -> Option<&dyn AiUpdateFacet> {
        Some(self)
    }
    fn as_ai_mut(&mut self) -> Option<&mut dyn AiUpdateFacet> {
        Some(self)
    }
}

impl UpdateFacet for TestAIUpdate {
    fn update(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> UpdateSleepTime {
        UpdateSleepTime::NONE
    }
}

impl AiUpdateFacet for TestAIUpdate {
    fn is_ai_in_dead_state(&self) -> Bool {
        self.ai_dead
    }
    fn mark_as_dead(&mut self) {
        self.ai_dead = TRUE;
    }
    fn get_enter_target(&self) -> Option<ObjectSlot> {
        self.enter_target
    }
}

fn parse_test_ai_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_field, _r| Ok(()))?;
    Ok(Box::new(TestModuleData {
        tag_key,
        module_name_key,
        facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0),
        innate_stealth: FALSE,
        granted_by_sp: FALSE,
    }))
}

fn create_test_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let d = data.as_any().downcast_ref::<TestModuleData>().unwrap().clone();
    Box::new(TestAIUpdate {
        data: d,
        ai_dead: FALSE,
        enter_target: None,
        schedule: UpdateScheduleTracker::new(),
    })
}

struct TestStealthUpdate {
    data: TestModuleData,
    enabled: Bool,
    schedule: UpdateScheduleTracker,
}

impl BehaviorModule for TestStealthUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }
    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        if self.data.innate_stealth != FALSE {
            sim.set_status(ctx.owner_slot, ObjectStatusType::CanStealth);
        }
        if self.data.granted_by_sp != FALSE {
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
        } else {
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
        }
    }
    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }
    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        self.schedule.xfer_update_header(xfer, sim)
    }
    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
    }
    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }
    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }
    fn as_stealth_mut(&mut self) -> Option<&mut dyn StealthFacet> {
        Some(self)
    }
}

impl UpdateFacet for TestStealthUpdate {
    fn update(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> UpdateSleepTime {
        UpdateSleepTime::NONE
    }
}

impl StealthFacet for TestStealthUpdate {
    fn mark_as_detected(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, _frames: UnsignedInt) {
        sim.clear_status(ctx.owner_slot, ObjectStatusType::Stealthed);
    }
    fn receive_grant(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        active: Bool,
        _frames: UnsignedInt,
    ) {
        self.enabled = active;
        sim.set_status(ctx.owner_slot, ObjectStatusType::CanStealth);
        sim.set_status(ctx.owner_slot, ObjectStatusType::Stealthed);
    }
}

fn parse_test_stealth_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut innate = TRUE;
    let mut granted = FALSE;
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("InnateStealth") {
            r.parse_bool(&mut innate)
        } else if field.eq_ignore_ascii_case("GrantedBySpecialPower") {
            r.parse_bool(&mut granted)
        } else if field.eq_ignore_ascii_case("StealthDelay") {
            let mut tmp = 0;
            r.parse_duration_unsigned_int(&mut tmp)
        } else {
            Ok(())
        }
    })?;
    Ok(Box::new(TestModuleData {
        tag_key,
        module_name_key,
        facet_mask: FacetMask(0x001 | FacetMask::STEALTH_UPDATE.0),
        innate_stealth: innate,
        granted_by_sp: granted,
    }))
}

fn create_test_stealth(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let d = data.as_any().downcast_ref::<TestModuleData>().unwrap().clone();
    Box::new(TestStealthUpdate {
        data: d,
        enabled: TRUE,
        schedule: UpdateScheduleTracker::new(),
    })
}

// -----------------------------------------------------------------------------
// Shared INI & Harness Helpers
// -----------------------------------------------------------------------------

const UPGRADE_INI: &str = "\
Upgrade UpgHeal\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n\
Upgrade UpgOld\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 50\nEnd\n\
Upgrade UpgWpn\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n\
Upgrade UpgConflict\n  Type = PLAYER\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n\
Upgrade UpgObjConflict\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n\
";

const ARMOR_INI: &str = "\
Armor SecondLifeArmor\n  Armor = Default 50%\nEnd\n\
";

const WEAPON_BLOCKS: &[&str] = &[
    "Weapon Wpn_SD_A\n  PrimaryDamage = 0.0\n  PrimaryDamageRadius = 0.0\n  DamageType = EXPLOSION\n  DeathType = NORMAL\n  AutoReloadsClip = Yes\n  ShotsPerBarrel = 1\nEnd",
    "Weapon Wpn_SD_B\n  PrimaryDamage = 0.0\n  PrimaryDamageRadius = 0.0\n  DamageType = FLAME\n  DeathType = BURNED\n  AutoReloadsClip = Yes\n  ShotsPerBarrel = 1\nEnd",
    "Weapon Wpn_ReactPris\n  PrimaryDamage = 0.0\n  PrimaryDamageRadius = 0.0\n  DamageType = SMALL_ARMS\n  DeathType = NORMAL\n  ClipSize = 2\n  ClipReloadTime = 300\n  DelayBetweenShots = 100\n  AutoReloadsClip = Yes\n  ShotsPerBarrel = 1\nEnd",
    "Weapon Wpn_ReactDmg\n  PrimaryDamage = 0.0\n  PrimaryDamageRadius = 0.0\n  DamageType = SMALL_ARMS\n  DeathType = NORMAL\n  ClipSize = 3\n  ClipReloadTime = 0\n  DelayBetweenShots = 100\n  AutoReloadsClip = Yes\n  ShotsPerBarrel = 1\nEnd",
    "Weapon Wpn_ReactReal\n  PrimaryDamage = 0.0\n  PrimaryDamageRadius = 0.0\n  DamageType = SMALL_ARMS\n  DeathType = NORMAL\n  ClipSize = 1\n  ClipReloadTime = 150\n  DelayBetweenShots = 0\n  AutoReloadsClip = Yes\n  ShotsPerBarrel = 1\nEnd",
    "Weapon Wpn_ReactRubble\n  PrimaryDamage = 0.0\n  PrimaryDamageRadius = 0.0\n  DamageType = SMALL_ARMS\n  DeathType = NORMAL\n  ClipSize = 1\n  ClipReloadTime = 0\n  AutoReloadsClip = No\n  ShotsPerBarrel = 1\nEnd",
    "Weapon Wpn_ContPris\n  PrimaryDamage = 0.0\n  PrimaryDamageRadius = 0.0\n  DamageType = EXPLOSION\n  DeathType = NORMAL\n  ClipSize = 2\n  ClipReloadTime = 200\n  DelayBetweenShots = 100\n  AutoReloadsClip = Yes\n  ShotsPerBarrel = 1\nEnd",
    "Weapon Wpn_ContDmg\n  PrimaryDamage = 0.0\n  PrimaryDamageRadius = 0.0\n  DamageType = EXPLOSION\n  DeathType = NORMAL\n  ClipSize = 2\n  ClipReloadTime = 100\n  DelayBetweenShots = 100\n  AutoReloadsClip = Yes\n  ShotsPerBarrel = 1\nEnd",
    "Weapon Wpn_Death\n  PrimaryDamage = 15.0\n  PrimaryDamageRadius = 40.0\n  RadiusDamageAffects = ENEMIES\n  DamageType = EXPLOSION\n  DeathType = EXPLODED\n  AutoReloadsClip = Yes\n  ShotsPerBarrel = 1\nEnd",
];

const OBJECT_INI: &str = "\
Object P53_Inf\n\
  KindOf = INFANTRY SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
End\n\
Object P53_Stealth\n\
  KindOf = INFANTRY STEALTH_GARRISON SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 80\n    InitialHealth = 80\n  End\n\
  Behavior = StealthUpdate ModuleTag_Stealth\n    StealthDelay = 100\n    InnateStealth = Yes\n  End\n\
End\n\
Object P53_Grantable\n\
  KindOf = INFANTRY SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 80\n    InitialHealth = 80\n  End\n\
  Behavior = StealthUpdate ModuleTag_Stealth\n    StealthDelay = 100\n    InnateStealth = No\n    GrantedBySpecialPower = Yes\n  End\n\
End\n\
Object P53_Hero\n\
  KindOf = INFANTRY HERO SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n\
End\n\
Object P53_VehAI\n\
  KindOf = VEHICLE SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 150\n    InitialHealth = 150\n  End\n\
  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n\
End\n\
Object P53_Struct\n\
  KindOf = STRUCTURE SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 300\n    InitialHealth = 300\n  End\n\
End\n\
Object OC_T0\n\
  KindOf = STRUCTURE SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 500\n    InitialHealth = 500\n  End\n\
  Behavior = OpenContain ModuleTag_Contain\n\
    ContainMax = 3\n\
    DamagePercentToUnits = 30%\n\
    BurnedDeathToUnits = Yes\n\
    AllowInsideKindOf = INFANTRY VEHICLE\n\
    ForbidInsideKindOf = HERO\n\
    PassengersAllowedToFire = No\n\
    DoorOpenTime = 100\n\
    NumberOfExitPaths = 2\n\
    WeaponBonusPassedToPassengers = Yes\n\
    AllowAlliesInside = Yes\n\
    AllowEnemiesInside = No\n\
    AllowNeutralInside = No\n\
    DeathTypes = ALL -CRUSHED\n\
  End\n\
End\n\
Object OC_T1\n\
  KindOf = VEHICLE SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 400\n    InitialHealth = 400\n  End\n\
  Behavior = OpenContain ModuleTag_Contain\n\
    ContainMax = 4\n\
    DamagePercentToUnits = 50%\n\
    BurnedDeathToUnits = No\n\
    PassengersAllowedToFire = Yes\n\
    DoorOpenTime = 200\n\
    AllowAlliesInside = Yes\n\
    AllowEnemiesInside = Yes\n\
    AllowNeutralInside = Yes\n\
  End\n\
End\n\
Object SD_Single\n\
  KindOf = INFANTRY SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
  Behavior = SlowDeathBehavior ModuleTag_SD\n\
    DeathTypes = ALL -BURNED\n\
    ProbabilityModifier = 10\n\
    ModifierBonusPerOverkillPercent = 25%\n\
    SinkRate = 60.0\n\
    SinkDelay = 100\n\
    SinkDelayVariance = 100\n\
    DestructionDelay = 400\n\
    DestructionDelayVariance = 200\n\
    FX = INITIAL FX_SD_InitA FX_SD_InitB\n\
    FX = MIDPOINT FX_SD_MidA\n\
    FX = FINAL FX_SD_FinalA\n\
    OCL = INITIAL OCL_SD_A OCL_SD_B\n\
    OCL = FINAL OCL_SD_A\n\
    Weapon = INITIAL Wpn_SD_A Wpn_SD_B\n\
    Weapon = MIDPOINT Wpn_SD_B\n\
  End\n\
End\n\
Object SD_Dual_AI\n\
  KindOf = VEHICLE SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n\
  Behavior = SlowDeathBehavior ModuleTag_SDA\n\
    DeathTypes = ALL -CRUSHED\n\
    ProbabilityModifier = 10\n\
    ModifierBonusPerOverkillPercent = 10%\n\
    SinkRate = 30.0\n\
    SinkDelay = 60\n\
    SinkDelayVariance = 90\n\
    DestructionDelay = 300\n\
    DestructionDelayVariance = 150\n\
    FX = INITIAL FX_SD_InitA\n\
    OCL = MIDPOINT OCL_SD_B\n\
  End\n\
  Behavior = SlowDeathBehavior ModuleTag_SDB\n\
    DeathTypes = ALL\n\
    ProbabilityModifier = 20\n\
    ModifierBonusPerOverkillPercent = 50%\n\
    SinkRate = 45.0\n\
    SinkDelay = 100\n\
    SinkDelayVariance = 100\n\
    DestructionDelay = 360\n\
    DestructionDelayVariance = 120\n\
    Weapon = FINAL Wpn_SD_B\n\
  End\n\
End\n\
Object SD_Undead\n\
  KindOf = VEHICLE SELECTABLE\n\
  ArmorSet\n    Conditions = SECOND_LIFE\n    Armor = SecondLifeArmor\n  End\n\
  Body = UndeadBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n    SecondLifeMaxHealth = 60\n  End\n\
  Behavior = SlowDeathBehavior ModuleTag_SDA\n\
    ProbabilityModifier = 12\n\
    ModifierBonusPerOverkillPercent = 20%\n\
    SinkRate = 30.0\n\
    SinkDelay = 90\n\
    DestructionDelay = 300\n\
    FX = INITIAL FX_SD_InitA\n\
  End\n\
  Behavior = SlowDeathBehavior ModuleTag_SDB\n\
    ProbabilityModifier = 18\n\
    SinkDelay = 120\n\
    DestructionDelay = 360\n\
    OCL = FINAL OCL_SD_A\n\
  End\n\
End\n\
Object SD_Fling\n\
  KindOf = INFANTRY SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
  Behavior = SlowDeathBehavior ModuleTag_SD\n\
    ProbabilityModifier = 10\n\
    SinkRate = 30.0\n\
    SinkDelay = 90\n\
    DestructionDelay = 300\n\
    FlingForce = 5.0\n\
    FlingForceVariance = 2.0\n\
    FlingPitch = 30.0\n\
    FlingPitchVariance = 15.0\n\
  End\n\
End\n\
Object ID_Multi\n\
  KindOf = VEHICLE SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
  Behavior = InstantDeathBehavior ModuleTag_ID\n\
    DeathTypes = ALL -CRUSHED\n\
    FX = FX_ID_A FX_ID_B\n\
    OCL = OCL_ID_A OCL_ID_B\n\
    Weapon = Wpn_SD_A Wpn_SD_B\n\
  End\n\
End\n\
Object AH_Self\n\
  KindOf = VEHICLE SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n\
  Behavior = AutoHealBehavior ModuleTag_AH\n\
    StartsActive = No\n\
    TriggeredBy = UpgHeal\n\
    RemovesUpgrades = UpgOld\n\
    FXListUpgrade = FX_Upg_Heal\n\
    HealingAmount = 15\n\
    HealingDelay = 200\n\
    StartHealingDelay = 100\n\
    Radius = 0.0\n\
  End\n\
End\n\
Object AH_RadiusCont\n\
  KindOf = INFANTRY SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n\
  Behavior = AutoHealBehavior ModuleTag_AH\n\
    StartsActive = Yes\n\
    SingleBurst = No\n\
    HealingAmount = 20\n\
    HealingDelay = 100\n\
    Radius = 100.0\n\
    KindOf = INFANTRY VEHICLE\n\
    ForbiddenKindOf = STRUCTURE\n\
    SkipSelfForHealing = No\n\
  End\n\
End\n\
Object AH_RadiusBurst\n\
  KindOf = INFANTRY SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n\
  Behavior = AutoHealBehavior ModuleTag_AH\n\
    StartsActive = Yes\n\
    SingleBurst = Yes\n\
    HealingAmount = 25\n\
    HealingDelay = 100\n\
    Radius = 100.0\n\
    KindOf = INFANTRY VEHICLE\n\
    ForbiddenKindOf = STRUCTURE\n\
    SkipSelfForHealing = Yes\n\
  End\n\
End\n\
Object AH_WholePlayer\n\
  KindOf = STRUCTURE SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n\
  Behavior = AutoHealBehavior ModuleTag_AH\n\
    StartsActive = Yes\n\
    AffectsWholePlayer = Yes\n\
    SkipSelfForHealing = Yes\n\
    HealingAmount = 12\n\
    HealingDelay = 150\n\
    KindOf = INFANTRY VEHICLE\n\
    ForbiddenKindOf = HERO\n\
  End\n\
End\n\
Object FWD_Test\n\
  KindOf = VEHICLE SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
  Behavior = FireWeaponWhenDamagedBehavior ModuleTag_FWD\n\
    StartsActive = No\n\
    TriggeredBy = UpgWpn\n\
    FXListUpgrade = FX_Upg_Wpn\n\
    DamageTypes = ALL -EXPLOSION\n\
    DamageAmount = 10.0\n\
    ReactionWeaponPristine = Wpn_ReactPris\n\
    ReactionWeaponDamaged = Wpn_ReactDmg\n\
    ReactionWeaponReallyDamaged = Wpn_ReactReal\n\
    ReactionWeaponRubble = Wpn_ReactRubble\n\
    ContinuousWeaponPristine = Wpn_ContPris\n\
    ContinuousWeaponDamaged = Wpn_ContDmg\n\
  End\n\
End\n\
Object FWD_Active\n\
  KindOf = VEHICLE SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
  Behavior = FireWeaponWhenDamagedBehavior ModuleTag_FWD\n\
    StartsActive = Yes\n\
    DamageTypes = ALL\n\
    DamageAmount = 5.0\n\
    ReactionWeaponPristine = Wpn_ReactDmg\n\
    ContinuousWeaponPristine = Wpn_ContDmg\n\
  End\n\
End\n\
Object FWDead_Test\n\
  KindOf = STRUCTURE SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
  Behavior = FireWeaponWhenDeadBehavior ModuleTag_FWDead\n\
    StartsActive = No\n\
    TriggeredBy = UpgWpn\n\
    ConflictsWith = UpgConflict UpgObjConflict\n\
    DeathTypes = ALL -SUICIDED\n\
    DeathWeapon = Wpn_Death\n\
  End\n\
End\n\
Object GS_Emitter\n\
  KindOf = STRUCTURE SELECTABLE\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
  Behavior = GrantStealthBehavior ModuleTag_GS\n\
    StartRadius = 10.0\n\
    FinalRadius = 70.0\n\
    RadiusGrowRate = 25.0\n\
    KindOf = INFANTRY VEHICLE\n\
  End\n\
End\n\
";

const TEAM_NEUTRAL: TeamId = TeamId(0);
const TEAM_A: TeamId = TeamId(1);
const TEAM_B: TeamId = TeamId(2);

fn build_rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    register_behavior_modules(&mut rules.module_registry, &mut rules.name_keys);
    register_contain_modules(&mut rules.module_registry, &mut rules.name_keys);
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "AIUpdateInterface",
        ModuleType::Behavior,
        FacetMask(0x001 | FacetMask::AI_UPDATE.0),
        SleepyUpdatePhase::Initial,
        true,
        parse_test_ai_data,
        create_test_ai,
    );
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "StealthUpdate",
        ModuleType::Behavior,
        FacetMask(0x001 | FacetMask::STEALTH_UPDATE.0),
        SleepyUpdatePhase::Normal,
        false,
        parse_test_stealth_data,
        create_test_stealth,
    );

    rules
        .upgrade_center
        .load_from_ini(
            &mut IniReader::new("upgrade.ini", UPGRADE_INI.as_bytes(), IniLoadType::Overwrite),
            &mut rules.name_keys,
        )
        .expect("load upgrades");

    rules
        .armor_store
        .load_from_ini(
            &mut IniReader::new("armor.ini", ARMOR_INI.as_bytes(), IniLoadType::Overwrite),
            &mut rules.name_keys,
        )
        .expect("parse armor");

    for fx in [
        "FX_SD_InitA",
        "FX_SD_InitB",
        "FX_SD_MidA",
        "FX_SD_FinalA",
        "FX_ID_A",
        "FX_ID_B",
        "FX_Upg_Heal",
        "FX_Upg_Wpn",
    ] {
        rules.fx_list_names.insert(fx.to_owned());
    }
    for ocl in ["OCL_SD_A", "OCL_SD_B", "OCL_ID_A", "OCL_ID_B"] {
        rules.ocl_names.insert(ocl.to_owned());
    }

    for block in WEAPON_BLOCKS {
        let mut wt = WeaponTemplate::new();
        wt.parse_ini_block(block).expect("parse weapon");
        rules.weapon_store.add_template(wt);
    }

    let mut obj_ini = IniReader::new("object.ini", OBJECT_INI.as_bytes(), IniLoadType::Overwrite);
    rules
        .load_thing_templates_from_ini(&mut obj_ini)
        .unwrap_or_else(|e| {
            let line = OBJECT_INI
                .lines()
                .nth(obj_ini.get_line_num().saturating_sub(1) as usize)
                .unwrap_or("");
            panic!(
                "load thing templates at line {}: {:?} ({:?})",
                obj_ini.get_line_num(),
                e,
                line
            );
        });
    rules
}

fn new_sim(rules: RulesRegistry) -> SimContext {
    let mut sim = SimContext::new();
    // The harness world: 20-unit partition cells over its 1000 by 1000 terrain.
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
    sim.rules = rules;
    sim.world.team_players.insert(TEAM_NEUTRAL, 0);
    sim.world.team_players.insert(TEAM_A, 1);
    sim.world.team_players.insert(TEAM_B, 2);
    sim.world.team_is_neutral.insert(TEAM_NEUTRAL, TRUE);
    for i in 0..3 {
        for j in 0..3 {
            let rel = if i == j {
                Relationship::Allies
            } else if i == 0 || j == 0 {
                Relationship::Neutral
            } else {
                Relationship::Enemies
            };
            sim.world.player_relationships.insert((i, j), rel);
        }
    }
    sim
}

fn spawn(sim: &mut SimContext, name: &str, team: TeamId, pos: Coord3D) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    sim.spawn_object_with_status(tid, team, pos, 0.0, ObjectStatusMaskType::new())
}

fn spawn_with_status(
    sim: &mut SimContext,
    name: &str,
    team: TeamId,
    pos: Coord3D,
    status: ObjectStatusMaskType,
) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    sim.spawn_object_with_status(tid, team, pos, 0.0, status)
}

fn find_module_by_name(
    sim: &mut SimContext,
    owner: ObjectSlot,
    name: &str,
    mut skip: usize,
) -> Option<ModuleSlot> {
    let key = sim.rules.name_keys.name_to_key(name);
    let obj = sim.world.objects.get(owner)?;
    for &(ms, _) in &obj.behaviors {
        let Some(p) = sim.world.modules.payload(ms) else {
            continue;
        };
        if p.module_name_key() == key {
            if skip == 0 {
                return Some(ms);
            }
            skip -= 1;
        }
    }
    None
}

fn body_health(sim: &SimContext, owner: ObjectSlot) -> Real {
    let bs = sim.world.objects.get(owner).unwrap().body_slot.unwrap();
    sim.module_sidecar(bs).as_body_health().unwrap()
}

fn body_health_bits(sim: &SimContext, owner: ObjectSlot) -> u32 {
    body_health(sim, owner).to_bits()
}

fn save_module_hex(sim: &mut SimContext, slot: ModuleSlot) -> String {
    let (mut payload, _ctx, gen_id) = sim.world.modules.take_payload(slot).expect("take payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    sim.world.modules.restore_payload(slot, gen_id, payload);
    bytes_to_hex(xs.as_bytes())
}

fn crc_module(sim: &mut SimContext, slot: ModuleSlot) -> u32 {
    let (payload, _ctx, gen_id) = sim.world.modules.take_payload(slot).expect("take payload");
    let mut xc = XferCrc::new();
    payload.crc(&mut xc);
    sim.world.modules.restore_payload(slot, gen_id, payload);
    xc.get_crc()
}

fn load_module_bytes(sim: &mut SimContext, slot: ModuleSlot, bytes: &[u8]) {
    let (mut payload, _ctx, gen_id) = sim.world.modules.take_payload(slot).expect("take payload");
    let mut xl = XferLoad::new(bytes);
    payload.xfer(&mut xl, sim).expect("xfer load");
    sim.world.modules.restore_payload(slot, gen_id, payload);
}

fn load_post_process_module(sim: &mut SimContext, slot: ModuleSlot) {
    let _ = sim.dispatch_module(slot, |m, s, c| {
        m.load_post_process(s, c);
    });
}

fn step_single_update(sim: &mut SimContext, mod_slot: ModuleSlot, frame: UnsignedInt) {
    let owner = sim.world.modules.meta(mod_slot).owner_slot;
    if sim.world.objects.get(owner).is_none_or(|o| o.is_destroyed()) {
        return;
    }
    sim.world.frame = frame;
    let Some(sched) = sim.module_schedule(mod_slot) else {
        return;
    };
    if sched.next_call_frame() > frame {
        return;
    }
    let disabled_mask = sim
        .world
        .objects
        .get(owner)
        .map(|o| o.disabled_mask.clone())
        .unwrap_or_default();
    let sleep_len = sim
        .dispatch_module(mod_slot, |m, s, c| {
            let Some(u) = m.as_update_mut() else {
                return UpdateSleepTime::NONE;
            };
            if u.disabled_types_to_process().test_for_all(&disabled_mask) != FALSE {
                let mut sl = u.update(s, c);
                if sl.0 < 1 {
                    sl = UpdateSleepTime::NONE;
                }
                sl
            } else {
                UpdateSleepTime::NONE
            }
        })
        .unwrap_or(UpdateSleepTime::NONE);

    let mut next = frame.saturating_add(sleep_len.0);
    if next > UPDATE_SLEEP_FOREVER {
        next = UPDATE_SLEEP_FOREVER;
    }
    sim.world
        .modules
        .meta_mut(mod_slot)
        .schedule
        .set_next_call_frame(next);
    let idx = sim.world.modules.meta(mod_slot).schedule.index_in_logic();
    if idx >= 0 {
        sim.world
            .sleepy_updates
            .rebalance(&mut sim.world.modules, idx as usize);
    }
}

fn destroy_and_flush_all(sim: &mut SimContext) {
    let slots: Vec<ObjectSlot> = sim.world.objects.world_iter().collect();
    for s in slots {
        if sim.world.objects.get(s).is_some_and(|o| !o.is_destroyed()) {
            sim.world.enqueue_destroy(s);
        }
    }
    sim.process_destroy_list();
    sim.world.weapon_fires.clear();
    sim.world.stealth_detected_events.clear();
    sim.world.ai_exit_orders.clear();
}

trait SidecarBodyExt {
    fn as_body_health(&self) -> Option<Real>;
}

impl SidecarBodyExt for ModuleSidecar {
    fn as_body_health(&self) -> Option<Real> {
        match *self {
            ModuleSidecar::Body { health, .. } => Some(health),
            _ => None,
        }
    }
}

fn hurt(sim: &mut SimContext, slot: ObjectSlot, dmg_type: DamageType, death_type: DeathType, amount: Real) {
    let mut d = DamageInfo::default();
    d.input.damage_type = dmg_type;
    d.input.death_type = death_type;
    d.input.amount = amount;
    sim.attempt_damage_on(slot, &mut d);
}

#[test]
fn golden_wp53_a_all_1000_seeds() {
    let doc = load_golden_json();
    let diag = doc.get("seed0_diagnostics");
    let vecs = doc.get("vectors").as_arr();
    assert_eq!(vecs.len(), 1000);

    let rules = build_rules();
    let mut sim = new_sim(rules);

    for v in vecs.iter() {
        let seed = v.get("seed").as_u32();
        let inp = v.get("in");
        let exp = v.get("out");

        destroy_and_flush_all(&mut sim);
        sim.world.next_object_id = 1;
        sim.world.player_upgrades.clear();

        let frame0 = inp.get("frame").as_u32();
        let logic_seed = inp.get("logic_seed").as_u32();
        sim.world.frame = frame0;
        sim.world.rng.init_random_with_seed(logic_seed);

        // =====================================================================
        // 1) OpenContain (PKT-282)
        // =====================================================================
        let oc_use_t1 = inp.get("oc_t1").as_u32() != 0;
        let oc_expose = inp.get("oc_expose").as_u32() != 0;
        let oc_action = inp.get("oc_action").as_i32();
        let oc_allow_fire_set = inp.get("oc_allow_fire").as_u32() != 0;
        let oc_wanter_kill = inp.get("oc_wanter_kill").as_u32() != 0;

        let oc_obj = spawn(
            &mut sim,
            if oc_use_t1 { "OC_T1" } else { "OC_T0" },
            TEAM_A,
            Coord3D::new(100.0, 100.0, 0.0),
        );
        let p_inf = spawn(&mut sim, "P53_Inf", TEAM_A, Coord3D::new(100.0, 100.0, 0.0));
        let p_st = spawn(&mut sim, "P53_Stealth", TEAM_B, Coord3D::new(100.0, 100.0, 0.0));
        let p_hero = spawn(&mut sim, "P53_Hero", TEAM_A, Coord3D::new(100.0, 100.0, 0.0));
        let p_neu = spawn(&mut sim, "P53_VehAI", TEAM_NEUTRAL, Coord3D::new(100.0, 100.0, 0.0));
        let p_ally_st = spawn(&mut sim, "P53_Stealth", TEAM_A, Coord3D::new(100.0, 100.0, 0.0));

        let oc_slot = find_module_by_name(&mut sim, oc_obj, "OpenContain", 0).unwrap();

        let mut valid_mask = 0u32;
        if sim.is_valid_container_for(oc_obj, p_inf, TRUE) != FALSE { valid_mask |= 1; }
        if sim.is_valid_container_for(oc_obj, p_st, TRUE) != FALSE { valid_mask |= 2; }
        if sim.is_valid_container_for(oc_obj, p_hero, TRUE) != FALSE { valid_mask |= 4; }
        if sim.is_valid_container_for(oc_obj, p_neu, TRUE) != FALSE { valid_mask |= 8; }
        if sim.is_valid_container_for(oc_obj, p_ally_st, TRUE) != FALSE { valid_mask |= 16; }
        assert_eq!(valid_mask, exp.get("oc_valid").as_u32(), "seed {seed} oc_valid");

        let oc_obj_id = sim.world.objects.id_of(oc_obj).unwrap();
        let _ = sim.dispatch_module(oc_slot, |m, s, c| {
            let oc = m.as_contain_mut().unwrap();
            oc.add_to_contain(s, c, p_inf);
            oc.add_to_contain(s, c, p_ally_st);
            if oc_use_t1 {
                oc.add_to_contain(s, c, p_st);
                oc.add_to_contain(s, c, p_neu);
            }
        });
        let p_neu_id = sim.world.objects.id_of(p_neu).unwrap();
        let p_hero_id = sim.world.objects.id_of(p_hero).unwrap();
        let _ = sim.dispatch_module(oc_slot, |m, s, c| {
            let oc = m.as_contain_mut().unwrap();
            oc.on_object_wants_to_enter_or_exit(p_hero_id, ObjectEnterExitType::WantsToEnter);
            oc.on_object_wants_to_enter_or_exit(p_neu_id, ObjectEnterExitType::WantsToExit);
            if oc_allow_fire_set {
                oc.set_passenger_allowed_to_fire(TRUE);
            }
            let exit = m.as_exit_mut().unwrap();
            exit.set_rally_point(&Coord3D::new(150.0, 200.0, 0.0));
            let mut nat = Coord3D::default();
            exit.get_natural_rally_point(s, c, &mut nat, TRUE);
        });

        let (oc_pre_count, oc_pre_stealth, oc_pre_hero, oc_pre_pm, oc_pre_ids) = {
            let oc = sim.world.modules.payload(oc_slot).unwrap().as_contain().unwrap();
            let ctx = sim.world.modules.meta(oc_slot).context(oc_slot);
            (
                oc.get_contain_count(&sim, ctx),
                oc.get_stealth_units_contained(),
                oc.get_hero_units_contained(&sim, ctx),
                oc.get_player_entered_mask(),
                oc.get_contained_items(&sim, ctx)
                    .iter()
                    .map(|id| id.0 as u32)
                    .collect::<Vec<u32>>(),
            )
        };
        let oc_pre_crc = crc_module(&mut sim, oc_slot);
        let oc_pre_hex = save_module_hex(&mut sim, oc_slot);

        let exp_pre_ids: Vec<u32> = exp.get("oc_pre_ids").as_arr().iter().map(|x| x.as_u32()).collect();
        assert_eq!(oc_pre_count, exp.get("oc_pre_cnt").as_u32(), "seed {seed} oc_pre_cnt");
        assert_eq!(oc_pre_stealth, exp.get("oc_pre_st").as_u32(), "seed {seed} oc_pre_st");
        assert_eq!(oc_pre_hero, exp.get("oc_pre_hr").as_u32(), "seed {seed} oc_pre_hr");
        assert_eq!(oc_pre_pm as u32, exp.get("oc_pre_pm").as_u32(), "seed {seed} oc_pre_pm");
        assert_eq!(oc_pre_ids, exp_pre_ids, "seed {seed} oc_pre_ids");
        assert_eq!(oc_pre_crc, exp.get("oc_pre_crc").as_u32(), "seed {seed} oc_pre_crc");
        assert_eq!(oc_pre_hex, exp.get("oc_pre_xfer").as_str(), "seed {seed} oc_pre_xfer");

        if oc_wanter_kill {
            hurt(&mut sim, p_hero, DamageType::Unresistable, DeathType::Normal, 999.0);
        }
        step_single_update(&mut sim, oc_slot, frame0 + 1);
        let (oc_mid_pm, oc_mid_wanters) = {
            let oc = sim.world.modules.payload(oc_slot).unwrap().as_contain().unwrap();
            (
                oc.get_player_entered_mask(),
                oc.has_objects_wanting_to_enter_or_exit() != FALSE,
            )
        };
        assert_eq!(oc_mid_pm as u32, exp.get("oc_mid_pm").as_u32(), "seed {seed} oc_mid_pm");
        assert_eq!(
            u32::from(oc_mid_wanters),
            exp.get("oc_mid_w").as_u32(),
            "seed {seed} oc_mid_w"
        );

        let expose_b = if oc_expose { TRUE } else { FALSE };
        match oc_action {
            0 => {
                let _ = sim.dispatch_module(oc_slot, |m, s, c| {
                    let oc = m.as_contain_mut().unwrap();
                    oc.mark_all_passengers_detected(s, c);
                    oc.remove_from_contain(s, c, p_ally_st, expose_b);
                    oc.remove_all_contained(s, c, expose_b);
                });
            }
            1 => {
                let mut hd = DamageInfo::default();
                hd.input.damage_type = DamageType::Unresistable;
                hd.input.death_type = DeathType::Exploded;
                hd.input.source_id = oc_obj_id;
                hd.input.amount = 35.0;
                let _ = sim.dispatch_module(oc_slot, |m, s, c| {
                    m.as_contain_mut().unwrap().harm_and_force_exit_all_contained(s, c, &hd);
                });
            }
            2 => {
                let _ = sim.dispatch_module(oc_slot, |m, s, c| {
                    m.as_contain_mut().unwrap().kill_all_contained(s, c);
                });
            }
            _ => {
                let crush = (seed % 8) == 7;
                hurt(
                    &mut sim,
                    oc_obj,
                    if crush { DamageType::Crush } else { DamageType::Explosion },
                    if crush { DeathType::Crushed } else { DeathType::Normal },
                    1000.0,
                );
            }
        }

        let (oc_post_count, oc_post_stealth, oc_post_hero) = {
            let oc = sim.world.modules.payload(oc_slot).unwrap().as_contain().unwrap();
            let ctx = sim.world.modules.meta(oc_slot).context(oc_slot);
            (
                oc.get_contain_count(&sim, ctx),
                oc.get_stealth_units_contained(),
                oc.get_hero_units_contained(&sim, ctx),
            )
        };
        let inf_hp_bits = body_health_bits(&sim, p_inf);
        let inf_contained = sim.world.objects.get(p_inf).is_some_and(|o| o.contained_by.is_some());
        let oc_post_crc = crc_module(&mut sim, oc_slot);
        let oc_post_hex = save_module_hex(&mut sim, oc_slot);

        assert_eq!(oc_post_count, exp.get("oc_post_cnt").as_u32(), "seed {seed} oc_post_cnt");
        assert_eq!(oc_post_stealth, exp.get("oc_post_st").as_u32(), "seed {seed} oc_post_st");
        assert_eq!(oc_post_hero, exp.get("oc_post_hr").as_u32(), "seed {seed} oc_post_hr");
        assert_eq!(inf_hp_bits, exp.get("oc_inf_hp_bits").as_u32(), "seed {seed} oc_inf_hp_bits");
        assert_eq!(u32::from(inf_contained), exp.get("oc_inf_cont").as_u32(), "seed {seed} oc_inf_cont");
        assert_eq!(oc_post_crc, exp.get("oc_post_crc").as_u32(), "seed {seed} oc_post_crc");
        assert_eq!(oc_post_hex, exp.get("oc_post_xfer").as_str(), "seed {seed} oc_post_xfer");

        // XferLoad round-trip
        let oc_rt = spawn(
            &mut sim,
            if oc_use_t1 { "OC_T1" } else { "OC_T0" },
            TEAM_A,
            Coord3D::new(100.0, 100.0, 0.0),
        );
        let oc_rt_slot = find_module_by_name(&mut sim, oc_rt, "OpenContain", 0).unwrap();
        load_module_bytes(&mut sim, oc_rt_slot, &hex_to_bytes(&oc_post_hex));
        load_post_process_module(&mut sim, oc_rt_slot);
        let oc_rt_post_hex = save_module_hex(&mut sim, oc_rt_slot);
        if seed == 0 {
            assert_eq!(
                u32::from(oc_rt_post_hex == oc_post_hex),
                diag.get("oc_rt_match").as_u32(),
                "seed 0 oc_rt_match"
            );
        }
        if oc_post_count == 0 && oc_post_stealth > 0 {
            // `killAllContained` leaves stale `stealth_units_contained` which `OpenContain::loadPostProcess` clears.
            assert_eq!(&oc_rt_post_hex[46..54], "00000000", "seed {seed} oc_rt cleared stealth count");
            assert_eq!(&oc_rt_post_hex[..46], &oc_post_hex[..46], "seed {seed} oc_rt prefix");
            assert_eq!(&oc_rt_post_hex[54..], &oc_post_hex[54..], "seed {seed} oc_rt suffix");
        } else {
            assert_eq!(oc_rt_post_hex, oc_post_hex, "seed {seed} oc_rt");
        }

        // OpenContain::onCollide matching C++ setup
        let oc_col = spawn(&mut sim, "OC_T0", TEAM_A, Coord3D::new(120.0, 100.0, 0.0));
        let oc_col_enemy = spawn(&mut sim, "P53_Inf", TEAM_B, Coord3D::new(120.0, 100.0, 0.0));
        let oc_col_veh = spawn(&mut sim, "P53_VehAI", TEAM_A, Coord3D::new(120.0, 100.0, 0.0));
        let oc_col_slot = find_module_by_name(&mut sim, oc_col, "OpenContain", 0).unwrap();
        let _ = sim.dispatch_module(oc_col_slot, |m, s, c| {
            m.as_contain_mut().unwrap().add_to_contain(s, c, oc_col_enemy);
        });
        let _ = sim.dispatch_module(oc_col_slot, |m, s, c| {
            m.as_collide_mut().unwrap().on_collide(s, c, Some(oc_col_veh), None, None);
        });
        let oc_col_cnt = sim.contain_count(oc_col);
        let oc_col_veh_in = sim
            .world
            .modules
            .payload(oc_col_slot)
            .and_then(|p| p.as_contain())
            .is_some_and(|c| {
                let ctx = sim.world.modules.meta(oc_col_slot).context(oc_col_slot);
                c.is_contained(&sim, ctx, sim.world.objects.id_of(oc_col_veh).unwrap()) != FALSE
            });
        assert_eq!(oc_col_cnt, exp.get("oc_col_cnt").as_u32(), "seed {seed} oc_col_cnt");
        assert_eq!(u32::from(oc_col_veh_in), exp.get("oc_col_veh").as_u32(), "seed {seed} oc_col_veh");

        destroy_and_flush_all(&mut sim);
        sim.world.next_object_id = 1;
        sim.world.frame = frame0;

        // =====================================================================
        // 2) SlowDeathBehavior (PKT-295)
        // =====================================================================
        let sd_mode = inp.get("sd_mode").as_i32();
        let sd_tmpl = match sd_mode {
            0 => "SD_Single",
            1 => "SD_Dual_AI",
            2 => "SD_Undead",
            _ => "SD_Fling",
        };
        let sd_dmg_amt = f32::from_bits(inp.get("sd_dmg_bits").as_u32());
        let sd_burned = inp.get("sd_burned").as_u32() != 0;

        let sd_obj = spawn(&mut sim, sd_tmpl, TEAM_A, Coord3D::new(200.0, 200.0, 5.0));
        sim.world.rng.init_random_with_seed(logic_seed);
        let sd0_slot = find_module_by_name(&mut sim, sd_obj, "SlowDeathBehavior", 0).unwrap();
        let sd1_slot = find_module_by_name(&mut sim, sd_obj, "SlowDeathBehavior", 1);

        let mut sd_dmg = DamageInfo::default();
        sd_dmg.input.damage_type = DamageType::Unresistable;
        sd_dmg.input.death_type = if sd_burned { DeathType::Burned } else { DeathType::Normal };
        sd_dmg.input.amount = sd_dmg_amt;
        sd_dmg.output.actual_damage_dealt = sd_dmg_amt;
        sd_dmg.output.actual_damage_clipped = 100.0;

        let ctx0 = sim.world.modules.meta(sd0_slot).context(sd0_slot);
        let sd0_prob = sim.world.modules.payload(sd0_slot).unwrap().as_slow_death().unwrap().get_probability_modifier(&sim, ctx0, &sd_dmg);
        let sd0_app = sim.world.modules.payload(sd0_slot).unwrap().as_slow_death().unwrap().is_die_applicable(&sim, ctx0, &sd_dmg) != FALSE;
        let sd1_prob = sd1_slot.map_or(0, |s1| {
            let ctx1 = sim.world.modules.meta(s1).context(s1);
            sim.world.modules.payload(s1).unwrap().as_slow_death().unwrap().get_probability_modifier(&sim, ctx1, &sd_dmg)
        });
        assert_eq!(sd0_prob, exp.get("sd0_prob").as_i32(), "seed {seed} sd0_prob");
        assert_eq!(u32::from(sd0_app), exp.get("sd0_app").as_u32(), "seed {seed} sd0_app");
        assert_eq!(sd1_prob, exp.get("sd1_prob").as_i32(), "seed {seed} sd1_prob");

        hurt(
            &mut sim,
            sd_obj,
            if sd_mode == 2 { DamageType::Explosion } else { DamageType::Unresistable },
            if sd_burned { DeathType::Burned } else { DeathType::Normal },
            sd_dmg_amt,
        );
        assert_eq!(sim.world.rng.get_game_logic_random_seed_crc(), exp.get("sd_die_rng").as_u32(), "seed {seed} sd_die_rng");

        let (sd0_sink, sd0_mid, sd0_dest, sd0_flags) = {
            let g = sim
                .world
                .modules
                .payload(sd0_slot)
                .unwrap()
                .as_any()
                .unwrap()
                .downcast_ref::<SlowDeathBehavior>()
                .unwrap();
            (g.core.sink_frame, g.core.midpoint_frame, g.core.destruction_frame, g.core.flags)
        };
        let sd0_pri = sim.module_schedule(sd0_slot).unwrap().priority();
        let sd1_flags = sd1_slot.map_or(0, |s1| {
            sim.world
                .modules
                .payload(s1)
                .unwrap()
                .as_any()
                .unwrap()
                .downcast_ref::<SlowDeathBehavior>()
                .unwrap()
                .core
                .flags
        });
        let sd1_pri = sd1_slot.map_or(0, |s1| sim.module_schedule(s1).unwrap().priority());
        let sd0_crc = crc_module(&mut sim, sd0_slot);
        let sd0_hex = save_module_hex(&mut sim, sd0_slot);

        assert_eq!(sd0_sink, exp.get("sd0_sink").as_u32(), "seed {seed} sd0_sink");
        assert_eq!(sd0_mid, exp.get("sd0_mid").as_u32(), "seed {seed} sd0_mid");
        assert_eq!(sd0_dest, exp.get("sd0_dest").as_u32(), "seed {seed} sd0_dest");
        assert_eq!(sd0_flags, exp.get("sd0_flags").as_u32(), "seed {seed} sd0_flags");
        assert_eq!(sd0_pri, exp.get("sd0_pri").as_u32(), "seed {seed} sd0_pri");
        assert_eq!(sd1_flags, exp.get("sd1_flags").as_u32(), "seed {seed} sd1_flags");
        assert_eq!(sd1_pri, exp.get("sd1_pri").as_u32(), "seed {seed} sd1_pri");
        assert_eq!(sd0_crc, exp.get("sd0_crc").as_u32(), "seed {seed} sd0_crc");
        assert_eq!(sd0_hex, exp.get("sd0_xfer").as_str(), "seed {seed} sd0_xfer");

        let mut sd_destroyed_frame = 0u32;
        for k in 1..=25u32 {
            let f = frame0 + k;
            step_single_update(&mut sim, sd0_slot, f);
            if let Some(s1) = sd1_slot {
                step_single_update(&mut sim, s1, f);
            }
            if sim.world.objects.get(sd_obj).is_some_and(|o| o.is_destroyed()) && sd_destroyed_frame == 0 {
                sd_destroyed_frame = f;
            }
            sim.process_destroy_list();
            if sd_destroyed_frame != 0 {
                break;
            }
        }
        let sd_pos_z = if sd_destroyed_frame == 0 {
            sim.world.objects.get(sd_obj).unwrap().thing.get_position().z
        } else {
            0.0
        };
        let sd0_flags_post = if sd_destroyed_frame == 0 {
            sim.world
                .modules
                .payload(sd0_slot)
                .unwrap()
                .as_any()
                .unwrap()
                .downcast_ref::<SlowDeathBehavior>()
                .unwrap()
                .core
                .flags
        } else {
            sd0_flags
        };
        assert_eq!(sd_destroyed_frame, exp.get("sd_dest_f").as_u32(), "seed {seed} sd_dest_f");
        assert_eq!(sd_pos_z.to_bits(), exp.get("sd_pos_z_bits").as_u32(), "seed {seed} sd_pos_z");
        assert_eq!(sd0_flags_post, exp.get("sd0_flags_post").as_u32(), "seed {seed} sd0_flags_post");
        assert_eq!(sim.world.rng.get_game_logic_random_seed_crc(), exp.get("sd_step_rng").as_u32(), "seed {seed} sd_step_rng");

        if seed == 0 {
            assert_eq!(oc_pre_count, diag.get("oc_pre_count").as_u32());
            assert_eq!(oc_pre_stealth, diag.get("oc_pre_stealth").as_u32());
            assert_eq!(oc_pre_hero, diag.get("oc_pre_hero").as_u32());
            assert_eq!(oc_pre_pm as u32, diag.get("oc_pre_player_mask").as_u32());
            assert_eq!(oc_mid_pm as u32, diag.get("oc_mid_player_mask").as_u32());
            assert_eq!(u32::from(oc_mid_wanters), diag.get("oc_mid_wanters").as_u32());
            assert_eq!(oc_post_count, diag.get("oc_post_count").as_u32());
            assert_eq!(sd0_sink, diag.get("sd0_sink").as_u32());
            assert_eq!(sd0_mid, diag.get("sd0_mid").as_u32());
            assert_eq!(sd0_dest, diag.get("sd0_dest").as_u32());
            assert_eq!(sd0_flags, diag.get("sd0_flags").as_u32());
            assert_eq!(sd0_pri, diag.get("sd0_pri").as_u32());
            assert_eq!(sd_destroyed_frame, diag.get("sd_destroyed_frame").as_u32());
        }

        destroy_and_flush_all(&mut sim);
        sim.world.next_object_id = 1;
        sim.world.frame = frame0;
        sim.world.rng.init_random_with_seed(logic_seed);

        // =====================================================================
        // 3) InstantDeathBehavior (PKT-276)
        // =====================================================================
        let id_uc = inp.get("id_uc").as_u32() != 0;
        let id_crushed = inp.get("id_crushed").as_u32() != 0;
        let mut id_status = ObjectStatusMaskType::new();
        if id_uc {
            id_status.set(ObjectStatusType::UnderConstruction as Int, TRUE as Int);
        }
        let id_obj = spawn_with_status(&mut sim, "ID_Multi", TEAM_A, Coord3D::new(50.0, 60.0, 0.0), id_status);
        let id_slot = find_module_by_name(&mut sim, id_obj, "InstantDeathBehavior", 0).unwrap();
        let id_pre_hex = save_module_hex(&mut sim, id_slot);
        let id_crc = crc_module(&mut sim, id_slot);

        hurt(
            &mut sim,
            id_obj,
            DamageType::Unresistable,
            if id_crushed { DeathType::Crushed } else { DeathType::Normal },
            200.0,
        );
        let id_destroyed = sim.world.objects.get(id_obj).is_some_and(|o| o.is_destroyed());
        assert_eq!(u32::from(id_destroyed), exp.get("id_dest").as_u32(), "seed {seed} id_dest");
        assert_eq!(sim.world.rng.get_game_logic_random_seed_crc(), exp.get("id_rng").as_u32(), "seed {seed} id_rng");
        assert_eq!(id_crc, exp.get("id_crc").as_u32(), "seed {seed} id_crc");
        assert_eq!(id_pre_hex, exp.get("id_xfer").as_str(), "seed {seed} id_xfer");
        if seed == 0 {
            assert_eq!(u32::from(id_destroyed), diag.get("id_destroyed").as_u32());
        }

        destroy_and_flush_all(&mut sim);
        sim.world.next_object_id = 1;
        sim.world.frame = frame0;
        sim.world.rng.init_random_with_seed(logic_seed);

        // =====================================================================
        // 4) AutoHealBehavior (PKT-257)
        // =====================================================================
        let ah_mode = inp.get("ah_mode").as_i32();
        let ah_tmpl = match ah_mode {
            0 => "AH_Self",
            1 => "AH_RadiusCont",
            2 => "AH_RadiusBurst",
            _ => "AH_WholePlayer",
        };
        let ah_obj = spawn(&mut sim, ah_tmpl, TEAM_A, Coord3D::new(300.0, 300.0, 0.0));
        let ah_slot = find_module_by_name(&mut sim, ah_obj, "AutoHealBehavior", 0).unwrap();
        let ah_init_rng = sim.world.rng.get_game_logic_random_seed_crc();
        let ah_init_pri = sim.module_schedule(ah_slot).unwrap().priority();

        assert_eq!(ah_init_rng, exp.get("ah_rng").as_u32(), "seed {seed} ah_rng");
        assert_eq!(ah_init_pri, exp.get("ah_init_pri").as_u32(), "seed {seed} ah_init_pri");

        let ah_ally = spawn(&mut sim, "P53_Inf", TEAM_A, Coord3D::new(320.0, 300.0, 0.0));
        let ah_enemy = spawn(&mut sim, "P53_Inf", TEAM_B, Coord3D::new(320.0, 300.0, 0.0));
        let ah_forbid = spawn(
            &mut sim,
            if ah_mode == 3 { "P53_Hero" } else { "P53_Struct" },
            TEAM_A,
            Coord3D::new(320.0, 300.0, 0.0),
        );
        sim.update_partition();

        hurt(&mut sim, ah_obj, DamageType::Unresistable, DeathType::Normal, 60.0);
        hurt(&mut sim, ah_ally, DamageType::Unresistable, DeathType::Normal, 50.0);
        hurt(&mut sim, ah_enemy, DamageType::Unresistable, DeathType::Normal, 50.0);
        hurt(&mut sim, ah_forbid, DamageType::Unresistable, DeathType::Normal, 50.0);

        let ut_old_mask = sim
            .rules
            .upgrade_center
            .find_upgrade("UpgOld", &mut sim.rules.name_keys)
            .unwrap()
            .upgrade_mask()
            .clone();
        if ah_mode == 0 {
            let empty = UpgradeMaskType::new();
            sim.world
                .objects
                .get_mut(ah_obj)
                .unwrap()
                .upgrades_completed
                .clear_and_set(&empty, &ut_old_mask);
            sim.grant_upgrade(ah_obj, "UpgHeal", FALSE);
            step_single_update(&mut sim, ah_slot, frame0 + 1);
            hurt(&mut sim, ah_obj, DamageType::Unresistable, DeathType::Normal, 10.0);
        }
        let ah_old_removed = sim
            .world
            .objects
            .get(ah_obj)
            .unwrap()
            .upgrades_completed
            .test_for_any(&ut_old_mask)
            != FALSE;

        for k in 1..=10u32 {
            step_single_update(&mut sim, ah_slot, frame0 + k);
        }
        if seed % 6 == 0 {
            let _ = sim.dispatch_module(ah_slot, |m, s, c| {
                m.as_any_mut()
                    .unwrap()
                    .downcast_mut::<AutoHealBehavior>()
                    .unwrap()
                    .stop_healing(s, c);
            });
        } else if seed % 6 == 3 {
            sim.world
                .modules
                .payload_mut(ah_slot)
                .unwrap()
                .as_any_mut()
                .unwrap()
                .downcast_mut::<AutoHealBehavior>()
                .unwrap()
                .undo_upgrade();
        }
        let ah_self_hp_bits = body_health_bits(&sim, ah_obj);
        let ah_ally_hp_bits = body_health_bits(&sim, ah_ally);
        let ah_enemy_hp_bits = body_health_bits(&sim, ah_enemy);
        let ah_forbid_hp_bits = body_health_bits(&sim, ah_forbid);
        let (ah_soonest, ah_stopped) = {
            let g = sim
                .world
                .modules
                .payload(ah_slot)
                .unwrap()
                .as_any()
                .unwrap()
                .downcast_ref::<AutoHealBehavior>()
                .unwrap();
            (g.soonest_heal_frame, g.stopped != FALSE)
        };
        let ah_post_pri = sim.module_schedule(ah_slot).unwrap().priority();
        let ah_crc = crc_module(&mut sim, ah_slot);
        let ah_hex = save_module_hex(&mut sim, ah_slot);

        assert_eq!(u32::from(ah_old_removed), exp.get("ah_old_rem").as_u32(), "seed {seed} ah_old_rem");
        assert_eq!(ah_self_hp_bits, exp.get("ah_hp_bits").as_u32(), "seed {seed} ah_hp_bits");
        assert_eq!(ah_ally_hp_bits, exp.get("ah_ally_hp_bits").as_u32(), "seed {seed} ah_ally_hp_bits");
        assert_eq!(ah_enemy_hp_bits, exp.get("ah_enemy_hp_bits").as_u32(), "seed {seed} ah_enemy_hp_bits");
        assert_eq!(ah_forbid_hp_bits, exp.get("ah_forbid_hp_bits").as_u32(), "seed {seed} ah_forbid_hp_bits");
        assert_eq!(ah_soonest, exp.get("ah_soon").as_u32(), "seed {seed} ah_soon");
        assert_eq!(u32::from(ah_stopped), exp.get("ah_stop").as_u32(), "seed {seed} ah_stop");
        assert_eq!(ah_post_pri, exp.get("ah_post_pri").as_u32(), "seed {seed} ah_post_pri");
        assert_eq!(ah_crc, exp.get("ah_crc").as_u32(), "seed {seed} ah_crc");
        assert_eq!(ah_hex, exp.get("ah_xfer").as_str(), "seed {seed} ah_xfer");
        if seed == 0 {
            assert_eq!(ah_init_pri, diag.get("ah_init_pri").as_u32());
            assert_eq!(ah_self_hp_bits, diag.get("ah_self_hp_bits").as_u32());
        }

        destroy_and_flush_all(&mut sim);
        sim.world.next_object_id = 1;
        sim.world.frame = frame0;
        sim.world.rng.init_random_with_seed(logic_seed);

        // =====================================================================
        // 5) FireWeaponWhenDamagedBehavior (PKT-268)
        // =====================================================================
        let fwd_starts_active = inp.get("fwd_sa").as_u32() != 0;
        let fwd_upg = inp.get("fwd_upg").as_u32() != 0;
        let fwd_excl = inp.get("fwd_excl").as_u32() != 0;
        let fwd_dmg_amt = f32::from_bits(inp.get("fwd_dmg_bits").as_u32());

        let fwd_obj = spawn(
            &mut sim,
            if fwd_starts_active { "FWD_Active" } else { "FWD_Test" },
            TEAM_A,
            Coord3D::new(400.0, 400.0, 0.0),
        );
        let fwd_slot = find_module_by_name(&mut sim, fwd_obj, "FireWeaponWhenDamagedBehavior", 0).unwrap();
        let fwd_init_pri = sim.module_schedule(fwd_slot).unwrap().priority();
        if fwd_upg {
            sim.grant_upgrade(fwd_obj, "UpgWpn", FALSE);
        }
        for k in 1..=12u32 {
            step_single_update(&mut sim, fwd_slot, frame0 + k);
        }
        sim.world.frame = frame0 + 12;
        hurt(
            &mut sim,
            fwd_obj,
            if fwd_excl { DamageType::Explosion } else { DamageType::SmallArms },
            DeathType::Normal,
            fwd_dmg_amt,
        );
        if sim.world.objects.get(fwd_obj).is_some_and(|o| !o.is_destroyed()) {
            step_single_update(&mut sim, fwd_slot, frame0 + 13);
            sim.world.frame = frame0 + 15;
            hurt(&mut sim, fwd_obj, DamageType::SmallArms, DeathType::Normal, 40.0);
            step_single_update(&mut sim, fwd_slot, frame0 + 17);
        }
        let fwd_post_pri = sim.module_schedule(fwd_slot).unwrap().priority();
        let fwd_crc = crc_module(&mut sim, fwd_slot);
        let fwd_hex = save_module_hex(&mut sim, fwd_slot);
        assert_eq!(fwd_init_pri, exp.get("fwd_init_pri").as_u32(), "seed {seed} fwd_init_pri");
        assert_eq!(fwd_post_pri, exp.get("fwd_post_pri").as_u32(), "seed {seed} fwd_post_pri");
        assert_eq!(fwd_crc, exp.get("fwd_crc").as_u32(), "seed {seed} fwd_crc");
        assert_eq!(fwd_hex, exp.get("fwd_xfer").as_str(), "seed {seed} fwd_xfer");
        if seed == 0 {
            assert_eq!(fwd_init_pri, diag.get("fwd_init_pri").as_u32());
            assert_eq!(fwd_post_pri, diag.get("fwd_post_pri").as_u32());
        }

        destroy_and_flush_all(&mut sim);
        sim.world.next_object_id = 1;
        sim.world.frame = frame0;
        sim.world.rng.init_random_with_seed(logic_seed);

        // =====================================================================
        // 6) FireWeaponWhenDeadBehavior (PKT-269)
        // =====================================================================
        let f2_give_upg = inp.get("f2_upg").as_u32() != 0;
        let f2_uc = inp.get("f2_uc").as_u32() != 0;
        let f2_oc = inp.get("f2_oc").as_u32() != 0;
        let f2_pc = inp.get("f2_pc").as_u32() != 0;
        let f2_sui = inp.get("f2_sui").as_u32() != 0;

        let mut f2_status = ObjectStatusMaskType::new();
        if f2_uc {
            f2_status.set(ObjectStatusType::UnderConstruction as Int, TRUE as Int);
        }
        let f2_obj = spawn_with_status(&mut sim, "FWDead_Test", TEAM_A, Coord3D::new(450.0, 450.0, 0.0), f2_status);
        let f2_id = sim.world.objects.id_of(f2_obj).unwrap();
        let f2_slot = find_module_by_name(&mut sim, f2_obj, "FireWeaponWhenDeadBehavior", 0).unwrap();
        if f2_give_upg {
            sim.grant_upgrade(f2_obj, "UpgWpn", FALSE);
            let wpn_mask = sim
                .rules
                .upgrade_center
                .find_upgrade("UpgWpn", &mut sim.rules.name_keys)
                .unwrap()
                .upgrade_mask()
                .clone();
            sim.dispatch_attempt_upgrade(f2_obj, &wpn_mask);
        }
        if f2_oc {
            let mask = sim
                .rules
                .upgrade_center
                .find_upgrade("UpgObjConflict", &mut sim.rules.name_keys)
                .unwrap()
                .upgrade_mask()
                .clone();
            let empty = UpgradeMaskType::new();
            sim.world
                .objects
                .get_mut(f2_obj)
                .unwrap()
                .upgrades_completed
                .clear_and_set(&empty, &mask);
        }
        if f2_pc {
            let mask = sim
                .rules
                .upgrade_center
                .find_upgrade("UpgConflict", &mut sim.rules.name_keys)
                .unwrap()
                .upgrade_mask()
                .clone();
            let empty = UpgradeMaskType::new();
            let player_a = sim.player_index(TEAM_A);
            sim.world
                .player_upgrades
                .entry(player_a)
                .or_default()
                .clear_and_set(&empty, &mask);
        }
        let f2_crc = crc_module(&mut sim, f2_slot);
        let f2_hex = save_module_hex(&mut sim, f2_slot);
        let _f2_target = spawn(&mut sim, "P53_Inf", TEAM_B, Coord3D::new(460.0, 450.0, 0.0));
        sim.update_partition();
        hurt(
            &mut sim,
            f2_obj,
            DamageType::Unresistable,
            if f2_sui { DeathType::Suicided } else { DeathType::Normal },
            500.0,
        );
        let f2_fired = sim
            .world
            .weapon_fires
            .iter()
            .any(|(oid, wname, ..)| *oid == f2_id && wname == "Wpn_Death");
        assert_eq!(u32::from(f2_fired), exp.get("f2_fired").as_u32(), "seed {seed} f2_fired");
        assert_eq!(f2_crc, exp.get("f2_crc").as_u32(), "seed {seed} f2_crc");
        assert_eq!(f2_hex, exp.get("f2_xfer").as_str(), "seed {seed} f2_xfer");

        destroy_and_flush_all(&mut sim);
        sim.world.next_object_id = 1;
        sim.world.frame = frame0;
        sim.world.rng.init_random_with_seed(logic_seed);

        // =====================================================================
        // 7) GrantStealthBehavior (PKT-273)
        // =====================================================================
        let gs_obj = spawn(&mut sim, "GS_Emitter", TEAM_A, Coord3D::new(10.0, 10.0, 0.0));
        let gs_near = spawn(&mut sim, "P53_Grantable", TEAM_A, Coord3D::new(30.0, 10.0, 0.0));
        let gs_mid = spawn(&mut sim, "P53_Grantable", TEAM_A, Coord3D::new(65.0, 10.0, 0.0));
        let gs_nost = spawn(&mut sim, "P53_Inf", TEAM_A, Coord3D::new(25.0, 10.0, 0.0));
        let gs_enemy = spawn(&mut sim, "P53_Grantable", TEAM_B, Coord3D::new(20.0, 10.0, 0.0));
        let gs_offmap = spawn(&mut sim, "P53_Grantable", TEAM_A, Coord3D::new(-5.0, -5.0, 0.0));
        sim.world.objects.get_mut(gs_offmap).unwrap().private_status |= PRIVATE_STATUS_OFF_MAP;
        sim.update_partition();

        let gs_slot = find_module_by_name(&mut sim, gs_obj, "GrantStealthBehavior", 0).unwrap();
        let gs_init_pri = sim.module_schedule(gs_slot).unwrap().priority();
        assert_eq!(gs_init_pri, exp.get("gs_init_pri").as_u32(), "seed {seed} gs_init_pri");

        let has_can_stealth = |sim: &SimContext, slot: ObjectSlot| -> u32 {
            u32::from(
                sim.world
                    .objects
                    .get(slot)
                    .is_some_and(|o| o.status.test(ObjectStatusType::CanStealth as Int) != FALSE),
            )
        };

        step_single_update(&mut sim, gs_slot, frame0 + 1);
        let gs_s1 = [
            has_can_stealth(&sim, gs_near),
            has_can_stealth(&sim, gs_mid),
            has_can_stealth(&sim, gs_nost),
            has_can_stealth(&sim, gs_enemy),
            has_can_stealth(&sim, gs_offmap),
        ];
        let gs_s1_rad_bits = sim
            .world
            .modules
            .payload(gs_slot)
            .unwrap()
            .as_any()
            .unwrap()
            .downcast_ref::<GrantStealthBehavior>()
            .unwrap()
            .current_scan_radius
            .to_bits();
        let gs_crc = crc_module(&mut sim, gs_slot);
        let gs_hex = save_module_hex(&mut sim, gs_slot);

        let exp_s1: Vec<u32> = exp.get("gs_s1").as_arr().iter().map(|x| x.as_u32()).collect();
        assert_eq!(gs_s1.to_vec(), exp_s1, "seed {seed} gs_s1");
        assert_eq!(gs_s1_rad_bits, exp.get("gs_s1_rad_bits").as_u32(), "seed {seed} gs_s1_rad");
        assert_eq!(gs_crc, exp.get("gs_crc").as_u32(), "seed {seed} gs_crc");
        assert_eq!(gs_hex, exp.get("gs_xfer").as_str(), "seed {seed} gs_xfer");

        step_single_update(&mut sim, gs_slot, frame0 + 2);
        let gs_s2 = [
            has_can_stealth(&sim, gs_mid),
            u32::from(sim.world.objects.get(gs_obj).is_some_and(|o| o.is_destroyed())),
        ];
        let exp_s2: Vec<u32> = exp.get("gs_s2").as_arr().iter().map(|x| x.as_u32()).collect();
        assert_eq!(gs_s2.to_vec(), exp_s2, "seed {seed} gs_s2");

        step_single_update(&mut sim, gs_slot, frame0 + 3);
        let gs_s3_dest = u32::from(sim.world.objects.get(gs_obj).is_some_and(|o| o.is_destroyed()));
        let gs_s3_rad_bits = sim
            .world
            .modules
            .payload(gs_slot)
            .unwrap()
            .as_any()
            .unwrap()
            .downcast_ref::<GrantStealthBehavior>()
            .unwrap()
            .current_scan_radius
            .to_bits();
        assert_eq!(gs_s3_dest, exp.get("gs_s3_dest").as_u32(), "seed {seed} gs_s3_dest");
        assert_eq!(gs_s3_rad_bits, exp.get("gs_s3_rad_bits").as_u32(), "seed {seed} gs_s3_rad");

        if seed == 0 {
            assert_eq!(gs_s1[0], diag.get("gs_s1_near").as_u32());
            assert_eq!(gs_s1[1], diag.get("gs_s1_mid").as_u32());
            assert_eq!(gs_s2[0], diag.get("gs_s2_mid").as_u32());
            assert_eq!(gs_s3_dest, diag.get("gs_s3_dest").as_u32());
        }
    }
}

#[test]
fn wp53_a_contracts_registration_collide_and_reentrancy() {
    let rules = build_rules();
    let mut sim = new_sim(rules);
    sim.world.frame = 50;
    sim.world.rng.init_random_with_seed(0x123456);

    // 1) OpenContain::on_collide ejecting enemy & adding entering ally
    let oc = spawn(&mut sim, "OC_T0", TEAM_A, Coord3D::new(100.0, 100.0, 0.0));
    let enemy = spawn(&mut sim, "P53_Inf", TEAM_B, Coord3D::new(100.0, 100.0, 0.0));
    let veh = spawn(&mut sim, "P53_VehAI", TEAM_A, Coord3D::new(100.0, 100.0, 0.0));

    let oc_slot = find_module_by_name(&mut sim, oc, "OpenContain", 0).unwrap();
    let ai_slot = find_module_by_name(&mut sim, veh, "AIUpdateInterface", 0).unwrap();
    sim.world
        .modules
        .payload_mut(ai_slot)
        .unwrap()
        .as_any_mut()
        .unwrap()
        .downcast_mut::<TestAIUpdate>()
        .unwrap()
        .enter_target = Some(oc);

    let _ = sim.dispatch_module(oc_slot, |m, s, c| {
        m.as_contain_mut().unwrap().add_to_contain(s, c, enemy);
    });
    assert_eq!(sim.contain_count(oc), 1);

    let _ = sim.dispatch_module(oc_slot, |m, s, c| {
        m.as_collide_mut().unwrap().on_collide(s, c, Some(veh), None, None);
    });
    assert_eq!(sim.contain_count(oc), 1);
    let veh_id = sim.world.objects.id_of(veh).unwrap();
    let enemy_id = sim.world.objects.id_of(enemy).unwrap();
    let oc_contains = sim
        .world
        .modules
        .payload(oc_slot)
        .and_then(|p| p.as_contain())
        .map(|c| {
            let ctx = sim.world.modules.meta(oc_slot).context(oc_slot);
            c.get_contained_items(&sim, ctx).to_vec()
        })
        .unwrap();
    assert_eq!(oc_contains, vec![veh_id]);
    assert!(sim.world.objects.get(enemy).unwrap().contained_by.is_none());
    assert_ne!(veh_id, enemy_id);

    // 2) Deferred containment ops re-entrancy (`ContainAdd` / `ContainRemove` / `ContainRemoveAll`)
    sim.contain_add(oc, enemy); // invalid for OC_T0? Wait: direct contain_add bypasses is_valid_container_for
    assert_eq!(sim.contain_count(oc), 2);
    sim.contain_remove(oc, veh, FALSE);
    assert_eq!(sim.contain_count(oc), 1);
    sim.contain_remove_all(oc, TRUE);
    assert_eq!(sim.contain_count(oc), 0);

    // 3) UndeadBody + SlowDeathBehavior second life + final slow death contract
    // (`SD_Undead` uses `SecondLifeArmor = 50%` so 250.0 Explosion => 125.0 >= 100.0 triggers second life)
    let undead = spawn(&mut sim, "SD_Undead", TEAM_A, Coord3D::new(200.0, 200.0, 0.0));
    let sd0 = find_module_by_name(&mut sim, undead, "SlowDeathBehavior", 0).unwrap();
    let sd1 = find_module_by_name(&mut sim, undead, "SlowDeathBehavior", 1).unwrap();
    hurt(&mut sim, undead, DamageType::Explosion, DeathType::Normal, 250.0);
    // First lethal hit enters second life (60 HP) and starts 1 SlowDeathBehavior (`UndeadBody.cpp:114-141`)
    assert!((body_health(&sim, undead) - 60.0).abs() < 1e-4);
    let active_count = [sd0, sd1]
        .iter()
        .filter(|&&s| {
            sim.world.modules.payload(s).unwrap().as_slow_death().unwrap().is_slow_death_in_progress() != FALSE
        })
        .count();
    assert_eq!(active_count, 1);

    // Second fatal damage completes kill cleanly without panicking or double-activating SlowDeath
    hurt(&mut sim, undead, DamageType::Explosion, DeathType::Normal, 200.0);
    assert!((body_health(&sim, undead) - 0.0).abs() < 1e-4);
    let active_post = [sd0, sd1]
        .iter()
        .filter(|&&s| {
            sim.world.modules.payload(s).unwrap().as_slow_death().unwrap().is_slow_death_in_progress() != FALSE
        })
        .count();
    assert_eq!(active_post, 1);

    // 3b) True neutral `AllowNeutralInside` check (`OC_T0` forbids neutral, `OC_T1` allows)
    let team_true_neutral = TeamId(3);
    sim.world.team_players.insert(team_true_neutral, 0);
    let oc_t1 = spawn(&mut sim, "OC_T1", TEAM_A, Coord3D::new(100.0, 100.0, 0.0));
    let true_neu = spawn(&mut sim, "P53_VehAI", team_true_neutral, Coord3D::new(100.0, 100.0, 0.0));
    assert_eq!(sim.is_valid_container_for(oc, true_neu, TRUE), FALSE);
    assert_eq!(sim.is_valid_container_for(oc_t1, true_neu, TRUE), TRUE);

    // 4) Direct module smoke check on non-spy registration
    let mut nk = NameKeyGenerator::new();
    let mut reg = zh_logic::modules::module_factory::ModuleRegistry::new();
    register_behavior_modules(&mut reg, &mut nk);
    register_contain_modules(&mut reg, &mut nk);
    assert!(reg.find_descriptor("OpenContain", ModuleType::Behavior).is_some());
    assert!(reg.find_descriptor("SlowDeathBehavior", ModuleType::Behavior).is_some());
    assert!(reg.find_descriptor("InstantDeathBehavior", ModuleType::Behavior).is_some());
    assert!(reg.find_descriptor("AutoHealBehavior", ModuleType::Behavior).is_some());
    assert!(reg.find_descriptor("FireWeaponWhenDamagedBehavior", ModuleType::Behavior).is_some());
    assert!(reg.find_descriptor("FireWeaponWhenDeadBehavior", ModuleType::Behavior).is_some());
    assert!(reg.find_descriptor("GrantStealthBehavior", ModuleType::Behavior).is_some());
}
