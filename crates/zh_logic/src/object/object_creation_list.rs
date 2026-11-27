//! `PKT-391` (data only): the contents of `ObjectCreationList` blocks, as
//! `GameLogic/Object/ObjectCreationList.cpp` parses them, with
//! `DeliverPayloadData::getFieldParse` and `RadiusDecalTemplate::parseRadiusDecalTemplate`.
//! Creating the objects (`ObjectCreationNugget::create`) is not ported.
//!
//! No packet document; the oracle is `tests/cpp_harness/gen_game_data_load.cpp` and the
//! golden test `tests/golden_game_data_load.rs`.
//!
//! A key outside a field table is skipped, as the reference build does (`DEBUG_CRASH`
//! only), and appended to `ignored`. With the `retail-crc` feature `ObjectNames` and
//! `ModelNames` keep every other name, as the retail executable does.

use std::collections::{BTreeMap, BTreeSet};

use zh_common::ini::{IniError, IniReader, SECONDS_PER_LOGICFRAME_REAL, SEPS_DEFAULT};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, Real, UnsignedInt, FALSE, TRUE};

use crate::object::thing_template::THE_SHADOW_NAMES;
use crate::object::weapon::WeaponStore;

/// `DebrisDispositionNames`.
pub const DEBRIS_DISPOSITION_NAMES: [&str; 9] = [
    "LIKE_EXISTING",
    "ON_GROUND_ALIGNED",
    "SEND_IT_FLYING",
    "SEND_IT_UP",
    "SEND_IT_OUT",
    "RANDOM_FORCE",
    "FLOATING",
    "INHERIT_VELOCITY",
    "WHIRLING",
];
/// `ON_GROUND_ALIGNED`.
pub const DEBRIS_ON_GROUND_ALIGNED: UnsignedInt = 0x02;
/// `SHADOW_ALPHA_DECAL`.
const SHADOW_ALPHA_DECAL: UnsignedInt = 0x20;
/// `StaticGameLODNames`.
const STATIC_GAME_LOD_NAMES: [&str; 5] = ["Low", "Medium", "High", "VeryHigh", "Custom"];
/// `TheWeaponSlotTypeNamesLookupList`.
const WEAPON_SLOT_NAMES: [&str; 3] = ["PRIMARY", "SECONDARY", "TERTIARY"];

/// What the parsers look names up in. An unknown name leaves the reference empty.
#[derive(Clone, Copy)]
pub struct OclRefs<'a> {
    pub weapon_store: &'a WeaponStore,
    pub fx_list_names: &'a BTreeSet<String>,
}

impl OclRefs<'_> {
    /// `INI::parseWeaponTemplate`.
    fn weapon(&self, ini: &mut IniReader<'_>) -> Result<String, IniError> {
        let name = ini.next_token(SEPS_DEFAULT)?;
        Ok(if self.weapon_store.has_template(name) {
            name.to_owned()
        } else {
            String::new()
        })
    }

    /// `INI::parseFXList`.
    fn fx_list(&self, ini: &mut IniReader<'_>) -> Result<String, IniError> {
        let name = ini.next_token(SEPS_DEFAULT)?;
        Ok(if self.fx_list_names.contains(name) {
            name.to_owned()
        } else {
            String::new()
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadiusDecalTemplate {
    pub name: String,
    pub shadow_type: UnsignedInt,
    pub min_opacity: Real,
    pub max_opacity: Real,
    pub opacity_throb_time: UnsignedInt,
    pub color: UnsignedInt,
    pub only_visible_to_owning_player: Bool,
}

impl Default for RadiusDecalTemplate {
    fn default() -> Self {
        Self {
            name: String::new(),
            shadow_type: SHADOW_ALPHA_DECAL,
            min_opacity: 1.0,
            max_opacity: 1.0,
            opacity_throb_time: zh_common::ini::LOGICFRAMES_PER_SECOND as UnsignedInt,
            color: 0,
            only_visible_to_owning_player: TRUE,
        }
    }
}

impl RadiusDecalTemplate {
    /// `parseRadiusDecalTemplate`: a nested block.
    pub fn parse(
        &mut self,
        ini: &mut IniReader<'_>,
        ignored: &mut Vec<String>,
    ) -> Result<(), IniError> {
        ini.read_block_fields(|key, ini| {
            match key {
                "Texture" => ini.parse_ascii_string(&mut self.name)?,
                "Style" => ini.parse_bit_string32(&mut self.shadow_type, &THE_SHADOW_NAMES)?,
                "OpacityMin" => ini.parse_percent_to_real(&mut self.min_opacity)?,
                "OpacityMax" => ini.parse_percent_to_real(&mut self.max_opacity)?,
                "OpacityThrobTime" => {
                    ini.parse_duration_unsigned_int(&mut self.opacity_throb_time)?
                }
                "Color" => ini.parse_color_int(&mut self.color)?,
                "OnlyVisibleToOwningPlayer" => {
                    ini.parse_bool(&mut self.only_visible_to_owning_player)?
                }
                _ => ignored.push(key.to_owned()),
            }
            Ok(())
        })
    }
}

/// `DeliverPayloadData`.
#[derive(Debug, Clone, PartialEq)]
pub struct DeliverPayloadData {
    pub visible_drop_bone_name: String,
    pub visible_sub_object_name: String,
    pub visible_payload_template_name: String,
    pub dist_to_target: Real,
    pub pre_open_distance: Real,
    pub max_attempts: Int,
    pub drop_offset: Coord3D,
    pub drop_variance: Coord3D,
    pub drop_delay: UnsignedInt,
    pub fire_weapon: Bool,
    pub self_destruct_object: Bool,
    pub visible_num_bones: Int,
    pub dive_start_distance: Real,
    pub dive_end_distance: Real,
    /// -1 is no slot.
    pub strafing_weapon_slot: Int,
    pub visible_items_dropped_per_interval: Int,
    pub inherit_transport_velocity: Bool,
    pub is_parachute_directly: Bool,
    pub exit_pitch_rate: Real,
    pub strafe_fx: String,
    pub strafe_length: Real,
    pub visible_payload_weapon: String,
    pub delivery_decal: RadiusDecalTemplate,
    pub delivery_decal_radius: Real,
}

impl Default for DeliverPayloadData {
    fn default() -> Self {
        Self {
            visible_drop_bone_name: String::new(),
            visible_sub_object_name: String::new(),
            visible_payload_template_name: String::new(),
            dist_to_target: 0.0,
            pre_open_distance: 0.0,
            max_attempts: 1,
            drop_offset: Coord3D::default(),
            drop_variance: Coord3D::default(),
            drop_delay: 0,
            fire_weapon: FALSE,
            self_destruct_object: FALSE,
            visible_num_bones: 0,
            dive_start_distance: 0.0,
            dive_end_distance: 0.0,
            strafing_weapon_slot: -1,
            visible_items_dropped_per_interval: 0,
            inherit_transport_velocity: FALSE,
            is_parachute_directly: FALSE,
            exit_pitch_rate: 0.0,
            strafe_fx: String::new(),
            strafe_length: 0.0,
            visible_payload_weapon: String::new(),
            delivery_decal: RadiusDecalTemplate::default(),
            delivery_decal_radius: 0.0,
        }
    }
}

impl DeliverPayloadData {
    /// One row of `DeliverPayloadData::getFieldParse`; `false` when `key` is not in it.
    pub fn parse_field(
        &mut self,
        key: &str,
        ini: &mut IniReader<'_>,
        refs: OclRefs<'_>,
        ignored: &mut Vec<String>,
    ) -> Result<bool, IniError> {
        match key {
            "DeliveryDistance" => ini.parse_real(&mut self.dist_to_target)?,
            "PreOpenDistance" => ini.parse_real(&mut self.pre_open_distance)?,
            "MaxAttempts" => ini.parse_int(&mut self.max_attempts)?,
            "DropDelay" => ini.parse_duration_unsigned_int(&mut self.drop_delay)?,
            "DropOffset" => ini.parse_coord3d(&mut self.drop_offset)?,
            "DropVariance" => ini.parse_coord3d(&mut self.drop_variance)?,
            "InheritTransportVelocity" => ini.parse_bool(&mut self.inherit_transport_velocity)?,
            "ExitPitchRate" => ini.parse_angular_velocity_real(&mut self.exit_pitch_rate)?,
            "ParachuteDirectly" => ini.parse_bool(&mut self.is_parachute_directly)?,
            "VisibleItemsDroppedPerInterval" => {
                ini.parse_int(&mut self.visible_items_dropped_per_interval)?
            }
            "VisibleDropBoneBaseName" => {
                ini.parse_ascii_string(&mut self.visible_drop_bone_name)?
            }
            "VisibleSubObjectBaseName" => {
                ini.parse_ascii_string(&mut self.visible_sub_object_name)?
            }
            "VisibleNumBones" => ini.parse_int(&mut self.visible_num_bones)?,
            "VisiblePayloadTemplateName" => {
                ini.parse_ascii_string(&mut self.visible_payload_template_name)?
            }
            "VisiblePayloadWeaponTemplate" => self.visible_payload_weapon = refs.weapon(ini)?,
            "SelfDestructObject" => ini.parse_bool(&mut self.self_destruct_object)?,
            "FireWeapon" => ini.parse_bool(&mut self.fire_weapon)?,
            "DiveStartDistance" => ini.parse_real(&mut self.dive_start_distance)?,
            "DiveEndDistance" => ini.parse_real(&mut self.dive_end_distance)?,
            "StrafingWeaponSlot" => {
                self.strafing_weapon_slot = weapon_slot(ini)?;
            }
            "StrafeWeaponFX" => self.strafe_fx = refs.fx_list(ini)?,
            "StrafeLength" => ini.parse_real(&mut self.strafe_length)?,
            "DeliveryDecal" => self.delivery_decal.parse(ini, ignored)?,
            "DeliveryDecalRadius" => ini.parse_real(&mut self.delivery_decal_radius)?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

fn weapon_slot(ini: &mut IniReader<'_>) -> Result<Int, IniError> {
    IniReader::scan_index_list(ini.next_token(SEPS_DEFAULT)?, &WEAPON_SLOT_NAMES)
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimSet {
    pub initial: String,
    pub flying: String,
    pub final_anim: String,
}

/// `GenericObjectCreationNugget`: `CreateObject` and `CreateDebris`.
#[derive(Debug, Clone, PartialEq)]
pub struct GenericNugget {
    /// `CreateObject` (templates) rather than `CreateDebris` (models).
    pub names_are_objects: Bool,
    pub names: Vec<String>,
    pub put_in_container: String,
    pub anim_sets: Vec<AnimSet>,
    pub fx_final: String,
    pub particle_system_name: String,
    pub debris_to_generate: Int,
    pub mass: Real,
    pub extra_bounciness: Real,
    pub extra_friction: Real,
    pub offset: Coord3D,
    /// `DebrisDisposition` bits.
    pub disposition: UnsignedInt,
    pub disposition_intensity: Real,
    pub spin_rate: Real,
    pub yaw_rate: Real,
    pub roll_rate: Real,
    pub pitch_rate: Real,
    pub min_mag: Real,
    pub max_mag: Real,
    pub min_pitch: Real,
    pub max_pitch: Real,
    pub min_frames: UnsignedInt,
    pub max_frames: UnsignedInt,
    pub shadow_type: UnsignedInt,
    /// `StaticGameLODLevel`.
    pub min_lod_required: Int,
    pub invulnerable_time: UnsignedInt,
    pub min_health: Real,
    pub max_health: Real,
    pub fade_frames: UnsignedInt,
    pub fade_sound_name: String,
    pub min_distance_a_formation: Real,
    pub min_distance_b_formation: Real,
    pub max_distance_formation: Real,
    pub object_count: Int,
    pub bounce_sound: String,
    pub requires_live_player: Bool,
    pub contain_inside_source_object: Bool,
    pub preserve_layer: Bool,
    pub ok_to_change_model_color: Bool,
    pub orient_in_force_direction: Bool,
    pub spread_formation: Bool,
    pub fade_in: Bool,
    pub fade_out: Bool,
    pub ignore_primary_obstacle: Bool,
    pub inherits_veterancy: Bool,
    pub dies_on_bad_land: Bool,
    pub skip_if_significantly_airborne: Bool,
}

impl GenericNugget {
    fn new(names_are_objects: Bool) -> Self {
        Self {
            names_are_objects,
            names: Vec::new(),
            put_in_container: String::new(),
            anim_sets: Vec::new(),
            fx_final: String::new(),
            particle_system_name: String::new(),
            debris_to_generate: 1,
            mass: 0.0,
            extra_bounciness: 0.0,
            extra_friction: 0.0,
            offset: Coord3D::default(),
            disposition: DEBRIS_ON_GROUND_ALIGNED,
            disposition_intensity: 0.0,
            spin_rate: -1.0,
            yaw_rate: -1.0,
            roll_rate: -1.0,
            pitch_rate: -1.0,
            min_mag: 0.0,
            max_mag: 0.0,
            min_pitch: 0.0,
            max_pitch: 0.0,
            min_frames: 0,
            max_frames: 0,
            shadow_type: 0,
            min_lod_required: 0,
            invulnerable_time: 0,
            min_health: 1.0,
            max_health: 1.0,
            fade_frames: 0,
            fade_sound_name: String::new(),
            min_distance_a_formation: 0.0,
            min_distance_b_formation: 0.0,
            max_distance_formation: 0.0,
            object_count: 0,
            bounce_sound: String::new(),
            requires_live_player: FALSE,
            contain_inside_source_object: FALSE,
            preserve_layer: TRUE,
            ok_to_change_model_color: FALSE,
            orient_in_force_direction: FALSE,
            spread_formation: FALSE,
            fade_in: FALSE,
            fade_out: FALSE,
            ignore_primary_obstacle: FALSE,
            inherits_veterancy: FALSE,
            dies_on_bad_land: FALSE,
            skip_if_significantly_airborne: FALSE,
        }
    }

    /// `parseDebrisObjectNames`.
    fn parse_names(&mut self, ini: &mut IniReader<'_>) -> Result<(), IniError> {
        let first = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let mut token = Some(first);
        while let Some(name) = token {
            self.names.push(name);
            if cfg!(feature = "retail-crc") {
                let _skipped = ini.next_token_or_null(SEPS_DEFAULT);
            }
            token = ini.next_token_or_null(SEPS_DEFAULT).map(str::to_owned);
        }
        Ok(())
    }

    /// `getCommonFieldParse`.
    fn parse_common(&mut self, key: &str, ini: &mut IniReader<'_>) -> Result<bool, IniError> {
        match key {
            "PutInContainer" => ini.parse_ascii_string(&mut self.put_in_container)?,
            "ParticleSystem" => ini.parse_ascii_string(&mut self.particle_system_name)?,
            "Count" => ini.parse_int(&mut self.debris_to_generate)?,
            "IgnorePrimaryObstacle" => ini.parse_bool(&mut self.ignore_primary_obstacle)?,
            "OrientInForceDirection" => ini.parse_bool(&mut self.orient_in_force_direction)?,
            "ExtraBounciness" => ini.parse_real(&mut self.extra_bounciness)?,
            "ExtraFriction" => {
                ini.parse_real(&mut self.extra_friction)?;
                self.extra_friction *= SECONDS_PER_LOGICFRAME_REAL;
            }
            "Offset" => ini.parse_coord3d(&mut self.offset)?,
            "Disposition" => {
                ini.parse_bit_string32(&mut self.disposition, &DEBRIS_DISPOSITION_NAMES)?
            }
            "DispositionIntensity" => ini.parse_real(&mut self.disposition_intensity)?,
            "SpinRate" => ini.parse_angular_velocity_real(&mut self.spin_rate)?,
            "YawRate" => ini.parse_angular_velocity_real(&mut self.yaw_rate)?,
            "RollRate" => ini.parse_angular_velocity_real(&mut self.roll_rate)?,
            "PitchRate" => ini.parse_angular_velocity_real(&mut self.pitch_rate)?,
            "MinForceMagnitude" => ini.parse_real(&mut self.min_mag)?,
            "MaxForceMagnitude" => ini.parse_real(&mut self.max_mag)?,
            "MinForcePitch" => ini.parse_angle_real(&mut self.min_pitch)?,
            "MaxForcePitch" => ini.parse_angle_real(&mut self.max_pitch)?,
            "MinLifetime" => ini.parse_duration_unsigned_int(&mut self.min_frames)?,
            "MaxLifetime" => ini.parse_duration_unsigned_int(&mut self.max_frames)?,
            "SpreadFormation" => ini.parse_bool(&mut self.spread_formation)?,
            "MinDistanceAFormation" => ini.parse_real(&mut self.min_distance_a_formation)?,
            "MinDistanceBFormation" => ini.parse_real(&mut self.min_distance_b_formation)?,
            "MaxDistanceFormation" => ini.parse_real(&mut self.max_distance_formation)?,
            "FadeIn" => ini.parse_bool(&mut self.fade_in)?,
            "FadeOut" => ini.parse_bool(&mut self.fade_out)?,
            "FadeTime" => ini.parse_duration_unsigned_int(&mut self.fade_frames)?,
            "FadeSound" => ini.parse_ascii_string(&mut self.fade_sound_name)?,
            "PreserveLayer" => ini.parse_bool(&mut self.preserve_layer)?,
            "DiesOnBadLand" => ini.parse_bool(&mut self.dies_on_bad_land)?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn parse_object_field(&mut self, key: &str, ini: &mut IniReader<'_>) -> Result<bool, IniError> {
        match key {
            "ContainInsideSourceObject" => {
                ini.parse_bool(&mut self.contain_inside_source_object)?
            }
            "ObjectNames" => self.parse_names(ini)?,
            "ObjectCount" => ini.parse_int(&mut self.object_count)?,
            "InheritsVeterancy" => ini.parse_bool(&mut self.inherits_veterancy)?,
            "SkipIfSignificantlyAirborne" => {
                ini.parse_bool(&mut self.skip_if_significantly_airborne)?
            }
            "InvulnerableTime" => ini.parse_duration_unsigned_int(&mut self.invulnerable_time)?,
            "MinHealth" => ini.parse_percent_to_real(&mut self.min_health)?,
            "MaxHealth" => ini.parse_percent_to_real(&mut self.max_health)?,
            "RequiresLivePlayer" => ini.parse_bool(&mut self.requires_live_player)?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn parse_debris_field(
        &mut self,
        key: &str,
        ini: &mut IniReader<'_>,
        refs: OclRefs<'_>,
    ) -> Result<bool, IniError> {
        match key {
            "ModelNames" => self.parse_names(ini)?,
            "Mass" => ini.parse_positive_non_zero_real(&mut self.mass)?,
            "AnimationSet" => {
                let initial = ini.next_ascii_string()?;
                let flying = ini.next_ascii_string()?;
                let final_anim = ini.next_ascii_string()?;
                self.anim_sets.push(AnimSet {
                    initial,
                    flying,
                    final_anim,
                });
            }
            "FXFinal" => self.fx_final = refs.fx_list(ini)?,
            "OkToChangeModelColor" => ini.parse_bool(&mut self.ok_to_change_model_color)?,
            "MinLODRequired" => {
                self.min_lod_required = IniReader::scan_index_list(
                    ini.next_token(SEPS_DEFAULT)?,
                    &STATIC_GAME_LOD_NAMES,
                )?;
            }
            "Shadow" => ini.parse_bit_string32(&mut self.shadow_type, &THE_SHADOW_NAMES)?,
            "BounceSound" => {
                let event = ini.next_token(SEPS_DEFAULT)?;
                if !event.eq_ignore_ascii_case("NoSound") {
                    self.bounce_sound = event.to_owned();
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RandomForceNugget {
    pub spin_rate: Real,
    pub min_mag: Real,
    pub max_mag: Real,
    pub min_pitch: Real,
    pub max_pitch: Real,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Payload {
    pub name: String,
    pub count: Int,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeliverPayloadNugget {
    pub transport_name: String,
    pub put_in_container: String,
    pub start_at_preferred_height: Bool,
    pub start_at_max_speed: Bool,
    pub formation_size: UnsignedInt,
    pub formation_spacing: Real,
    pub convergence_factor: Real,
    pub error_radius: Real,
    pub delay_delivery_frames_max: UnsignedInt,
    pub payload: Vec<Payload>,
    pub data: DeliverPayloadData,
}

impl Default for DeliverPayloadNugget {
    fn default() -> Self {
        Self {
            transport_name: String::new(),
            put_in_container: String::new(),
            start_at_preferred_height: TRUE,
            start_at_max_speed: FALSE,
            formation_size: 1,
            formation_spacing: 25.0,
            convergence_factor: 0.0,
            error_radius: 0.0,
            delay_delivery_frames_max: 0,
            payload: Vec::new(),
            data: DeliverPayloadData::default(),
        }
    }
}

impl DeliverPayloadNugget {
    fn parse_field(&mut self, key: &str, ini: &mut IniReader<'_>) -> Result<bool, IniError> {
        match key {
            "Transport" => ini.parse_ascii_string(&mut self.transport_name)?,
            "StartAtPreferredHeight" => ini.parse_bool(&mut self.start_at_preferred_height)?,
            "StartAtMaxSpeed" => ini.parse_bool(&mut self.start_at_max_speed)?,
            "FormationSize" => ini.parse_unsigned_int(&mut self.formation_size)?,
            "FormationSpacing" => ini.parse_real(&mut self.formation_spacing)?,
            "WeaponConvergenceFactor" => ini.parse_real(&mut self.convergence_factor)?,
            "WeaponErrorRadius" => ini.parse_real(&mut self.error_radius)?,
            "DelayDeliveryMax" => {
                ini.parse_duration_unsigned_int(&mut self.delay_delivery_frames_max)?
            }
            "Payload" => {
                let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
                let count = match ini.next_token_or_null(SEPS_DEFAULT) {
                    Some(token) => IniReader::scan_int(token)?,
                    None => 1,
                };
                self.payload.push(Payload { name, count });
            }
            "PutInContainer" => ini.parse_ascii_string(&mut self.put_in_container)?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttackNugget {
    pub number_of_shots: Int,
    pub weapon_slot: Int,
    pub delivery_decal: RadiusDecalTemplate,
    pub delivery_decal_radius: Real,
}

impl Default for AttackNugget {
    fn default() -> Self {
        Self {
            number_of_shots: 1,
            weapon_slot: 0,
            delivery_decal: RadiusDecalTemplate::default(),
            delivery_decal_radius: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectCreationNugget {
    /// `CreateObject` and `CreateDebris`.
    Generic(Box<GenericNugget>),
    ApplyRandomForce(RandomForceNugget),
    DeliverPayload(Box<DeliverPayloadNugget>),
    /// The weapon's name; empty is none.
    FireWeapon(String),
    Attack(AttackNugget),
}

/// Reads one nugget block, `None` when `key` names no nugget.
fn parse_nugget(
    key: &str,
    ini: &mut IniReader<'_>,
    refs: OclRefs<'_>,
    ignored: &mut Vec<String>,
) -> Result<Option<ObjectCreationNugget>, IniError> {
    let nugget = match key {
        "CreateObject" | "CreateDebris" => {
            let objects = key == "CreateObject";
            let mut n = GenericNugget::new(if objects { TRUE } else { FALSE });
            ini.read_block_fields(|key, ini| {
                let known = n.parse_common(key, ini)?
                    || if objects {
                        n.parse_object_field(key, ini)?
                    } else {
                        n.parse_debris_field(key, ini, refs)?
                    };
                if !known {
                    ignored.push(key.to_owned());
                }
                Ok(())
            })?;
            ObjectCreationNugget::Generic(Box::new(n))
        }
        "ApplyRandomForce" => {
            let mut n = RandomForceNugget::default();
            ini.read_block_fields(|key, ini| {
                match key {
                    "SpinRate" => ini.parse_angular_velocity_real(&mut n.spin_rate)?,
                    "MinForceMagnitude" => ini.parse_real(&mut n.min_mag)?,
                    "MaxForceMagnitude" => ini.parse_real(&mut n.max_mag)?,
                    "MinForcePitch" => ini.parse_angle_real(&mut n.min_pitch)?,
                    "MaxForcePitch" => ini.parse_angle_real(&mut n.max_pitch)?,
                    _ => ignored.push(key.to_owned()),
                }
                Ok(())
            })?;
            ObjectCreationNugget::ApplyRandomForce(n)
        }
        "DeliverPayload" => {
            let mut n = DeliverPayloadNugget::default();
            ini.read_block_fields(|key, ini| {
                if !n.parse_field(key, ini)? && !n.data.parse_field(key, ini, refs, ignored)? {
                    ignored.push(key.to_owned());
                }
                Ok(())
            })?;
            ObjectCreationNugget::DeliverPayload(Box::new(n))
        }
        "FireWeapon" => {
            let mut weapon = String::new();
            ini.read_block_fields(|key, ini| {
                if key == "Weapon" {
                    weapon = refs.weapon(ini)?;
                } else {
                    ignored.push(key.to_owned());
                }
                Ok(())
            })?;
            ObjectCreationNugget::FireWeapon(weapon)
        }
        "Attack" => {
            let mut n = AttackNugget::default();
            ini.read_block_fields(|key, ini| {
                match key {
                    "NumberOfShots" => ini.parse_int(&mut n.number_of_shots)?,
                    "WeaponSlot" => n.weapon_slot = weapon_slot(ini)?,
                    "DeliveryDecal" => n.delivery_decal.parse(ini, ignored)?,
                    "DeliveryDecalRadius" => ini.parse_real(&mut n.delivery_decal_radius)?,
                    _ => ignored.push(key.to_owned()),
                }
                Ok(())
            })?;
            ObjectCreationNugget::Attack(n)
        }
        _ => return Ok(None),
    };
    Ok(Some(nugget))
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObjectCreationList {
    /// In definition order, which is the order of creation.
    pub nuggets: Vec<ObjectCreationNugget>,
}

/// `TheObjectCreationListStore`. Names are case sensitive.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObjectCreationListStore {
    lists: BTreeMap<String, ObjectCreationList>,
}

impl ObjectCreationListStore {
    /// `findObjectCreationList`.
    pub fn find(&self, name: &str) -> Option<&ObjectCreationList> {
        if name.eq_ignore_ascii_case("None") {
            return None;
        }
        self.lists.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &ObjectCreationList)> {
        self.lists.iter()
    }

    /// `parseObjectCreationListDefinition`, from the token after the block type. A
    /// second definition replaces the list.
    pub fn parse_definition(
        &mut self,
        ini: &mut IniReader<'_>,
        refs: OclRefs<'_>,
        ignored: &mut Vec<String>,
    ) -> Result<(), IniError> {
        let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let list = self.lists.entry(name).or_default();
        list.nuggets.clear();
        ini.read_block_fields(|key, ini| {
            match parse_nugget(key, ini, refs, ignored)? {
                Some(nugget) => list.nuggets.push(nugget),
                None => ignored.push(key.to_owned()),
            }
            Ok(())
        })
    }
}
