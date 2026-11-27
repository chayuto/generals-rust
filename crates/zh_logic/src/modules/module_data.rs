//! `ModuleData` of the logic modules whose behaviour is not ported: the field tables,
//! so that the retail object data parses completely. Shared pieces and registration.
//!
//! A module declared here has a data parser and no constructor, so
//! [`ModuleRegistry::is_ported`] stays false and no instance is created. Field names
//! match exactly, as `findFieldParse` does (`strcmp`).
//!
//! Golden test: `crates/zh_logic/tests/golden_logic_module_data.rs`, over the reference
//! parsers (`tests/cpp_harness/gen_logic_module_data.cpp`).

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::{Coord3D, RGBColor};
use zh_common::types::{Bool, Int, NameKeyType, Real, UnsignedInt, VeterancyLevel};
use zh_common::xfer::bit_flags::BitFlags;

use crate::combat::locomotor_set::LOCOMOTOR_SET_NAMES;
use crate::modules::behavior::slow_death_behavior::SlowDeathBehaviorModuleData;
use crate::modules::contain::open_contain::OpenContainModuleData;
use crate::modules::contain::transport_contain::TransportContainModuleData;
use crate::modules::die::die_module::{get_veterancy_level_flag, DieMuxData};
use crate::modules::module::ModuleData;
use crate::modules::module_factory::{
    BuiltinModuleEntry, ModuleDataParseFn, ModuleParseContext, ModuleRegistry, BUILTIN_ZH_MODULES,
};
use crate::modules::update::ai_update::AiUpdateModuleData;
use crate::modules::update::dock_update::DockUpdateModuleData;
use crate::modules::update::dynamic_geometry_info_update::DynamicGeometryInfoUpdateModuleData;
use crate::modules::update::physics_update::{height_to_speed, PhysicsBehaviorModuleData};
use crate::modules::upgrade::upgrade_module::UpgradeMuxData;
use crate::rules::kind_of::{KindOfMaskType, KINDOF_NAMES};

/// `(field table name, member value)` pairs.
pub type FieldList = Vec<(String, String)>;

/// A member written the way the reference oracle writes it.
pub trait FieldText {
    fn field_text(&self) -> String;

    /// For a member the reference derives from gravity.
    fn field_text_at(&self, _gravity: Real) -> String {
        self.field_text()
    }
}

impl FieldText for Real {
    fn field_text(&self) -> String {
        format!("{:08x}", self.to_bits())
    }
}

impl FieldText for Int {
    fn field_text(&self) -> String {
        self.to_string()
    }
}

impl FieldText for UnsignedInt {
    fn field_text(&self) -> String {
        self.to_string()
    }
}

impl FieldText for Bool {
    fn field_text(&self) -> String {
        self.to_string()
    }
}

impl FieldText for String {
    fn field_text(&self) -> String {
        self.clone()
    }
}

/// A resolved reference; `None` is the reference's null pointer.
impl FieldText for Option<String> {
    fn field_text(&self) -> String {
        self.clone().unwrap_or_default()
    }
}

impl FieldText for Vec<String> {
    fn field_text(&self) -> String {
        self.join(",")
    }
}

impl FieldText for Vec<Option<String>> {
    fn field_text(&self) -> String {
        self.iter()
            .map(FieldText::field_text)
            .collect::<Vec<_>>()
            .join(",")
    }
}

impl FieldText for Coord3D {
    fn field_text(&self) -> String {
        [self.x, self.y, self.z].map(|v| v.field_text()).join(",")
    }
}

impl FieldText for RGBColor {
    fn field_text(&self) -> String {
        [self.red, self.green, self.blue]
            .map(|v| v.field_text())
            .join(",")
    }
}

impl<const N: usize> FieldText for BitFlags<N> {
    fn field_text(&self) -> String {
        (0..N)
            .map(|i| if self.test(i as Int) != 0 { '1' } else { '0' })
            .collect()
    }
}

pub(crate) fn push<T: FieldText>(out: &mut FieldList, name: &str, value: &T) {
    out.push((name.to_owned(), value.field_text()));
}

/// Field parsers, one per reference `INI::parse*` function.
pub(crate) mod parse {
    use super::*;

    type Ctx<'a> = ModuleParseContext<'a>;
    type Done = Result<(), IniError>;

    macro_rules! forward {
        ($($name:ident: $ty:ty => $method:ident;)*) => {$(
            pub(crate) fn $name(v: &mut $ty, r: &mut IniReader<'_>, _: &Ctx<'_>) -> Done {
                r.$method(v)
            }
        )*};
    }

    forward! {
        real: Real => parse_real;
        percent: Real => parse_percent_to_real;
        angle: Real => parse_angle_real;
        angular_velocity: Real => parse_angular_velocity_real;
        velocity: Real => parse_velocity_real;
        duration_real: Real => parse_duration_real;
        duration: UnsignedInt => parse_duration_unsigned_int;
        int: Int => parse_int;
        uint: UnsignedInt => parse_unsigned_int;
        boolean: Bool => parse_bool;
        ascii: String => parse_ascii_string;
        string_vec_append: Vec<String> => parse_ascii_string_vector_append;
        coord3d: Coord3D => parse_coord3d;
        rgb: RGBColor => parse_rgb_color;
    }

    /// `parseDurationUnsignedInt` on an `Int` member.
    pub(crate) fn duration_int(v: &mut Int, r: &mut IniReader<'_>, _: &Ctx<'_>) -> Done {
        let mut frames: UnsignedInt = 0;
        r.parse_duration_unsigned_int(&mut frames)?;
        *v = frames as Int;
        Ok(())
    }

    /// `parseUnsignedInt` on an `Int` member.
    pub(crate) fn uint_int(v: &mut Int, r: &mut IniReader<'_>, _: &Ctx<'_>) -> Done {
        let mut value: UnsignedInt = 0;
        r.parse_unsigned_int(&mut value)?;
        *v = value as Int;
        Ok(())
    }

    /// `parseAudioEventRTS`: `NoSound` leaves the member alone; the event name is kept.
    pub(crate) fn audio(v: &mut String, r: &mut IniReader<'_>, _: &Ctx<'_>) -> Done {
        let token = r.next_token(SEPS_DEFAULT)?;
        if !token.eq_ignore_ascii_case("NoSound") {
            *v = token.to_owned();
        }
        Ok(())
    }

    /// A `findXxx` that matches names exactly and leaves null for anything else.
    pub(crate) fn lookup(
        token: &str,
        known: &std::collections::BTreeSet<String>,
    ) -> Option<String> {
        known.get(token).cloned()
    }

    pub(crate) fn fx_list(v: &mut Option<String>, r: &mut IniReader<'_>, c: &Ctx<'_>) -> Done {
        *v = lookup(r.next_token(SEPS_DEFAULT)?, c.fx_list_names);
        Ok(())
    }

    pub(crate) fn ocl(v: &mut Option<String>, r: &mut IniReader<'_>, c: &Ctx<'_>) -> Done {
        *v = lookup(r.next_token(SEPS_DEFAULT)?, c.ocl_names);
        Ok(())
    }

    pub(crate) fn particle_system(
        v: &mut Option<String>,
        r: &mut IniReader<'_>,
        c: &Ctx<'_>,
    ) -> Done {
        *v = lookup(r.next_token(SEPS_DEFAULT)?, c.particle_system_names);
        Ok(())
    }

    pub(crate) fn weapon(v: &mut Option<String>, r: &mut IniReader<'_>, c: &Ctx<'_>) -> Done {
        let token = r.next_token(SEPS_DEFAULT)?;
        *v = if token.eq_ignore_ascii_case("None") {
            None
        } else {
            c.weapon_store
                .find_weapon_template(token)
                .map(|w| w.name.clone())
        };
        Ok(())
    }

    pub(crate) fn special_power(
        v: &mut Option<String>,
        r: &mut IniReader<'_>,
        c: &Ctx<'_>,
    ) -> Done {
        let token = r.next_token(SEPS_DEFAULT)?;
        *v = c
            .special_power_store
            .find_special_power_template(token)
            .map(|_| token.to_owned());
        Ok(())
    }

    pub(crate) fn kind_of(v: &mut KindOfMaskType, r: &mut IniReader<'_>, _: &Ctx<'_>) -> Done {
        v.parse_from_ini(r, &KINDOF_NAMES)
    }

    /// `parseLookupList` with `TheWeaponSlotTypeNamesLookupList`.
    pub(crate) fn weapon_slot(v: &mut Int, r: &mut IniReader<'_>, _: &Ctx<'_>) -> Done {
        r.parse_index_list(v, &["PRIMARY", "SECONDARY", "TERTIARY"])
    }

    /// `parseIndexList` with `TheLocomotorSetNames`.
    pub(crate) fn locomotor_set(v: &mut Int, r: &mut IniReader<'_>, _: &Ctx<'_>) -> Done {
        r.parse_index_list(v, &LOCOMOTOR_SET_NAMES)
    }
}

/// Fields a data struct takes from a base class or a mux.
pub trait DataPart {
    /// `None` when `field` is not one of this part's.
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>>;

    /// `gravity` is for the two members the reference derives from it.
    fn part_fields(&self, gravity: Real, out: &mut FieldList);
}

/// `AIUpdateModuleData`. It rejects what it does not know, so it is the last part.
impl DataPart for AiUpdateModuleData {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        _: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        Some(self.parse_field(field, reader))
    }

    fn part_fields(&self, _: Real, out: &mut FieldList) {
        for (name, turret) in [("Turret", 0), ("AltTurret", 1)] {
            push(out, name, &Bool::from(self.turrets[turret].is_some()));
        }
        push(
            out,
            "AutoAcquireEnemiesWhenIdle",
            &self.auto_acquire_enemies_when_idle,
        );
        push(out, "MoodAttackCheckRate", &self.mood_attack_check_rate);
        push(out, "ForbidPlayerCommands", &self.forbid_player_commands);
        push(out, "TurretsLinked", &self.turrets_linked);
    }
}

/// `DockUpdateModuleData`; the last part, like the AI one.
impl DataPart for DockUpdateModuleData {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        _: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        Some(self.parse_field(field, reader))
    }

    fn part_fields(&self, _: Real, out: &mut FieldList) {
        push(
            out,
            "NumberApproachPositions",
            &self.number_approach_positions,
        );
        push(out, "AllowsPassthrough", &self.allows_passthrough);
    }
}

/// `DynamicGeometryInfoUpdateModuleData`; the last part.
impl DataPart for DynamicGeometryInfoUpdateModuleData {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        _: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        Some(self.parse_field(field, reader))
    }

    fn part_fields(&self, _: Real, out: &mut FieldList) {
        push(out, "InitialDelay", &self.initial_delay);
        push(out, "InitialHeight", &self.initial_height);
        push(out, "InitialMajorRadius", &self.initial_major_radius);
        push(out, "InitialMinorRadius", &self.initial_minor_radius);
        push(out, "FinalHeight", &self.final_height);
        push(out, "FinalMajorRadius", &self.final_major_radius);
        push(out, "FinalMinorRadius", &self.final_minor_radius);
        push(out, "TransitionTime", &self.transition_time);
        push(
            out,
            "ReverseAtTransitionTime",
            &self.reverse_at_transition_time,
        );
    }
}

/// `PhysicsBehaviorModuleData`; the last part.
impl DataPart for PhysicsBehaviorModuleData {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        Some(self.parse_field(field, reader, ctx))
    }

    fn part_fields(&self, gravity: Real, out: &mut FieldList) {
        push(out, "Mass", &self.mass);
        push(out, "ShockResistance", &self.shock_resistance);
        push(out, "ShockMaxYaw", &self.shock_max_yaw);
        push(out, "ShockMaxPitch", &self.shock_max_pitch);
        push(out, "ShockMaxRoll", &self.shock_max_roll);
        push(out, "ForwardFriction", &self.forward_friction);
        push(out, "LateralFriction", &self.lateral_friction);
        push(out, "ZFriction", &self.z_friction);
        push(out, "AerodynamicFriction", &self.aerodynamic_friction);
        push(out, "CenterOfMassOffset", &self.center_of_mass_offset);
        push(out, "AllowBouncing", &self.allow_bouncing);
        push(out, "AllowCollideForce", &self.allow_collide_force);
        push(
            out,
            "KillWhenRestingOnGround",
            &self.kill_when_resting_on_ground,
        );
        push(
            out,
            "MinFallHeightForDamage",
            &height_to_speed(gravity, self.min_fall_height_for_damage),
        );
        push(
            out,
            "FallHeightDamageFactor",
            &self.fall_height_damage_factor,
        );
        push(out, "PitchRollYawFactor", &self.pitch_roll_yaw_factor);
        push(
            out,
            "VehicleCrashesIntoBuildingWeaponTemplate",
            &self.crash_into_building_weapon,
        );
        push(
            out,
            "VehicleCrashesIntoNonBuildingWeaponTemplate",
            &self.crash_into_non_building_weapon,
        );
    }
}

impl DataPart for DieMuxData {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        _: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        self.parse_field(field, reader)
    }

    fn part_fields(&self, _: Real, out: &mut FieldList) {
        push(out, "DeathTypes", &self.death_types);
        // The reference keeps level `n` in bit `n - 1`, the port in bit `n`: compare
        // the levels the flags hold.
        use VeterancyLevel::{Elite, Heroic, Regular, Veteran};
        let levels: String = [Regular, Veteran, Elite, Heroic]
            .map(|level| {
                if get_veterancy_level_flag(self.veterancy_levels, level) != 0 {
                    '1'
                } else {
                    '0'
                }
            })
            .iter()
            .collect();
        push(out, "VeterancyLevels", &levels);
        push(out, "ExemptStatus", &self.exempt_status);
        push(out, "RequiredStatus", &self.required_status);
    }
}

impl DataPart for UpgradeMuxData {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        self.parse_field(field, reader, ctx)
    }

    fn part_fields(&self, _: Real, out: &mut FieldList) {
        push(out, "TriggeredBy", &self.activation_upgrade_names);
        push(out, "ConflictsWith", &self.conflicting_upgrade_names);
        push(out, "RemovesUpgrades", &self.removal_upgrade_names);
        push(out, "FXListUpgrade", &self.fx_list_upgrade);
        push(out, "RequiresAllTriggers", &self.requires_all_triggers);
    }
}

impl DataPart for OpenContainModuleData {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        self.parse_field(field, reader, ctx)
    }

    fn part_fields(&self, gravity: Real, out: &mut FieldList) {
        push(out, "ContainMax", &self.contain_max);
        push(out, "EnterSound", &self.enter_sound);
        push(out, "ExitSound", &self.exit_sound);
        push(
            out,
            "DamagePercentToUnits",
            &self.damage_percentage_to_units,
        );
        push(out, "BurnedDeathToUnits", &self.is_burned_death_to_units);
        push(out, "AllowInsideKindOf", &self.allow_inside_kind_of);
        push(out, "ForbidInsideKindOf", &self.forbid_inside_kind_of);
        push(
            out,
            "PassengersAllowedToFire",
            &self.passengers_allowed_to_fire,
        );
        push(out, "PassengersInTurret", &self.passengers_in_turret);
        push(out, "NumberOfExitPaths", &self.number_of_exit_paths);
        push(out, "DoorOpenTime", &self.door_open_time);
        push(
            out,
            "WeaponBonusPassedToPassengers",
            &self.weapon_bonus_passed_to_passengers,
        );
        push(out, "AllowAlliesInside", &self.allow_allies_inside);
        push(out, "AllowEnemiesInside", &self.allow_enemies_inside);
        push(out, "AllowNeutralInside", &self.allow_neutral_inside);
        self.die_mux.part_fields(gravity, out);
    }
}

impl DataPart for TransportContainModuleData {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        self.parse_field(field, reader, ctx)
    }

    fn part_fields(&self, gravity: Real, out: &mut FieldList) {
        push(out, "Slots", &self.slot_capacity);
        push(out, "ScatterNearbyOnExit", &self.scatter_nearby_on_exit);
        push(
            out,
            "OrientLikeContainerOnExit",
            &self.orient_like_container_on_exit,
        );
        push(
            out,
            "KeepContainerVelocityOnExit",
            &self.keep_container_velocity_on_exit,
        );
        push(out, "GoAggressiveOnExit", &self.go_aggressive_on_exit);
        push(
            out,
            "ResetMoodCheckTimeOnExit",
            &self.reset_mood_check_time_on_exit,
        );
        push(
            out,
            "DestroyRidersWhoAreNotFreeToExit",
            &self.destroy_riders_who_are_not_free_to_exit,
        );
        push(out, "ExitBone", &self.exit_bone);
        push(out, "ExitPitchRate", &self.exit_pitch_rate);
        let payload = &self.initial_payload;
        push(
            out,
            "InitialPayload",
            &format!("{}|{}", payload.name, payload.count),
        );
        push(out, "HealthRegen%PerSec", &self.health_regen);
        push(out, "ExitDelay", &self.exit_delay);
        push(
            out,
            "ArmedRidersUpgradeMyWeaponSet",
            &self.armed_riders_upgrade_weapon_set,
        );
        push(out, "DelayExitInAir", &self.is_delay_exit_in_air);
        self.open.part_fields(gravity, out);
    }
}

impl DataPart for SlowDeathBehaviorModuleData {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        self.parse_field(field, reader, ctx)
    }

    fn part_fields(&self, gravity: Real, out: &mut FieldList) {
        push(out, "SinkRate", &self.sink_rate);
        push(out, "ProbabilityModifier", &self.probability_modifier);
        push(
            out,
            "ModifierBonusPerOverkillPercent",
            &self.modifier_bonus_per_overkill_percent,
        );
        push(out, "SinkDelay", &self.sink_delay);
        push(out, "SinkDelayVariance", &self.sink_delay_variance);
        push(out, "DestructionDelay", &self.destruction_delay);
        push(
            out,
            "DestructionDelayVariance",
            &self.destruction_delay_variance,
        );
        push(out, "DestructionAltitude", &self.destruction_altitude);
        for (name, lists) in [
            ("FX", &self.fx),
            ("OCL", &self.ocls),
            ("Weapon", &self.weapons),
        ] {
            let text = lists.each_ref().map(FieldText::field_text).join(";");
            push(out, name, &text);
        }
        push(out, "FlingForce", &self.fling_force);
        push(out, "FlingForceVariance", &self.fling_force_variance);
        push(out, "FlingPitch", &self.fling_pitch);
        push(out, "FlingPitchVariance", &self.fling_pitch_variance);
        self.die_mux.part_fields(gravity, out);
    }
}

/// `RadiusDecalTemplate`.
#[derive(Debug, Clone, PartialEq)]
pub struct RadiusDecalTemplate {
    pub texture: String,
    /// `Style`, bits of `TheShadowNames`.
    pub shadow_type: UnsignedInt,
    pub min_opacity: Real,
    pub max_opacity: Real,
    pub opacity_throb_time: UnsignedInt,
    pub color: zh_common::ini::Color,
    pub only_visible_to_owning_player: Bool,
}

impl Default for RadiusDecalTemplate {
    fn default() -> Self {
        Self {
            texture: String::new(),
            // `SHADOW_ALPHA_DECAL`.
            shadow_type: 0x20,
            min_opacity: 1.0,
            max_opacity: 1.0,
            opacity_throb_time: 30,
            color: 0,
            only_visible_to_owning_player: 1,
        }
    }
}

impl FieldText for RadiusDecalTemplate {
    fn field_text(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}|{}",
            self.texture,
            self.shadow_type,
            self.min_opacity.field_text(),
            self.max_opacity.field_text(),
            self.opacity_throb_time,
            self.color,
            self.only_visible_to_owning_player
        )
    }
}

/// `RadiusDecalTemplate::parseRadiusDecalTemplate`: a nested block, parsed into the
/// template already there.
pub(crate) fn radius_decal(
    v: &mut RadiusDecalTemplate,
    r: &mut IniReader<'_>,
    _: &ModuleParseContext<'_>,
) -> Result<(), IniError> {
    const SHADOW_NAMES: [&str; 7] = [
        "SHADOW_DECAL",
        "SHADOW_VOLUME",
        "SHADOW_PROJECTION",
        "SHADOW_DYNAMIC_PROJECTION",
        "SHADOW_DIRECTIONAL_PROJECTION",
        "SHADOW_ALPHA_DECAL",
        "SHADOW_ADDITIVE_DECAL",
    ];
    r.read_block_fields(|field, r| match field {
        "Texture" => r.parse_ascii_string(&mut v.texture),
        "Style" => r.parse_bit_string32(&mut v.shadow_type, &SHADOW_NAMES),
        "OpacityMin" => r.parse_percent_to_real(&mut v.min_opacity),
        "OpacityMax" => r.parse_percent_to_real(&mut v.max_opacity),
        "OpacityThrobTime" => r.parse_duration_unsigned_int(&mut v.opacity_throb_time),
        "Color" => r.parse_color_int(&mut v.color),
        "OnlyVisibleToOwningPlayer" => r.parse_bool(&mut v.only_visible_to_owning_player),
        _ => Err(IniError::UnknownToken),
    })
}

/// A data struct of this family.
pub trait DataOnly: ModuleData + Sized {
    const MODULE: &'static str;
    fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self;
    fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>>;
    fn fields(&self, gravity: Real, out: &mut FieldList);
}

const fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// The `ModuleFactory::init` entry of `name`; an unknown name fails to compile when
/// used in a constant.
pub(crate) const fn builtin(name: &str) -> &'static BuiltinModuleEntry {
    let mut i = 0;
    while i < BUILTIN_ZH_MODULES.len() {
        if same(BUILTIN_ZH_MODULES[i].name, name) {
            return &BUILTIN_ZH_MODULES[i];
        }
        i += 1;
    }
    panic!("not a ModuleFactory module")
}

/// Declares a `ModuleData` struct and its field table. Each `part` is a base class or
/// mux, tried in order after the struct's own fields; each field line is
/// `"IniName" => member: Type = default, parser;`.
macro_rules! module_data {
    (
        $(#[$meta:meta])*
        $name:ident = $module:literal;
        $( part $part:ident: $part_ty:ty = $part_new:expr; )*
        $( $(#[$field_meta:meta])* $ini:literal => $field:ident: $ty:ty = $default:expr, $parse:expr; )*
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $name {
            pub tag_key: zh_common::types::NameKeyType,
            pub module_name_key: zh_common::types::NameKeyType,
            $( pub $part: $part_ty, )*
            $( $(#[$field_meta])* pub $field: $ty, )*
        }

        impl $crate::modules::module_data::DataOnly for $name {
            const MODULE: &'static str = $module;

            fn new(
                tag_key: zh_common::types::NameKeyType,
                module_name_key: zh_common::types::NameKeyType,
            ) -> Self {
                Self {
                    tag_key,
                    module_name_key,
                    $( $part: ($part_new)(tag_key, module_name_key), )*
                    $( $field: $default, )*
                }
            }

            #[allow(unused_variables)]
            fn parse_field(
                &mut self,
                field: &str,
                reader: &mut zh_common::ini::IniReader<'_>,
                ctx: &$crate::modules::module_factory::ModuleParseContext<'_>,
            ) -> Option<Result<(), zh_common::ini::IniError>> {
                match field {
                    $( $ini => return Some(($parse)(&mut self.$field, reader, ctx)), )*
                    _ => {}
                }
                $(
                    if let Some(result) = $crate::modules::module_data::DataPart::parse_part(
                        &mut self.$part, field, reader, ctx,
                    ) {
                        return Some(result);
                    }
                )*
                None
            }

            #[allow(unused_variables)]
            fn fields(
                &self,
                gravity: zh_common::types::Real,
                out: &mut $crate::modules::module_data::FieldList,
            ) {
                $(
                    out.push((
                        $ini.to_owned(),
                        $crate::modules::module_data::FieldText::field_text_at(&self.$field, gravity),
                    ));
                )*
                $( $crate::modules::module_data::DataPart::part_fields(&self.$part, gravity, out); )*
            }
        }

        impl $crate::modules::module::ModuleData for $name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn tag_key(&self) -> zh_common::types::NameKeyType {
                self.tag_key
            }

            fn module_name_key(&self) -> zh_common::types::NameKeyType {
                self.module_name_key
            }

            fn facet_mask(&self) -> $crate::arena::FacetMask {
                const { $crate::modules::module_data::builtin($module) }.facet_mask
            }

            fn update_phase(&self) -> $crate::scheduler::SleepyUpdatePhase {
                const { $crate::modules::module_data::builtin($module) }.update_phase
            }

            fn is_ai_module(&self) -> bool {
                const { $crate::modules::module_data::builtin($module) }.is_ai_module
            }
        }
    };
}
pub(crate) use module_data;

/// `ModuleFactory::newModuleDataFromINI` for one of these.
pub(crate) fn parse_data<T: DataOnly + 'static>(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = T::new(tag_key, module_name_key);
    // The base classes' parsers ignore case; the reference's lookup does not.
    let mut table = FieldList::new();
    data.fields(0.0, &mut table);
    reader.read_block_fields(|field, r| {
        if !table.iter().any(|(name, _)| name == field) {
            return Err(IniError::UnknownToken);
        }
        data.parse_field(field, r, ctx)
            .unwrap_or(Err(IniError::UnknownToken))
    })?;
    Ok(Box::new(data))
}

fn fields_of<T: DataOnly + 'static>(data: &dyn ModuleData, gravity: Real) -> Option<FieldList> {
    let typed = data.as_any().downcast_ref::<T>()?;
    let mut out = FieldList::new();
    typed.fields(gravity, &mut out);
    Some(out)
}

/// One module that has data and no behaviour.
#[derive(Clone, Copy)]
pub struct DataOnlyModule {
    pub name: &'static str,
    pub parse: ModuleDataParseFn,
    /// The members of data made by `parse`.
    pub fields: fn(&dyn ModuleData, Real) -> Option<FieldList>,
}

const fn of<T: DataOnly + 'static>() -> DataOnlyModule {
    DataOnlyModule {
        name: T::MODULE,
        parse: parse_data::<T>,
        fields: fields_of::<T>,
    }
}

use crate::modules::behavior::bridge_behavior::BridgeBehaviorModuleData;
use crate::modules::behavior::countermeasures_behavior::CountermeasuresBehaviorModuleData;
use crate::modules::behavior::flight_deck_behavior::FlightDeckBehaviorModuleData;
use crate::modules::behavior::rebuild_hole_behavior::RebuildHoleBehaviorModuleData;
use crate::modules::contain::overlord_contain::OverlordContainModuleData;
use crate::modules::contain::rider_change_contain::RiderChangeContainModuleData;
use crate::modules::update::{
    animation_steering_update::*, assault_transport_ai_update::*, bone_fx_update::*,
    checkpoint_update::*, chinook_ai_update::*, cleanup_hazard_update::*,
    deliver_payload_ai_update::*, demo_trap_update::*, deploy_style_ai_update::*,
    dozer_ai_update::*, emp_update::*, enemy_near_update::*,
    fire_ocl_after_weapon_cooldown_update::*, firestorm_dynamic_geometry_info_update::*,
    hack_internet_ai_update::*, hijacker_update::*, jet_ai_update::*, missile_ai_update::*,
    mob_member_slaved_update::*, neutron_missile_slow_death_update::*, neutron_missile_update::*,
    particle_uplink_cannon_update::*, pilot_find_vehicle_update::*, railed_transport_ai_update::*,
    railed_transport_dock_update::*, railroad_guide_ai_update::*, spectre_gunship_update::*,
    sticky_bomb_update::*, structure_collapse_update::*, supply_truck_ai_update::*,
    transport_ai_update::*, wander_ai_update::*, wave_guide_update::*, worker_ai_update::*,
};

/// Every logic module the retail data declares that had no ported behaviour when its
/// data parser was written. `BridgeBehavior` has been ported since and keeps its
/// parser here.
pub const DATA_ONLY_MODULES: [DataOnlyModule; 41] = [
    of::<AnimationSteeringUpdateModuleData>(),
    of::<AssaultTransportAIUpdateModuleData>(),
    of::<BoneFXUpdateModuleData>(),
    of::<BridgeBehaviorModuleData>(),
    of::<CheckpointUpdateModuleData>(),
    of::<ChinookAIUpdateModuleData>(),
    of::<CleanupHazardUpdateModuleData>(),
    of::<CountermeasuresBehaviorModuleData>(),
    of::<DeliverPayloadAIUpdateModuleData>(),
    of::<DemoTrapUpdateModuleData>(),
    of::<DeployStyleAIUpdateModuleData>(),
    of::<DozerAIUpdateModuleData>(),
    of::<EMPUpdateModuleData>(),
    of::<EnemyNearUpdateModuleData>(),
    of::<FireOCLAfterWeaponCooldownUpdateModuleData>(),
    of::<FirestormDynamicGeometryInfoUpdateModuleData>(),
    of::<FlightDeckBehaviorModuleData>(),
    of::<HackInternetAIUpdateModuleData>(),
    of::<HijackerUpdateModuleData>(),
    of::<JetAIUpdateModuleData>(),
    of::<LeafletDropBehaviorModuleData>(),
    of::<MissileAIUpdateModuleData>(),
    of::<MobMemberSlavedUpdateModuleData>(),
    of::<NeutronMissileSlowDeathBehaviorModuleData>(),
    of::<NeutronMissileUpdateModuleData>(),
    of::<OverlordContainModuleData>(),
    of::<ParticleUplinkCannonUpdateModuleData>(),
    of::<PilotFindVehicleUpdateModuleData>(),
    of::<RailedTransportAIUpdateModuleData>(),
    of::<RailedTransportDockUpdateModuleData>(),
    of::<RailroadBehaviorModuleData>(),
    of::<RebuildHoleBehaviorModuleData>(),
    of::<RiderChangeContainModuleData>(),
    of::<SpectreGunshipUpdateModuleData>(),
    of::<StickyBombUpdateModuleData>(),
    of::<StructureCollapseUpdateModuleData>(),
    of::<SupplyTruckAIUpdateModuleData>(),
    of::<TransportAIUpdateModuleData>(),
    of::<WanderAIUpdateModuleData>(),
    of::<WaveGuideUpdateModuleData>(),
    of::<WorkerAIUpdateModuleData>(),
];

/// Gives the built-in stubs of [`DATA_ONLY_MODULES`] their data parsers. A module that
/// already has one keeps it.
pub fn register_data_only_modules(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    for module in DATA_ONLY_MODULES {
        let entry = builtin(module.name);
        if reg
            .find_descriptor(entry.name, entry.module_type)
            .is_some_and(|d| d.parse_data.is_some())
        {
            continue;
        }
        reg.register_descriptor(
            nk,
            entry.name,
            entry.module_type,
            entry.facet_mask,
            entry.update_phase,
            entry.is_ai_module,
            Some(module.parse),
            None,
        );
    }
}
