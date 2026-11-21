//! `PKT-134`: Special power template definitions and execution gate queries
//! (`GeneralsMD/Code/GameEngine/Include/Common/SpecialPower.h` &
//! `GeneralsMD/Code/GameEngine/Source/Common/RTS/SpecialPower.cpp`).
//!
//! Reference C++ packet: `WP-3.2b` (`docs/golden_vectors/WP3_2b_TechTree.json`)
//! Golden Vector test: `crates/zh_logic/tests/golden_wp3_2b_tech_tree.rs`

use crate::rules::science::{ScienceStore, ScienceType, SCIENCE_INVALID};
use crate::rules::special_power_types::{SpecialPowerType, SPECIAL_POWER_NAMES};
use crate::rules::{AcademyClassificationType, ACADEMY_CLASSIFICATION_TYPE_NAMES};
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{
    FieldParseEntry, FieldUserData, IniError, IniLoadType, IniReader, LOGICFRAMES_PER_SECOND,
    SEPS_DEFAULT,
};
use zh_common::types::{Bool, Int, Real, UnsignedInt, FALSE, TRUE};

/// Default defection detection time limit (`10` seconds at `30` FPS = `300` frames).
pub const DEFAULT_DEFECTION_DETECTION_PROTECTION_TIME_LIMIT: UnsignedInt =
    (LOGICFRAMES_PER_SECOND * 10) as UnsignedInt;

/// Callback interface replacing `Object*` queries in `SpecialPowerStore::canUseSpecialPower`.
pub trait SpecialPowerObjectAccess {
    fn has_special_power_module(&self, template: &SpecialPowerTemplate) -> bool;
    fn controlling_player_has_science(&self, science: ScienceType) -> bool;
}

/// Data fields for one override layer of `SpecialPowerTemplate`.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecialPowerTemplateData {
    pub name: String,
    pub id: UnsignedInt,
    pub power_type: SpecialPowerType,
    pub reload_time: UnsignedInt,
    pub required_science: ScienceType,
    pub initiate_sound: String,
    pub initiate_at_location_sound: String,
    pub academy_classification_type: AcademyClassificationType,
    pub detection_time: UnsignedInt,
    pub view_object_duration: UnsignedInt,
    pub view_object_range: Real,
    pub radius_cursor_radius: Real,
    pub public_timer: Bool,
    pub shared_n_sync: Bool,
    pub shortcut_power: Bool,
}

impl Default for SpecialPowerTemplateData {
    fn default() -> Self {
        Self {
            name: String::new(),
            id: 0,
            power_type: SpecialPowerType::Invalid,
            reload_time: 0,
            required_science: SCIENCE_INVALID,
            initiate_sound: String::new(),
            initiate_at_location_sound: String::new(),
            academy_classification_type: AcademyClassificationType::None,
            detection_time: DEFAULT_DEFECTION_DETECTION_PROTECTION_TIME_LIMIT,
            view_object_duration: 0,
            view_object_range: 0.0_f32,
            radius_cursor_radius: 0.0_f32,
            public_timer: FALSE,
            shared_n_sync: FALSE,
            shortcut_power: FALSE,
        }
    }
}

/// Special power template supporting `Overridable` chains (`map.ini` overrides).
#[derive(Debug, Clone)]
pub struct SpecialPowerTemplate {
    base: SpecialPowerTemplateData,
    base_is_override: bool,
    overrides: Vec<SpecialPowerTemplateData>,
}

impl SpecialPowerTemplate {
    fn final_override(&self) -> &SpecialPowerTemplateData {
        self.overrides.last().unwrap_or(&self.base)
    }

    fn delete_overrides(&mut self) -> bool {
        if self.base_is_override {
            false
        } else {
            self.overrides.clear();
            true
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.final_override().name
    }

    #[inline]
    pub fn id(&self) -> UnsignedInt {
        self.final_override().id
    }

    #[inline]
    pub fn special_power_type(&self) -> SpecialPowerType {
        self.final_override().power_type
    }

    #[inline]
    pub fn reload_time(&self) -> UnsignedInt {
        self.final_override().reload_time
    }

    #[inline]
    pub fn required_science(&self) -> ScienceType {
        self.final_override().required_science
    }

    #[inline]
    pub fn initiate_sound(&self) -> &str {
        &self.final_override().initiate_sound
    }

    #[inline]
    pub fn initiate_at_target_sound(&self) -> &str {
        &self.final_override().initiate_at_location_sound
    }

    #[inline]
    pub fn has_public_timer(&self) -> Bool {
        self.final_override().public_timer
    }

    #[inline]
    pub fn is_shared_n_sync(&self) -> Bool {
        self.final_override().shared_n_sync
    }

    #[inline]
    pub fn detection_time(&self) -> UnsignedInt {
        self.final_override().detection_time
    }

    #[inline]
    pub fn view_object_duration(&self) -> UnsignedInt {
        self.final_override().view_object_duration
    }

    #[inline]
    pub fn view_object_range(&self) -> Real {
        self.final_override().view_object_range
    }

    #[inline]
    pub fn radius_cursor_radius(&self) -> Real {
        self.final_override().radius_cursor_radius
    }

    #[inline]
    pub fn is_shortcut_power(&self) -> Bool {
        self.final_override().shortcut_power
    }

    /// Reads `m_academyClassificationType` from the base template without traversing
    /// `getFO()`, matching `SpecialPower.h:124`.
    #[inline]
    pub const fn academy_classification_type(&self) -> AcademyClassificationType {
        self.base.academy_classification_type
    }
}

#[derive(Debug, Clone)]
enum SpecialPowerParseOp {
    ReloadTime(UnsignedInt),
    RequiredScience(String),
    InitiateSound(String),
    InitiateAtLocationSound(String),
    PublicTimer(Bool),
    PowerType(SpecialPowerType),
    DetectionTime(UnsignedInt),
    SharedSyncedTimer(Bool),
    ViewObjectDuration(UnsignedInt),
    ViewObjectRange(Real),
    RadiusCursorRadius(Real),
    ShortcutPower(Bool),
    AcademyClassify(AcademyClassificationType),
}

#[derive(Debug, Default)]
struct SpecialPowerParseTarget {
    ops: Vec<SpecialPowerParseOp>,
}

fn sp_parse_reload_time(
    ini: &mut IniReader<'_>,
    target: &mut SpecialPowerParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut val = 0u32;
    ini.parse_duration_unsigned_int(&mut val)?;
    target.ops.push(SpecialPowerParseOp::ReloadTime(val));
    Ok(())
}

fn sp_parse_required_science(
    ini: &mut IniReader<'_>,
    target: &mut SpecialPowerParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let tok = ini.next_token(SEPS_DEFAULT)?.to_owned();
    target.ops.push(SpecialPowerParseOp::RequiredScience(tok));
    Ok(())
}

fn sp_parse_initiate_sound(
    ini: &mut IniReader<'_>,
    target: &mut SpecialPowerParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let tok = ini.next_token(SEPS_DEFAULT)?.to_owned();
    target.ops.push(SpecialPowerParseOp::InitiateSound(tok));
    Ok(())
}

fn sp_parse_initiate_at_loc_sound(
    ini: &mut IniReader<'_>,
    target: &mut SpecialPowerParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let tok = ini.next_token(SEPS_DEFAULT)?.to_owned();
    target
        .ops
        .push(SpecialPowerParseOp::InitiateAtLocationSound(tok));
    Ok(())
}

fn sp_parse_public_timer(
    ini: &mut IniReader<'_>,
    target: &mut SpecialPowerParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut b = FALSE;
    ini.parse_bool(&mut b)?;
    target.ops.push(SpecialPowerParseOp::PublicTimer(b));
    Ok(())
}

fn sp_parse_enum(
    ini: &mut IniReader<'_>,
    target: &mut SpecialPowerParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut raw = 0;
    ini.parse_index_list(&mut raw, &SPECIAL_POWER_NAMES)?;
    target
        .ops
        .push(SpecialPowerParseOp::PowerType(SpecialPowerType::from_i32(raw)));
    Ok(())
}

fn sp_parse_detection_time(
    ini: &mut IniReader<'_>,
    target: &mut SpecialPowerParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut val = 0u32;
    ini.parse_duration_unsigned_int(&mut val)?;
    target.ops.push(SpecialPowerParseOp::DetectionTime(val));
    Ok(())
}

fn sp_parse_shared_synced_timer(
    ini: &mut IniReader<'_>,
    target: &mut SpecialPowerParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut b = FALSE;
    ini.parse_bool(&mut b)?;
    target.ops.push(SpecialPowerParseOp::SharedSyncedTimer(b));
    Ok(())
}

fn sp_parse_view_object_duration(
    ini: &mut IniReader<'_>,
    target: &mut SpecialPowerParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut val = 0u32;
    ini.parse_duration_unsigned_int(&mut val)?;
    target.ops.push(SpecialPowerParseOp::ViewObjectDuration(val));
    Ok(())
}

fn sp_parse_view_object_range(
    ini: &mut IniReader<'_>,
    target: &mut SpecialPowerParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut val = 0.0_f32;
    ini.parse_real(&mut val)?;
    target.ops.push(SpecialPowerParseOp::ViewObjectRange(val));
    Ok(())
}

fn sp_parse_radius_cursor_radius(
    ini: &mut IniReader<'_>,
    target: &mut SpecialPowerParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut val = 0.0_f32;
    ini.parse_real(&mut val)?;
    target
        .ops
        .push(SpecialPowerParseOp::RadiusCursorRadius(val));
    Ok(())
}

fn sp_parse_shortcut_power(
    ini: &mut IniReader<'_>,
    target: &mut SpecialPowerParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut b = FALSE;
    ini.parse_bool(&mut b)?;
    target.ops.push(SpecialPowerParseOp::ShortcutPower(b));
    Ok(())
}

fn sp_parse_academy_classify(
    ini: &mut IniReader<'_>,
    target: &mut SpecialPowerParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut raw = 0;
    ini.parse_index_list(&mut raw, &ACADEMY_CLASSIFICATION_TYPE_NAMES)?;
    let act = AcademyClassificationType::from_i32(raw).ok_or(IniError::InvalidData)?;
    target.ops.push(SpecialPowerParseOp::AcademyClassify(act));
    Ok(())
}

static SPECIAL_POWER_FIELD_PARSE: &[FieldParseEntry<SpecialPowerParseTarget>] = &[
    FieldParseEntry {
        token: Some("ReloadTime"),
        parse: sp_parse_reload_time,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("RequiredScience"),
        parse: sp_parse_required_science,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("InitiateSound"),
        parse: sp_parse_initiate_sound,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("InitiateAtLocationSound"),
        parse: sp_parse_initiate_at_loc_sound,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("PublicTimer"),
        parse: sp_parse_public_timer,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("Enum"),
        parse: sp_parse_enum,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("DetectionTime"),
        parse: sp_parse_detection_time,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("SharedSyncedTimer"),
        parse: sp_parse_shared_synced_timer,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("ViewObjectDuration"),
        parse: sp_parse_view_object_duration,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("ViewObjectRange"),
        parse: sp_parse_view_object_range,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("RadiusCursorRadius"),
        parse: sp_parse_radius_cursor_radius,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("ShortcutPower"),
        parse: sp_parse_shortcut_power,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("AcademyClassify"),
        parse: sp_parse_academy_classify,
        user_data: FieldUserData::None,
    },
];

/// Global store of `SpecialPowerTemplate` definitions (`TheSpecialPowerStore`).
#[derive(Debug, Clone, Default)]
pub struct SpecialPowerStore {
    templates: Vec<SpecialPowerTemplate>,
    next_special_power_id: UnsignedInt,
}

impl SpecialPowerStore {
    /// Creates an empty `SpecialPowerStore`.
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
            next_special_power_id: 0,
        }
    }

    /// Subsystem initialization (`SpecialPowerStore::init`, no-op).
    pub fn init(&mut self) {}

    /// Subsystem update (`SpecialPowerStore::update`, no-op).
    pub fn update(&mut self) {}

    /// Removes override-only special powers and resets per-map overrides (`SpecialPowerStore::reset`).
    pub fn reset(&mut self) {
        self.templates.retain_mut(|sp| sp.delete_overrides());
    }

    /// Case-sensitive lookup by special power template name.
    pub fn find_special_power_template(&self, name: &str) -> Option<&SpecialPowerTemplate> {
        self.templates.iter().find(|t| t.name() == name)
    }

    /// Lookup by unique 1-based special power ID (`getID()`).
    pub fn find_special_power_template_by_id(&self, id: UnsignedInt) -> Option<&SpecialPowerTemplate> {
        self.templates.iter().find(|t| t.id() == id)
    }

    /// WorldBuilder index accessor (`0..get_num_special_powers()`).
    pub fn get_special_power_template_by_index(
        &self,
        index: UnsignedInt,
    ) -> Option<&SpecialPowerTemplate> {
        self.templates.get(index as usize)
    }

    /// Total number of templates currently registered.
    #[inline]
    pub fn get_num_special_powers(&self) -> Int {
        self.templates.len() as Int
    }

    /// Checks whether `obj` and its controlling player satisfy `special_power_template`.
    pub fn can_use_special_power(
        &self,
        obj: Option<&dyn SpecialPowerObjectAccess>,
        special_power_template: Option<&SpecialPowerTemplate>,
    ) -> Bool {
        let (Some(obj), Some(tmpl)) = (obj, special_power_template) else {
            return FALSE;
        };
        if !obj.has_special_power_module(tmpl) {
            return FALSE;
        }
        let req = tmpl.required_science();
        if req != SCIENCE_INVALID && !obj.controlling_player_has_science(req) {
            return FALSE;
        }
        TRUE
    }

    /// Parses one `SpecialPower <Name> ... End` INI block.
    pub fn parse_special_power_definition(
        &mut self,
        ini: &mut IniReader<'_>,
        science_store: &ScienceStore,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<(), IniError> {
        let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let existing_idx = self.templates.iter().position(|t| t.name() == name);
        let is_override = ini.get_load_type() == IniLoadType::CreateOverrides;

        if !is_override && existing_idx.is_some() {
            return Err(IniError::InvalidData);
        }

        let mut parsed = SpecialPowerParseTarget::default();
        ini.init_from_ini(&mut parsed, SPECIAL_POWER_FIELD_PARSE)?;

        let mut data = if is_override {
            if let Some(idx) = existing_idx {
                self.templates[idx].final_override().clone()
            } else {
                let mut base = self
                    .find_special_power_template("DefaultSpecialPower")
                    .map(|t| t.base.clone())
                    .unwrap_or_default();
                self.next_special_power_id = self.next_special_power_id.wrapping_add(1);
                base.name = name;
                base.id = self.next_special_power_id;
                base
            }
        } else {
            let mut base = self
                .find_special_power_template("DefaultSpecialPower")
                .map(|t| t.base.clone())
                .unwrap_or_default();
            self.next_special_power_id = self.next_special_power_id.wrapping_add(1);
            base.name = name;
            base.id = self.next_special_power_id;
            base
        };

        for op in parsed.ops {
            match op {
                SpecialPowerParseOp::ReloadTime(v) => data.reload_time = v,
                SpecialPowerParseOp::RequiredScience(tok) => {
                    data.required_science = science_store.friend_lookup_science(&tok, name_keys)?;
                }
                SpecialPowerParseOp::InitiateSound(tok) => {
                    if !tok.eq_ignore_ascii_case("NoSound") {
                        data.initiate_sound = tok;
                    }
                }
                SpecialPowerParseOp::InitiateAtLocationSound(tok) => {
                    if !tok.eq_ignore_ascii_case("NoSound") {
                        data.initiate_at_location_sound = tok;
                    }
                }
                SpecialPowerParseOp::PublicTimer(v) => data.public_timer = v,
                SpecialPowerParseOp::PowerType(v) => data.power_type = v,
                SpecialPowerParseOp::DetectionTime(v) => data.detection_time = v,
                SpecialPowerParseOp::SharedSyncedTimer(v) => data.shared_n_sync = v,
                SpecialPowerParseOp::ViewObjectDuration(v) => data.view_object_duration = v,
                SpecialPowerParseOp::ViewObjectRange(v) => data.view_object_range = v,
                SpecialPowerParseOp::RadiusCursorRadius(v) => data.radius_cursor_radius = v,
                SpecialPowerParseOp::ShortcutPower(v) => data.shortcut_power = v,
                SpecialPowerParseOp::AcademyClassify(v) => data.academy_classification_type = v,
            }
        }

        match (is_override, existing_idx) {
            (true, Some(idx)) => self.templates[idx].overrides.push(data),
            (true, None) => self.templates.push(SpecialPowerTemplate {
                base: data,
                base_is_override: true,
                overrides: Vec::new(),
            }),
            (false, None) => self.templates.push(SpecialPowerTemplate {
                base: data,
                base_is_override: false,
                overrides: Vec::new(),
            }),
            (false, Some(_)) => unreachable!("duplicate already rejected above"),
        }

        Ok(())
    }

    /// Loads all `SpecialPower` blocks from `ini`.
    pub fn load_from_ini(
        &mut self,
        ini: &mut IniReader<'_>,
        science_store: &ScienceStore,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<UnsignedInt, IniError> {
        ini.load_blocks(None, |block_name, reader| {
            if block_name.eq_ignore_ascii_case("SpecialPower") {
                let _block_kw = reader.next_token(SEPS_DEFAULT)?;
                self.parse_special_power_definition(reader, science_store, name_keys)?;
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        })
    }
}
