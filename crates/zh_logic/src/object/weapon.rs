//! `PKT-399`: Weapon templates, runtime weapons, reload/clip math, and snapshots.

use crate::gamelogic::global_data::{
    WeaponBonus, WeaponBonusField, WeaponBonusSet,
};
use crate::rules::combat_types::{
    DamageType, DeathType, WeaponStatus,
};
use zh_common::ini::{convert_duration_from_msecs_to_frames, IniError, IniReader};
use zh_common::math::{gm_ceilf, real_to_int_floor, Coord2D};
use zh_common::random_value::PrngState;
use zh_common::types::{
    Bool, Int, ObjectId, Real, UnsignedInt, UnsignedShort, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

use super::weapon_set::WeaponSlotType;

pub const NO_MAX_SHOTS_LIMIT: Int = 0x7fff_ffff;
pub const PATHFIND_CELL_SIZE_F: Real = 10.0_f32;
pub const WEAPON_UNDERSIZE: Real = 2.5_f32;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeaponReloadType {
    #[default]
    AutoReload = 0,
    NoReload = 1,
    ReturnToBase = 2,
}

impl WeaponReloadType {
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v { 0 => Some(Self::AutoReload), 1 => Some(Self::NoReload), 2 => Some(Self::ReturnToBase), _ => None }
    }
    pub const fn as_i32(self) -> Int { self as Int }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeaponPrefireType {
    #[default]
    PerShot = 0,
    PerAttack = 1,
    PerClip = 2,
}

impl WeaponPrefireType {
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v { 0 => Some(Self::PerShot), 1 => Some(Self::PerAttack), 2 => Some(Self::PerClip), _ => None }
    }
    pub const fn as_i32(self) -> Int { self as Int }
}

pub const WEAPON_RELOAD_NAMES: [&str; 3] = ["AUTO_RELOAD", "NO_RELOAD", "RETURN_TO_BASE"];
pub const WEAPON_PREFIRE_NAMES: [&str; 3] = ["PER_SHOT", "PER_ATTACK", "PER_CLIP"];

pub const WEAPON_ANTI_AIRBORNE_VEHICLE: Int = 0x01;
pub const WEAPON_ANTI_GROUND: Int = 0x02;
pub const WEAPON_ANTI_PROJECTILE: Int = 0x04;
pub const WEAPON_ANTI_SMALL_MISSILE: Int = 0x08;
pub const WEAPON_ANTI_MINE: Int = 0x10;
pub const WEAPON_ANTI_AIRBORNE_INFANTRY: Int = 0x20;
pub const WEAPON_ANTI_BALLISTIC_MISSILE: Int = 0x40;
pub const WEAPON_ANTI_PARACHUTE: Int = 0x80;

pub const WEAPON_AFFECTS_SELF: Int = 0x01;
pub const WEAPON_AFFECTS_ALLIES: Int = 0x02;
pub const WEAPON_AFFECTS_ENEMIES: Int = 0x04;
pub const WEAPON_AFFECTS_NEUTRALS: Int = 0x08;
pub const WEAPON_AFFECTS_SUICIDE: Int = 0x10;
pub const WEAPON_AFFECTS_NOT_SIMILAR: Int = 0x20;
pub const WEAPON_AFFECTS_NOT_AIRBORNE: Int = 0x40;

pub const WEAPON_COLLIDE_ALLIES: Int = 0x0001;
pub const WEAPON_COLLIDE_ENEMIES: Int = 0x0002;
pub const WEAPON_COLLIDE_STRUCTURES: Int = 0x0004;
pub const WEAPON_COLLIDE_SHRUBBERY: Int = 0x0008;
pub const WEAPON_COLLIDE_PROJECTILES: Int = 0x0010;
pub const WEAPON_COLLIDE_WALLS: Int = 0x0020;
pub const WEAPON_COLLIDE_SMALL_MISSILES: Int = 0x0040;
pub const WEAPON_COLLIDE_BALLISTIC_MISSILES: Int = 0x0080;
pub const WEAPON_COLLIDE_CONTROLLED_STRUCTURES: Int = 0x0100;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistoricWeaponDamageInfo {
    pub frame: UnsignedInt,
    pub location: Coord2D,
    pub trigger_id: UnsignedInt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeaponTemplate {
    pub name: String,
    pub primary_damage: Real,
    pub primary_damage_radius: Real,
    pub secondary_damage: Real,
    pub secondary_damage_radius: Real,
    pub shock_wave_amount: Real,
    pub shock_wave_radius: Real,
    pub shock_wave_taper_off: Real,
    pub attack_range: Real,
    pub minimum_attack_range: Real,
    pub request_assist_range: Real,
    pub aim_delta: Real,
    pub scatter_radius: Real,
    pub scatter_target_scalar: Real,
    pub scatter_radius_vs_infantry: Real,
    pub scatter_targets: Vec<Coord2D>,
    pub damage_type: DamageType,
    pub death_type: DeathType,
    pub weapon_speed: Real,
    pub min_weapon_speed: Real,
    pub scale_weapon_speed: Bool,
    pub weapon_recoil: Real,
    pub min_target_pitch: Real,
    pub max_target_pitch: Real,
    pub radius_damage_angle: Real,
    pub clip_size: Int,
    pub clip_reload_time: Int,
    pub min_delay_between_shots: Int,
    pub max_delay_between_shots: Int,
    pub continuous_fire_one_shots_needed: Int,
    pub continuous_fire_two_shots_needed: Int,
    pub continuous_fire_coast_frames: UnsignedInt,
    pub auto_reload_when_idle_frames: UnsignedInt,
    pub shots_per_barrel: Int,
    pub damage_dealt_at_self_position: Bool,
    pub reload_type: WeaponReloadType,
    pub prefire_type: WeaponPrefireType,
    pub pre_attack_delay: Int,
    pub continue_attack_range: Real,
    pub affects_mask: Int,
    pub collide_mask: Int,
    pub anti_mask: Int,
    pub leech_range_weapon: Bool,
    pub capable_of_following_waypoint: Bool,
    pub is_shows_ammo_pips: Bool,
    pub allow_attack_garrisoned_buildings: Bool,
    pub play_fx_when_stealthed: Bool,
    pub missile_calls_on_die: Bool,
    pub suspend_fx_delay: UnsignedInt,
    pub extra_bonus: WeaponBonusSet,
}

impl Default for WeaponTemplate {
    fn default() -> Self {
        Self {
            name: "NoNameWeapon".to_string(),
            primary_damage: 0.0, primary_damage_radius: 0.0,
            secondary_damage: 0.0, secondary_damage_radius: 0.0,
            shock_wave_amount: 0.0, shock_wave_radius: 0.0, shock_wave_taper_off: 0.0,
            attack_range: 0.0, minimum_attack_range: 0.0, request_assist_range: 0.0,
            aim_delta: 0.0, scatter_radius: 0.0, scatter_target_scalar: 0.0,
            scatter_radius_vs_infantry: 0.0, scatter_targets: Vec::new(),
            damage_type: DamageType::Explosion, death_type: DeathType::Normal,
            weapon_speed: 999_999.0, min_weapon_speed: 999_999.0, scale_weapon_speed: FALSE,
            weapon_recoil: 0.0, min_target_pitch: -std::f32::consts::PI,
            max_target_pitch: std::f32::consts::PI, radius_damage_angle: std::f32::consts::PI,
            clip_size: 0, clip_reload_time: 0, min_delay_between_shots: 0,
            max_delay_between_shots: 0, continuous_fire_one_shots_needed: Int::MAX,
            continuous_fire_two_shots_needed: Int::MAX, continuous_fire_coast_frames: 0,
            auto_reload_when_idle_frames: 0, shots_per_barrel: 1,
            damage_dealt_at_self_position: FALSE, reload_type: WeaponReloadType::AutoReload,
            prefire_type: WeaponPrefireType::PerShot, pre_attack_delay: 0,
            continue_attack_range: 0.0, affects_mask: WEAPON_AFFECTS_ALLIES | WEAPON_AFFECTS_ENEMIES | WEAPON_AFFECTS_NEUTRALS,
            collide_mask: WEAPON_COLLIDE_STRUCTURES, anti_mask: WEAPON_ANTI_GROUND,
            leech_range_weapon: FALSE, capable_of_following_waypoint: FALSE,
            is_shows_ammo_pips: FALSE, allow_attack_garrisoned_buildings: FALSE,
            play_fx_when_stealthed: FALSE, missile_calls_on_die: FALSE,
            suspend_fx_delay: 0, extra_bonus: WeaponBonusSet::new(),
        }
    }
}

impl WeaponTemplate {
    pub fn new() -> Self { Self::default() }

    pub fn parse_ini_block(&mut self, text: &str) -> Result<(), IniError> {
        let mut lines = text.lines();
        let first = lines.next().unwrap_or("");
        let mut parts = first.split_whitespace();
        if parts.next().unwrap_or("") != "Weapon" { return Err(IniError::InvalidData); }
        self.name = parts.next().unwrap_or("").to_string();
        for line in lines {
            let line = line.trim();
            if line.eq_ignore_ascii_case("End") { break; }
            let Some((key, value)) = line.split_once('=') else { continue };
            let key = key.trim();
            let value = value.trim();
            match key {
                "PrimaryDamage" => self.primary_damage = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "PrimaryDamageRadius" => self.primary_damage_radius = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "SecondaryDamage" => self.secondary_damage = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "SecondaryDamageRadius" => self.secondary_damage_radius = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ShockWaveAmount" => self.shock_wave_amount = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ShockWaveRadius" => self.shock_wave_radius = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ShockWaveTaperOff" => self.shock_wave_taper_off = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "AttackRange" => self.attack_range = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "MinimumAttackRange" => self.minimum_attack_range = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "RequestAssistRange" => self.request_assist_range = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "AcceptableAimDelta" => self.aim_delta = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)? * zh_common::ini::RADS_PER_DEGREE,
                "ScatterRadius" => self.scatter_radius = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ScatterTargetScalar" => self.scatter_target_scalar = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ScatterRadiusVsInfantry" => self.scatter_radius_vs_infantry = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "DamageType" => {
                    let i = IniReader::scan_index_list(value, &zh_common::ini::DAMAGE_TYPE_NAMES).map_err(|_| IniError::InvalidData)?;
                    self.damage_type = DamageType::from_i32(i).unwrap_or(DamageType::Explosion);
                }
                "DeathType" => {
                    let i = IniReader::scan_index_list(value, &zh_common::ini::DEATH_TYPE_NAMES).map_err(|_| IniError::InvalidData)?;
                    self.death_type = DeathType::from_i32(i).unwrap_or(DeathType::Normal);
                }
                "WeaponSpeed" => self.weapon_speed = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "MinWeaponSpeed" => self.min_weapon_speed = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ScaleWeaponSpeed" => self.scale_weapon_speed = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "WeaponRecoil" => self.weapon_recoil = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)? * zh_common::ini::RADS_PER_DEGREE,
                "MinTargetPitch" => self.min_target_pitch = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)? * zh_common::ini::RADS_PER_DEGREE,
                "MaxTargetPitch" => self.max_target_pitch = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)? * zh_common::ini::RADS_PER_DEGREE,
                "RadiusDamageAngle" => self.radius_damage_angle = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ClipSize" => self.clip_size = IniReader::scan_int(value).map_err(|_| IniError::InvalidData)?,
                "ClipReloadTime" => self.clip_reload_time = gm_ceilf(convert_duration_from_msecs_to_frames(IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?)) as Int,
                "DelayBetweenShots" => self.parse_shot_delay(value)?,
                "ContinuousFireOne" => self.continuous_fire_one_shots_needed = IniReader::scan_int(value).map_err(|_| IniError::InvalidData)?,
                "ContinuousFireTwo" => self.continuous_fire_two_shots_needed = IniReader::scan_int(value).map_err(|_| IniError::InvalidData)?,
                "ContinuousFireCoast" => self.continuous_fire_coast_frames = gm_ceilf(convert_duration_from_msecs_to_frames(IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?)) as UnsignedInt,
                "AutoReloadWhenIdle" => self.auto_reload_when_idle_frames = gm_ceilf(convert_duration_from_msecs_to_frames(IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?)) as UnsignedInt,
                "ShotsPerBarrel" => self.shots_per_barrel = IniReader::scan_int(value).map_err(|_| IniError::InvalidData)?,
                "DamageDealtAtSelfPosition" => self.damage_dealt_at_self_position = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "AutoReloadsClip" => self.reload_type = if IniReader::scan_bool(value).unwrap_or(FALSE) == TRUE { WeaponReloadType::AutoReload } else { match value.to_ascii_uppercase().as_str() { "RETURN_TO_BASE" => WeaponReloadType::ReturnToBase, _ => WeaponReloadType::NoReload } },
                "PreAttackDelay" => self.pre_attack_delay = gm_ceilf(convert_duration_from_msecs_to_frames(IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?)) as Int,
                "PreAttackType" => self.prefire_type = match IniReader::scan_index_list(value, &WEAPON_PREFIRE_NAMES).unwrap_or(0) { 1 => WeaponPrefireType::PerAttack, 2 => WeaponPrefireType::PerClip, _ => WeaponPrefireType::PerShot },
                "ContinueAttackRange" => self.continue_attack_range = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "SuspendFXDelay" => self.suspend_fx_delay = gm_ceilf(convert_duration_from_msecs_to_frames(IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?)) as UnsignedInt,
                "LeechRangeWeapon" => self.leech_range_weapon = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "CapableOfFollowingWaypoints" => self.capable_of_following_waypoint = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "ShowsAmmoPips" => self.is_shows_ammo_pips = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "AllowAttackGarrisonedBldgs" => self.allow_attack_garrisoned_buildings = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "PlayFXWhenStealthed" => self.play_fx_when_stealthed = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "MissileCallsOnDie" => self.missile_calls_on_die = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "RadiusDamageAffects" => self.affects_mask = parse_affects(value),
                "ProjectileCollidesWith" => self.collide_mask = parse_collide(value),
                "AntiAirborneVehicle" => if IniReader::scan_bool(value).unwrap_or(FALSE) == TRUE { self.anti_mask |= WEAPON_ANTI_AIRBORNE_VEHICLE },
                "AntiGround" => if IniReader::scan_bool(value).unwrap_or(FALSE) == TRUE { self.anti_mask |= WEAPON_ANTI_GROUND },
                "AntiProjectile" => if IniReader::scan_bool(value).unwrap_or(FALSE) == TRUE { self.anti_mask |= WEAPON_ANTI_PROJECTILE },
                "AntiSmallMissile" => if IniReader::scan_bool(value).unwrap_or(FALSE) == TRUE { self.anti_mask |= WEAPON_ANTI_SMALL_MISSILE },
                "AntiMine" => if IniReader::scan_bool(value).unwrap_or(FALSE) == TRUE { self.anti_mask |= WEAPON_ANTI_MINE },
                "AntiAirborneInfantry" => if IniReader::scan_bool(value).unwrap_or(FALSE) == TRUE { self.anti_mask |= WEAPON_ANTI_AIRBORNE_INFANTRY },
                "AntiBallisticMissile" => if IniReader::scan_bool(value).unwrap_or(FALSE) == TRUE { self.anti_mask |= WEAPON_ANTI_BALLISTIC_MISSILE },
                "AntiParachute" => if IniReader::scan_bool(value).unwrap_or(FALSE) == TRUE { self.anti_mask |= WEAPON_ANTI_PARACHUTE },
                "ScatterTarget" => {
                    let mut x = 0.0_f32; let mut y = 0.0_f32;
                    for token in value.split_whitespace() {
                        if let Some(v) = token.strip_prefix("X:") { x = IniReader::scan_real(v).map_err(|_| IniError::InvalidData)?; }
                        if let Some(v) = token.strip_prefix("Y:") { y = IniReader::scan_real(v).map_err(|_| IniError::InvalidData)?; }
                    }
                    self.scatter_targets.push(Coord2D { x, y });
                }
                "WeaponBonus" => {
                    let mut parts = value.split_whitespace();
                    let condition_name = parts.next().unwrap_or("");
                    let field_name = parts.next().unwrap_or("");
                    let percent = parts.next().unwrap_or("100%");
                    let ci = crate::gamelogic::global_data::THE_WEAPON_BONUS_NAMES.iter().position(|n| *n == condition_name);
                    let fi = crate::gamelogic::global_data::THE_WEAPON_BONUS_FIELD_NAMES.iter().position(|n| *n == field_name);
                    if let (Some(ci), Some(fi)) = (ci, fi) {
                        let pct = IniReader::scan_percent_to_real(percent).map_err(|_| IniError::InvalidData)? * 100.0_f32;
                        self.extra_bonus.entries[ci].set_field(crate::gamelogic::global_data::WeaponBonusField::from_i32(fi as i32).unwrap(), pct / 100.0_f32);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn parse_shot_delay(&mut self, value: &str) -> Result<(), IniError> {
        if let Some(rest) = value.strip_prefix("Min:") {
            let mut parts = rest.split_whitespace();
            let min_ms = IniReader::scan_real(parts.next().unwrap_or("0")).map_err(|_| IniError::InvalidData)?;
            let max_text = parts.find(|p| p.starts_with("Max:")).unwrap_or("Max:0");
            let max_ms = IniReader::scan_real(max_text.trim_start_matches("Max:")).map_err(|_| IniError::InvalidData)?;
            self.min_delay_between_shots = gm_ceilf(convert_duration_from_msecs_to_frames(min_ms)) as Int;
            self.max_delay_between_shots = gm_ceilf(convert_duration_from_msecs_to_frames(max_ms)) as Int;
        } else {
            let ms = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?;
            let f = gm_ceilf(convert_duration_from_msecs_to_frames(ms)) as Int;
            self.min_delay_between_shots = f;
            self.max_delay_between_shots = f;
        }
        Ok(())
    }

    pub fn get_attack_range(&self, bonus: &WeaponBonus) -> Real {
        let mut r = self.attack_range * bonus.get_field(WeaponBonusField::Range) - WEAPON_UNDERSIZE;
        if r < 0.0 { r = 0.0; }
        r
    }
    pub fn get_unmodified_attack_range(&self) -> Real { self.attack_range }
    pub fn get_minimum_attack_range(&self) -> Real {
        let mut r = self.minimum_attack_range - WEAPON_UNDERSIZE;
        if r < 0.0 { r = 0.0; }
        r
    }
    pub fn get_primary_damage(&self, bonus: &WeaponBonus) -> Real { self.primary_damage * bonus.get_field(WeaponBonusField::Damage) }
    pub fn get_primary_damage_radius(&self, bonus: &WeaponBonus) -> Real { self.primary_damage_radius * bonus.get_field(WeaponBonusField::Radius) }
    pub fn get_secondary_damage(&self, bonus: &WeaponBonus) -> Real { self.secondary_damage * bonus.get_field(WeaponBonusField::Damage) }
    pub fn get_secondary_damage_radius(&self, bonus: &WeaponBonus) -> Real { self.secondary_damage_radius * bonus.get_field(WeaponBonusField::Radius) }
    pub fn get_delay_between_shots(&self, bonus: &WeaponBonus, rng: &mut PrngState) -> Int {
        let delay = if self.min_delay_between_shots >= self.max_delay_between_shots {
            self.max_delay_between_shots
        } else {
            rng.get_game_logic_random_value(self.min_delay_between_shots, self.max_delay_between_shots)
        };
        real_to_int_floor((delay as Real) / bonus.get_field(WeaponBonusField::RateOfFire))
    }
    pub fn get_clip_reload_time(&self, bonus: &WeaponBonus) -> Int {
        real_to_int_floor((self.clip_reload_time as Real) / bonus.get_field(WeaponBonusField::RateOfFire))
    }
    pub fn get_pre_attack_delay(&self, bonus: &WeaponBonus) -> Int {
        ((self.pre_attack_delay as Real) * bonus.get_field(WeaponBonusField::PreAttack)) as Int
    }
    pub fn is_contact_weapon(&self) -> Bool { if (self.attack_range - WEAPON_UNDERSIZE) < PATHFIND_CELL_SIZE_F { TRUE } else { FALSE } }
    pub fn is_damage_weapon(&self) -> Bool {
        match self.damage_type {
            DamageType::Deploy | DamageType::Disarm => TRUE,
            DamageType::Hack => FALSE,
            _ => if self.primary_damage > 0.0_f32 || self.secondary_damage > 0.0_f32 { TRUE } else { FALSE },
        }
    }
}

fn parse_affects(value: &str) -> Int {
    let mut m = 0;
    for name in value.split_whitespace() {
        m |= match name { "SELF"=>WEAPON_AFFECTS_SELF, "ALLIES"=>WEAPON_AFFECTS_ALLIES, "ENEMIES"=>WEAPON_AFFECTS_ENEMIES, "NEUTRAL"=>WEAPON_AFFECTS_NEUTRALS, "NEUTRALs"=>WEAPON_AFFECTS_NEUTRALS, "NEUTRALS"=>WEAPON_AFFECTS_NEUTRALS, "SUICIDE"=>WEAPON_AFFECTS_SUICIDE, "NOT_SIMILAR"=>WEAPON_AFFECTS_NOT_SIMILAR, "NOT_AIRBORNE"=>WEAPON_AFFECTS_NOT_AIRBORNE, _=>0 };
    }
    m
}
fn parse_collide(value: &str) -> Int {
    let mut m = 0;
    for name in value.split_whitespace() {
        m |= match name { "ALLIES"=>WEAPON_COLLIDE_ALLIES, "ENEMIES"=>WEAPON_COLLIDE_ENEMIES, "STRUCTURES"=>WEAPON_COLLIDE_STRUCTURES, "SHRUBBERY"=>WEAPON_COLLIDE_SHRUBBERY, "PROJECTILES"=>WEAPON_COLLIDE_PROJECTILES, "WALLS"=>WEAPON_COLLIDE_WALLS, "SMALL_MISSILES"=>WEAPON_COLLIDE_SMALL_MISSILES, "BALLISTIC_MISSILES"=>WEAPON_COLLIDE_BALLISTIC_MISSILES, "CONTROLLED_STRUCTURES"=>WEAPON_COLLIDE_CONTROLLED_STRUCTURES, _=>0 };
    }
    m
}

#[derive(Debug, Clone, PartialEq)]
pub struct Weapon {
    pub template: WeaponTemplate,
    pub template_name: String,
    pub wslot: WeaponSlotType,
    pub status: WeaponStatus,
    pub ammo_in_clip: UnsignedInt,
    pub when_we_can_fire_again: UnsignedInt,
    pub when_pre_attack_finished: UnsignedInt,
    pub when_last_reload_started: UnsignedInt,
    pub last_fire_frame: UnsignedInt,
    pub suspend_fx_frame: UnsignedInt,
    pub projectile_stream_id: ObjectId,
    pub max_shot_count: Int,
    pub cur_barrel: Int,
    pub num_shots_for_cur_barrel: Int,
    pub scatter_targets_unused: Vec<Int>,
    pub pitch_limited: Bool,
    pub leech_weapon_range_active: Bool,
}

impl Weapon {
    pub fn new(template: WeaponTemplate, wslot: WeaponSlotType, current_frame: UnsignedInt) -> Self {
        let suspend = current_frame.wrapping_add(template.suspend_fx_delay);
        let pitch_limited = if template.min_target_pitch > -std::f32::consts::PI || template.max_target_pitch < std::f32::consts::PI { TRUE } else { FALSE };
        let num = template.shots_per_barrel;
        let mut w = Self {
            template,
            template_name: String::new(),
            wslot,
            status: WeaponStatus::OutOfAmmo,
            ammo_in_clip: 0,
            // Weapons start empty with all timers at zero; they must be loaded before use.
            when_we_can_fire_again: 0,
            when_pre_attack_finished: 0,
            when_last_reload_started: 0,
            last_fire_frame: 0,
            suspend_fx_frame: suspend,
            projectile_stream_id: ObjectId::INVALID,
            max_shot_count: NO_MAX_SHOTS_LIMIT,
            cur_barrel: 0,
            num_shots_for_cur_barrel: num,
            scatter_targets_unused: Vec::new(),
            pitch_limited,
            leech_weapon_range_active: FALSE,
        };
        w.template_name = w.template.name.clone();
        w
    }

    /// The `loadAmmoNow` of `WeaponSet::updateWeaponSet`. A new set's weapons start full; an
    /// instant load ignores bonuses, and no reload is shared before the set is installed.
    pub(crate) fn load_at_creation(&mut self, now: UnsignedInt) {
        self.ammo_in_clip = self.template.clip_size as UnsignedInt;
        if self.ammo_in_clip == 0 {
            self.ammo_in_clip = NO_MAX_SHOTS_LIMIT as UnsignedInt;
        }
        self.status = WeaponStatus::ReloadingClip;
        self.when_last_reload_started = now;
        self.when_we_can_fire_again = (now as Real) as UnsignedInt;
        self.rebuild_scatter_targets();
    }

    pub fn get_template(&self) -> &WeaponTemplate { &self.template }
    pub fn get_weapon_slot(&self) -> WeaponSlotType { self.wslot }
    pub fn get_status(&mut self, now: UnsignedInt) -> WeaponStatus {
        if now < self.when_pre_attack_finished { return WeaponStatus::PreAttack; }
        if now >= self.when_we_can_fire_again {
            self.status = if self.ammo_in_clip > 0 { WeaponStatus::ReadyToFire } else { WeaponStatus::OutOfAmmo };
        }
        self.status
    }
    pub fn get_status_const(&self, now: UnsignedInt) -> WeaponStatus {
        if now < self.when_pre_attack_finished { WeaponStatus::PreAttack }
        else if now >= self.when_we_can_fire_again {
            if self.ammo_in_clip > 0 { WeaponStatus::ReadyToFire } else { WeaponStatus::OutOfAmmo }
        } else { self.status }
    }
    pub fn get_remaining_ammo(&mut self, now: UnsignedInt) -> UnsignedInt {
        if self.get_status(now) == WeaponStatus::ReloadingClip { 0 } else { self.ammo_in_clip }
    }
    pub fn get_percent_ready_to_fire(&mut self, now: UnsignedInt) -> Real {
        match self.get_status(now) {
            WeaponStatus::OutOfAmmo | WeaponStatus::PreAttack => 0.0_f32,
            WeaponStatus::ReadyToFire => 1.0_f32,
            WeaponStatus::BetweenFiringShots | WeaponStatus::ReloadingClip => {
                if now >= self.when_we_can_fire_again { return 1.0_f32; }
                let total = self.when_we_can_fire_again.wrapping_sub(self.when_last_reload_started);
                if total == 0 { return 1.0_f32; }
                let time_so_far = now.wrapping_sub(self.when_last_reload_started);
                if time_so_far >= total { 1.0_f32 } else { (time_so_far as Real) / (total as Real) }
            }
            WeaponStatus::Count => 0.0_f32,
        }
    }
    pub fn set_clip_percent_full(&mut self, percent: Real, allow_reduction: Bool, current_frame: UnsignedInt) {
        if self.template.clip_size == 0 { return; }
        // C++ compares the signed result with the unsigned count, so a negative value wraps.
        let ammo = real_to_int_floor((self.template.clip_size as Real) * percent) as UnsignedInt;
        if (ammo > self.ammo_in_clip) || (allow_reduction == TRUE && ammo < self.ammo_in_clip) {
            self.ammo_in_clip = ammo;
            // Inverted on purpose: this is what C++ stores until the next `get_status`.
            self.status = if self.ammo_in_clip != 0 { WeaponStatus::OutOfAmmo } else { WeaponStatus::ReadyToFire };
            self.when_last_reload_started = current_frame;
            self.when_we_can_fire_again = current_frame;
            self.rebuild_scatter_targets();
        }
    }
    pub fn rebuild_scatter_targets(&mut self) {
        self.scatter_targets_unused.clear();
        for i in 0..self.template.scatter_targets.len() as i32 {
            self.scatter_targets_unused.push(i);
        }
    }
    pub fn is_pitch_limited(&self) -> Bool { self.pitch_limited }
    pub fn has_leech_range(&self) -> Bool { self.leech_weapon_range_active }
    pub fn set_leech_range_active(&mut self, active: Bool) { self.leech_weapon_range_active = active; }
    pub fn set_possible_next_shot_frame(&mut self, frame: UnsignedInt) { self.when_we_can_fire_again = frame; }
    pub fn set_pre_attack_finished_frame(&mut self, frame: UnsignedInt) { self.when_pre_attack_finished = frame; }
    pub fn set_last_reload_started_frame(&mut self, frame: UnsignedInt) { self.when_last_reload_started = frame; }
    pub fn transfer_next_shot_stats_from(&mut self, other: &Weapon) {
        // C++ copies `weapon.getStatus()`, which refreshes against the current frame; callers
        // must refresh `other` first.
        self.status = other.status;
        self.when_we_can_fire_again = other.when_we_can_fire_again;
        self.when_last_reload_started = other.when_last_reload_started;
    }
}

impl Snapshot for Weapon {
    fn crc(&self, xfer: &mut dyn Xfer) {
        let mut name = self.template_name.clone();
        let _ = xfer.xfer_ascii_string(&mut name);
        let mut slot = self.wslot.as_i32();
        let _ = xfer.xfer_int(&mut slot);
        let mut ammo = self.ammo_in_clip;
        let _ = xfer.xfer_unsigned_int(&mut ammo);
        let mut v = self.when_we_can_fire_again; let _ = xfer.xfer_unsigned_int(&mut v);
        let mut v = self.when_pre_attack_finished; let _ = xfer.xfer_unsigned_int(&mut v);
        let mut v = self.when_last_reload_started; let _ = xfer.xfer_unsigned_int(&mut v);
        let mut v = self.last_fire_frame; let _ = xfer.xfer_unsigned_int(&mut v);
        let mut id = self.projectile_stream_id; let _ = xfer.xfer_object_id(&mut id);
        let mut invalid = ObjectId::INVALID; let _ = xfer.xfer_object_id(&mut invalid);
        let mut i = self.max_shot_count; let _ = xfer.xfer_int(&mut i);
        let mut i = self.cur_barrel; let _ = xfer.xfer_int(&mut i);
        let mut i = self.num_shots_for_cur_barrel; let _ = xfer.xfer_int(&mut i);
        let mut count = self.scatter_targets_unused.len() as UnsignedShort;
        let _ = xfer.xfer_unsigned_short(&mut count);
        for &target in &self.scatter_targets_unused { let mut i = target; let _ = xfer.xfer_int(&mut i); }
        let mut p = self.pitch_limited; let _ = xfer.xfer_bool(&mut p);
        let mut l = self.leech_weapon_range_active; let _ = xfer.xfer_bool(&mut l);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 3;
        xfer.xfer_version(&mut version, 3)?;
        if version >= 2 {
            let mut name = self.template_name.clone();
            xfer.xfer_ascii_string(&mut name)?;
            if xfer.mode() == XferMode::Load { self.template_name = name; }
        }
        let mut slot = self.wslot.as_i32(); xfer.xfer_int(&mut slot)?; if xfer.mode() == XferMode::Load { self.wslot = WeaponSlotType::from_i32(slot).unwrap_or(WeaponSlotType::Primary); }
        let mut status = self.status.as_i32(); xfer.xfer_int(&mut status)?; if xfer.mode() == XferMode::Load { self.status = WeaponStatus::from_i32(status).unwrap_or(WeaponStatus::OutOfAmmo); }
        xfer.xfer_unsigned_int(&mut self.ammo_in_clip)?;
        xfer.xfer_unsigned_int(&mut self.when_we_can_fire_again)?;
        xfer.xfer_unsigned_int(&mut self.when_pre_attack_finished)?;
        xfer.xfer_unsigned_int(&mut self.when_last_reload_started)?;
        xfer.xfer_unsigned_int(&mut self.last_fire_frame)?;
        if version >= 3 { xfer.xfer_unsigned_int(&mut self.suspend_fx_frame)?; } else { self.suspend_fx_frame = 0; }
        xfer.xfer_object_id(&mut self.projectile_stream_id)?;
        let mut invalid = ObjectId::INVALID; xfer.xfer_object_id(&mut invalid)?;
        xfer.xfer_int(&mut self.max_shot_count)?;
        xfer.xfer_int(&mut self.cur_barrel)?;
        xfer.xfer_int(&mut self.num_shots_for_cur_barrel)?;
        let mut count = self.scatter_targets_unused.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if xfer.mode() == XferMode::Load {
            self.scatter_targets_unused.clear();
            for _ in 0..count { let mut i = 0; xfer.xfer_int(&mut i)?; self.scatter_targets_unused.push(i); }
        } else {
            for target in &mut self.scatter_targets_unused { xfer.xfer_int(target)?; }
        }
        xfer.xfer_bool(&mut self.pitch_limited)?;
        xfer.xfer_bool(&mut self.leech_weapon_range_active)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct WeaponStore {
    pub templates: Vec<WeaponTemplate>,
}

impl WeaponStore {
    pub fn new() -> Self { Self::default() }
    pub fn add_template(&mut self, template: WeaponTemplate) { self.templates.push(template); }
    pub fn find_weapon_template(&self, name: &str) -> Option<&WeaponTemplate> { self.templates.iter().find(|t| t.name == name) }
    pub fn allocate_new_weapon(&self, template: &WeaponTemplate, slot: WeaponSlotType, frame: UnsignedInt) -> Weapon {
        Weapon::new(template.clone(), slot, frame)
    }
}
