//! `PKT-399`: Weapon templates, runtime weapons, reload/clip math, and snapshots.

use crate::gamelogic::global_data::{WeaponBonus, WeaponBonusField, WeaponBonusSet};
use crate::rules::combat_types::{
    DamageType, DeathType, WeaponStatus,
};
use zh_common::ini::{
    convert_duration_from_msecs_to_frames, IniError, IniLoadType, IniReader, SEPS_DEFAULT,
};
use zh_common::math::{gm_ceilf, real_to_int_floor, Coord2D};
use zh_common::random_value::PrngState;
use zh_common::types::{
    Bool, Int, ObjectId, Real, UnsignedInt, UnsignedShort, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

use super::weapon_set::WeaponSlotType;
use crate::rules::object_status_types::{ObjectStatusType, OBJECT_STATUS_NAMES};
use zh_common::ini::VETERANCY_NAMES;

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

/// `TheWeaponReloadNames`.
pub const WEAPON_RELOAD_NAMES: [&str; 3] = ["YES", "NO", "RETURN_TO_BASE"];
pub const WEAPON_PREFIRE_NAMES: [&str; 3] = ["PER_SHOT", "PER_ATTACK", "PER_CLIP"];
/// `TheWeaponAffectsMaskNames`.
pub const WEAPON_AFFECTS_NAMES: [&str; 7] = [
    "SELF", "ALLIES", "ENEMIES", "NEUTRALS", "SUICIDE", "NOT_SIMILAR", "NOT_AIRBORNE",
];
/// `TheWeaponCollideMaskNames`.
pub const WEAPON_COLLIDE_NAMES: [&str; 9] = [
    "ALLIES", "ENEMIES", "STRUCTURES", "SHRUBBERY", "PROJECTILES", "WALLS", "SMALL_MISSILES",
    "BALLISTIC_MISSILES", "CONTROLLED_STRUCTURES",
];

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
    /// `m_projectileName`; empty for `None`.
    pub projectile_name: String,
    /// `m_damageStatusType`.
    pub damage_status_type: ObjectStatusType,
    /// `m_fireOCLNames`, by veterancy level; empty is none.
    pub fire_ocl: [String; 4],
    /// `m_projectileDetonationOCLNames`.
    pub projectile_detonation_ocl: [String; 4],
    pub projectile_stream_name: String,
    pub laser_name: String,
    pub laser_bone_name: String,
    pub historic_bonus_time: UnsignedInt,
    pub historic_bonus_radius: Real,
    pub historic_bonus_count: Int,
    /// Name of `m_historicBonusWeapon`; empty is none.
    pub historic_bonus_weapon: String,
    /// Event name of `m_fireSound`; empty for `NoSound` or unset.
    pub fire_sound: String,
    pub fire_sound_loop_time: UnsignedInt,
    /// `m_fireFXs`, `m_projectileDetonateFXs` and `m_projectileExhausts` by veterancy
    /// level; empty is none.
    pub fire_fx: [String; 4],
    pub projectile_detonation_fx: [String; 4],
    pub projectile_exhaust: [String; 4],
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
            projectile_name: String::new(), damage_status_type: ObjectStatusType::None,
            fire_ocl: Default::default(), projectile_detonation_ocl: Default::default(),
            projectile_stream_name: String::new(), laser_name: String::new(),
            laser_bone_name: String::new(), historic_bonus_time: 0,
            historic_bonus_radius: 0.0, historic_bonus_count: 0,
            historic_bonus_weapon: String::new(),
            fire_sound: String::new(), fire_sound_loop_time: 0,
            fire_fx: Default::default(), projectile_detonation_fx: Default::default(),
            projectile_exhaust: Default::default(),
        }
    }
}

/// A name field where `None` means no reference.
fn name_or_none(value: String) -> String {
    if value.eq_ignore_ascii_case("None") { String::new() } else { value }
}

/// `parseAllVetLevels*`: one value for every veterancy level.
fn parse_all_vet_levels(names: &mut [String; 4], ini: &mut IniReader<'_>) -> Result<(), IniError> {
    let name = name_or_none(ini.next_ascii_string()?);
    *names = std::array::from_fn(|_| name.clone());
    Ok(())
}

/// `parsePerVetLevel*`: a veterancy level name, then the value.
fn parse_per_vet_level(names: &mut [String; 4], ini: &mut IniReader<'_>) -> Result<(), IniError> {
    let level = IniReader::scan_index_list(ini.next_token(SEPS_DEFAULT)?, &VETERANCY_NAMES)?;
    names[level as usize] = name_or_none(ini.next_ascii_string()?);
    Ok(())
}

fn parse_mask(ini: &mut IniReader<'_>, mask: &mut Int, names: &[&str]) -> Result<(), IniError> {
    let mut bits = *mask as UnsignedInt;
    ini.parse_bit_string32(&mut bits, names)?;
    *mask = bits as Int;
    Ok(())
}

fn parse_mask_bit(ini: &mut IniReader<'_>, mask: &mut Int, bit: Int) -> Result<(), IniError> {
    let mut bits = *mask as UnsignedInt;
    ini.parse_bit_in_int32(&mut bits, bit as UnsignedInt)?;
    *mask = bits as Int;
    Ok(())
}

fn parse_frames(ini: &mut IniReader<'_>, frames: &mut Int) -> Result<(), IniError> {
    let mut value = 0;
    ini.parse_duration_unsigned_int(&mut value)?;
    *frames = value as Int;
    Ok(())
}

impl WeaponTemplate {
    pub fn new() -> Self { Self::default() }

    pub fn parse_ini_block(&mut self, text: &str) -> Result<(), IniError> {
        self.parse_ini_block_collect(text, &mut Vec::new())
    }

    /// As [`Self::parse_ini_block`], appending the keys the field table lacks.
    pub fn parse_ini_block_collect(
        &mut self,
        text: &str,
        ignored: &mut Vec<String>,
    ) -> Result<(), IniError> {
        let mut ini = IniReader::new("", text.as_bytes(), IniLoadType::Overwrite);
        ini.read_line(None)?;
        if ini.next_token(SEPS_DEFAULT)? != "Weapon" {
            return Err(IniError::InvalidData);
        }
        self.name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        self.parse_fields(&mut ini, ignored)
    }

    /// `INI::initFromINI` over `TheWeaponTemplateFieldParseTable`, from the line after
    /// the header through `End`. A key outside the table is skipped, as the reference
    /// build does (`DEBUG_CRASH` only), and appended to `ignored`.
    ///
    /// FX list, particle system and weapon names are kept as written; the caller makes
    /// the lookups.
    pub fn parse_fields(
        &mut self,
        ini: &mut IniReader<'_>,
        ignored: &mut Vec<String>,
    ) -> Result<(), IniError> {
        ini.read_block_fields(|key, ini| {
            if !self.parse_field(key, ini)? {
                ignored.push(key.to_owned());
            }
            Ok(())
        })?;
        if self.projectile_name.eq_ignore_ascii_case("None") {
            self.projectile_name.clear();
        }
        Ok(())
    }

    /// One row of the field table; `false` when `key` is not in it. Matching is case
    /// sensitive, as `findFieldParse`'s `strcmp` is.
    fn parse_field(&mut self, key: &str, ini: &mut IniReader<'_>) -> Result<bool, IniError> {
        match key {
            "PrimaryDamage" => ini.parse_real(&mut self.primary_damage)?,
            "PrimaryDamageRadius" => ini.parse_real(&mut self.primary_damage_radius)?,
            "SecondaryDamage" => ini.parse_real(&mut self.secondary_damage)?,
            "SecondaryDamageRadius" => ini.parse_real(&mut self.secondary_damage_radius)?,
            "ShockWaveAmount" => ini.parse_real(&mut self.shock_wave_amount)?,
            "ShockWaveRadius" => ini.parse_real(&mut self.shock_wave_radius)?,
            "ShockWaveTaperOff" => ini.parse_real(&mut self.shock_wave_taper_off)?,
            "AttackRange" => ini.parse_real(&mut self.attack_range)?,
            "MinimumAttackRange" => ini.parse_real(&mut self.minimum_attack_range)?,
            "RequestAssistRange" => ini.parse_real(&mut self.request_assist_range)?,
            "AcceptableAimDelta" => ini.parse_angle_real(&mut self.aim_delta)?,
            "ScatterRadius" => ini.parse_real(&mut self.scatter_radius)?,
            "ScatterTargetScalar" => ini.parse_real(&mut self.scatter_target_scalar)?,
            "ScatterRadiusVsInfantry" => ini.parse_real(&mut self.scatter_radius_vs_infantry)?,
            "DamageType" => {
                let token = ini.next_token(SEPS_DEFAULT)?;
                let i = IniReader::scan_index_list(token, &zh_common::ini::DAMAGE_TYPE_NAMES)?;
                self.damage_type = DamageType::from_i32(i).ok_or(IniError::InvalidData)?;
            }
            "DamageStatusType" => {
                let token = ini.next_token(SEPS_DEFAULT)?;
                let i = IniReader::scan_index_list(token, &OBJECT_STATUS_NAMES)?;
                self.damage_status_type =
                    ObjectStatusType::from_i32(i).ok_or(IniError::InvalidData)?;
            }
            "DeathType" => {
                let token = ini.next_token(SEPS_DEFAULT)?;
                let i = IniReader::scan_index_list(token, &zh_common::ini::DEATH_TYPE_NAMES)?;
                self.death_type = DeathType::from_i32(i).ok_or(IniError::InvalidData)?;
            }
            "WeaponSpeed" => ini.parse_velocity_real(&mut self.weapon_speed)?,
            "MinWeaponSpeed" => ini.parse_velocity_real(&mut self.min_weapon_speed)?,
            "ScaleWeaponSpeed" => ini.parse_bool(&mut self.scale_weapon_speed)?,
            "WeaponRecoil" => ini.parse_angle_real(&mut self.weapon_recoil)?,
            "MinTargetPitch" => ini.parse_angle_real(&mut self.min_target_pitch)?,
            "MaxTargetPitch" => ini.parse_angle_real(&mut self.max_target_pitch)?,
            "RadiusDamageAngle" => ini.parse_angle_real(&mut self.radius_damage_angle)?,
            "ProjectileObject" => ini.parse_ascii_string(&mut self.projectile_name)?,
            "FireSound" => {
                let event = ini.next_token(SEPS_DEFAULT)?;
                if !event.eq_ignore_ascii_case("NoSound") {
                    self.fire_sound = event.to_owned();
                }
            }
            "FireSoundLoopTime" => {
                ini.parse_duration_unsigned_int(&mut self.fire_sound_loop_time)?
            }
            "FireFX" => parse_all_vet_levels(&mut self.fire_fx, ini)?,
            "ProjectileDetonationFX" => {
                parse_all_vet_levels(&mut self.projectile_detonation_fx, ini)?
            }
            "FireOCL" => parse_all_vet_levels(&mut self.fire_ocl, ini)?,
            "ProjectileDetonationOCL" => {
                parse_all_vet_levels(&mut self.projectile_detonation_ocl, ini)?
            }
            "ProjectileExhaust" => parse_all_vet_levels(&mut self.projectile_exhaust, ini)?,
            "VeterancyFireFX" => parse_per_vet_level(&mut self.fire_fx, ini)?,
            "VeterancyProjectileDetonationFX" => {
                parse_per_vet_level(&mut self.projectile_detonation_fx, ini)?
            }
            "VeterancyFireOCL" => parse_per_vet_level(&mut self.fire_ocl, ini)?,
            "VeterancyProjectileDetonationOCL" => {
                parse_per_vet_level(&mut self.projectile_detonation_ocl, ini)?
            }
            "VeterancyProjectileExhaust" => {
                parse_per_vet_level(&mut self.projectile_exhaust, ini)?
            }
            "ClipSize" => ini.parse_int(&mut self.clip_size)?,
            "ContinuousFireOne" => ini.parse_int(&mut self.continuous_fire_one_shots_needed)?,
            "ContinuousFireTwo" => ini.parse_int(&mut self.continuous_fire_two_shots_needed)?,
            "ContinuousFireCoast" => {
                ini.parse_duration_unsigned_int(&mut self.continuous_fire_coast_frames)?
            }
            "AutoReloadWhenIdle" => {
                ini.parse_duration_unsigned_int(&mut self.auto_reload_when_idle_frames)?
            }
            "ClipReloadTime" => parse_frames(ini, &mut self.clip_reload_time)?,
            "DelayBetweenShots" => self.parse_shot_delay(ini)?,
            "ShotsPerBarrel" => ini.parse_int(&mut self.shots_per_barrel)?,
            "DamageDealtAtSelfPosition" => {
                ini.parse_bool(&mut self.damage_dealt_at_self_position)?
            }
            "RadiusDamageAffects" => {
                parse_mask(ini, &mut self.affects_mask, &WEAPON_AFFECTS_NAMES)?
            }
            "ProjectileCollidesWith" => {
                parse_mask(ini, &mut self.collide_mask, &WEAPON_COLLIDE_NAMES)?
            }
            "AntiAirborneVehicle" => {
                parse_mask_bit(ini, &mut self.anti_mask, WEAPON_ANTI_AIRBORNE_VEHICLE)?
            }
            "AntiGround" => parse_mask_bit(ini, &mut self.anti_mask, WEAPON_ANTI_GROUND)?,
            "AntiProjectile" => parse_mask_bit(ini, &mut self.anti_mask, WEAPON_ANTI_PROJECTILE)?,
            "AntiSmallMissile" => {
                parse_mask_bit(ini, &mut self.anti_mask, WEAPON_ANTI_SMALL_MISSILE)?
            }
            "AntiMine" => parse_mask_bit(ini, &mut self.anti_mask, WEAPON_ANTI_MINE)?,
            "AntiParachute" => parse_mask_bit(ini, &mut self.anti_mask, WEAPON_ANTI_PARACHUTE)?,
            "AntiAirborneInfantry" => {
                parse_mask_bit(ini, &mut self.anti_mask, WEAPON_ANTI_AIRBORNE_INFANTRY)?
            }
            "AntiBallisticMissile" => {
                parse_mask_bit(ini, &mut self.anti_mask, WEAPON_ANTI_BALLISTIC_MISSILE)?
            }
            "AutoReloadsClip" => {
                let token = ini.next_token(SEPS_DEFAULT)?;
                let i = IniReader::scan_index_list(token, &WEAPON_RELOAD_NAMES)?;
                self.reload_type = WeaponReloadType::from_i32(i).ok_or(IniError::InvalidData)?;
            }
            "ProjectileStreamName" => ini.parse_ascii_string(&mut self.projectile_stream_name)?,
            "LaserName" => ini.parse_ascii_string(&mut self.laser_name)?,
            "LaserBoneName" => ini.parse_ascii_string(&mut self.laser_bone_name)?,
            "WeaponBonus" => self.extra_bonus.parse_weapon_bonus_set(ini)?,
            "HistoricBonusTime" => {
                ini.parse_duration_unsigned_int(&mut self.historic_bonus_time)?
            }
            "HistoricBonusRadius" => ini.parse_real(&mut self.historic_bonus_radius)?,
            "HistoricBonusCount" => ini.parse_int(&mut self.historic_bonus_count)?,
            "HistoricBonusWeapon" => {
                let token = ini.next_token(SEPS_DEFAULT)?.to_owned();
                self.historic_bonus_weapon = name_or_none(token);
            }
            "LeechRangeWeapon" => ini.parse_bool(&mut self.leech_range_weapon)?,
            "ScatterTarget" => {
                let mut target = Coord2D { x: 0.0, y: 0.0 };
                ini.parse_coord2d(&mut target)?;
                self.scatter_targets.push(target);
            }
            "CapableOfFollowingWaypoints" => {
                ini.parse_bool(&mut self.capable_of_following_waypoint)?
            }
            "ShowsAmmoPips" => ini.parse_bool(&mut self.is_shows_ammo_pips)?,
            "AllowAttackGarrisonedBldgs" => {
                ini.parse_bool(&mut self.allow_attack_garrisoned_buildings)?
            }
            "PlayFXWhenStealthed" => ini.parse_bool(&mut self.play_fx_when_stealthed)?,
            "PreAttackDelay" => parse_frames(ini, &mut self.pre_attack_delay)?,
            "PreAttackType" => {
                let token = ini.next_token(SEPS_DEFAULT)?;
                let i = IniReader::scan_index_list(token, &WEAPON_PREFIRE_NAMES)?;
                self.prefire_type = WeaponPrefireType::from_i32(i).ok_or(IniError::InvalidData)?;
            }
            "ContinueAttackRange" => ini.parse_real(&mut self.continue_attack_range)?,
            "SuspendFXDelay" => ini.parse_duration_unsigned_int(&mut self.suspend_fx_delay)?,
            "MissileCallsOnDie" => ini.parse_bool(&mut self.missile_calls_on_die)?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// `WeaponTemplate::parseShotDelay`: `<msec>` or `Min:<msec> Max:<msec>`.
    fn parse_shot_delay(&mut self, ini: &mut IniReader<'_>) -> Result<(), IniError> {
        let seps = zh_common::ini::SEPS_COLON;
        let first = ini.next_token(seps)?.to_owned();
        let (min, max) = if first.eq_ignore_ascii_case("Min") {
            let min = IniReader::scan_int(ini.next_token(seps)?)?;
            if ini.next_token(seps)?.eq_ignore_ascii_case("Max") {
                (min, IniReader::scan_int(ini.next_token(seps)?)?)
            } else {
                (min, min)
            }
        } else {
            let both = IniReader::scan_int(&first)?;
            (both, both)
        };
        let frames = |msec: Int| gm_ceilf(convert_duration_from_msecs_to_frames(msec as Real)) as Int;
        self.min_delay_between_shots = frames(min);
        self.max_delay_between_shots = frames(max);
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
    /// Whether `INI::parseWeaponTemplate` finds a template: `None` never names one.
    pub fn has_template(&self, name: &str) -> bool {
        !name.eq_ignore_ascii_case("None") && self.find_weapon_template(name).is_some()
    }
    pub fn find_weapon_template(&self, name: &str) -> Option<&WeaponTemplate> { self.templates.iter().find(|t| t.name == name) }
    pub fn allocate_new_weapon(&self, template: &WeaponTemplate, slot: WeaponSlotType, frame: UnsignedInt) -> Weapon {
        Weapon::new(template.clone(), slot, frame)
    }
}
