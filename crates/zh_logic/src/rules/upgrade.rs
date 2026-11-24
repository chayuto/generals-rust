//! `PKT-137`: Upgrade templates, runtime upgrade snapshots, and upgrade center
//! (`GeneralsMD/Code/GameEngine/Include/Common/Upgrade.h` &
//! `GeneralsMD/Code/GameEngine/Source/Common/System/Upgrade.cpp`).
//!
//! Reference C++ packet: `WP-3.2b` (`docs/golden_vectors/WP3_2b_TechTree.json`)
//! Golden Vector test: `crates/zh_logic/tests/golden_wp3_2b_tech_tree.rs`

use crate::rules::science::{SCIENCE_INVALID, ScienceType};
use crate::rules::{ACADEMY_CLASSIFICATION_TYPE_NAMES, AcademyClassificationType};
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{
    FieldParseEntry, FieldUserData, IniError, IniReader, LOGICFRAMES_PER_SECONDS_REAL, SEPS_DEFAULT,
    VETERANCY_NAMES,
};
use zh_common::types::{
    Bool, FALSE, Int, NameKeyType, Real, TRUE, UnsignedInt, VeterancyLevel, XferVersion,
};
use zh_common::xfer::bit_flags::BitFlags;
use zh_common::xfer::{Snapshot, Xfer, XferError};

/// Maximum upgrade templates tracked by `UpgradeMaskType` (`Upgrade.h`).
pub const UPGRADE_MAX_COUNT: usize = 512;

/// 512-bit upgrade ownership mask (`BitFlags<512>`).
pub type UpgradeMaskType = BitFlags<UPGRADE_MAX_COUNT>;

/// Runtime production status of an `Upgrade` instance (`UpgradeStatusType`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpgradeStatusType {
    #[default]
    Invalid = 0,
    InProduction = 1,
    Complete = 2,
}

impl UpgradeStatusType {
    /// Converts a raw 4-byte integer into `UpgradeStatusType`.
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Invalid),
            1 => Some(Self::InProduction),
            2 => Some(Self::Complete),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Scope of an `UpgradeTemplate` (`UpgradeType`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpgradeType {
    #[default]
    Player = 0,
    Object = 1,
    Count = 2,
}

impl UpgradeType {
    /// Converts a raw index into `UpgradeType`.
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Player),
            1 => Some(Self::Object),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// INI string names matching `TheUpgradeTypeNames[]` in `Upgrade.cpp`.
pub const UPGRADE_TYPE_NAMES: [&str; 2] = ["PLAYER", "OBJECT"];

/// Callback trait replacing `Player*` queries in `UpgradeTemplate` / `UpgradeCenter`.
pub trait UpgradePlayerAccess {
    fn money_count(&self) -> UnsignedInt;
    fn builds_instantly(&self) -> bool {
        false
    }
}

/// Single runtime `Upgrade` instance belonging to a `Player` or `Object` (`Snapshot`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Upgrade {
    template_name: String,
    status: UpgradeStatusType,
}

impl Upgrade {
    /// Constructs a new runtime `Upgrade` initialized to `UpgradeStatusType::Invalid`.
    pub fn new(template_name: impl Into<String>) -> Self {
        Self {
            template_name: template_name.into(),
            status: UpgradeStatusType::Invalid,
        }
    }

    #[inline]
    pub fn template_name(&self) -> &str {
        &self.template_name
    }

    #[inline]
    pub const fn status(&self) -> UpgradeStatusType {
        self.status
    }

    #[inline]
    pub fn set_status(&mut self, status: UpgradeStatusType) {
        self.status = status;
    }
}

impl Snapshot for Upgrade {
    fn crc(&self, _xfer: &mut dyn Xfer) {
        // Matches `Upgrade::crc` in Upgrade.cpp (`{ }`).
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut raw = self.status.as_i32().to_le_bytes();
        xfer.xfer_bytes(&mut raw)?;
        self.status = UpgradeStatusType::from_i32(Int::from_le_bytes(raw))
            .ok_or(XferError::InvalidData)?;
        Ok(())
    }

    fn load_post_process(&mut self) {}
}

/// Single `UpgradeTemplate` definition loaded from INI (`Upgrade.h`).
#[derive(Debug, Clone, PartialEq)]
pub struct UpgradeTemplate {
    upgrade_type: UpgradeType,
    name: String,
    name_key: NameKeyType,
    display_name_label: String,
    build_time: Real,
    cost: Int,
    upgrade_mask: UpgradeMaskType,
    research_sound: String,
    unit_specific_sound: String,
    academy_classification_type: AcademyClassificationType,
    button_image_name: String,
    button_image: Option<String>,
}

impl Default for UpgradeTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl UpgradeTemplate {
    /// Constructs an `UpgradeTemplate` with C++ constructor defaults (`Upgrade.cpp:132`).
    pub fn new() -> Self {
        Self {
            upgrade_type: UpgradeType::Player,
            name: String::new(),
            name_key: NameKeyType::INVALID,
            display_name_label: String::new(),
            build_time: 0.0_f32,
            cost: 0,
            upgrade_mask: UpgradeMaskType::new(),
            research_sound: String::new(),
            unit_specific_sound: String::new(),
            academy_classification_type: AcademyClassificationType::None,
            button_image_name: String::new(),
            button_image: None,
        }
    }

    /// Calculates build duration in 30 Hz logic frames (`UpgradeTemplate::calcTimeToBuild`).
    pub fn calc_time_to_build(&self, player: Option<&dyn UpgradePlayerAccess>) -> Int {
        let _ = player;
        // Release build omits RTS_DEBUG / _ALLOW_DEBUG_CHEATS_IN_RELEASE `buildsInstantly()`.
        (self.build_time * LOGICFRAMES_PER_SECONDS_REAL) as Int
    }

    /// Calculates resource cost for `player` (`UpgradeTemplate::calcCostToBuild`).
    pub fn calc_cost_to_build(&self, player: Option<&dyn UpgradePlayerAccess>) -> Int {
        let _ = player;
        self.cost
    }

    #[inline]
    pub fn set_upgrade_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    #[inline]
    pub fn upgrade_name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn set_upgrade_name_key(&mut self, key: NameKeyType) {
        self.name_key = key;
    }

    #[inline]
    pub const fn upgrade_name_key(&self) -> NameKeyType {
        self.name_key
    }

    #[inline]
    pub fn display_name_label(&self) -> &str {
        &self.display_name_label
    }

    #[inline]
    pub const fn upgrade_mask(&self) -> &UpgradeMaskType {
        &self.upgrade_mask
    }

    #[inline]
    pub const fn upgrade_type(&self) -> UpgradeType {
        self.upgrade_type
    }

    #[inline]
    pub fn research_complete_sound(&self) -> &str {
        &self.research_sound
    }

    #[inline]
    pub fn unit_specific_sound(&self) -> &str {
        &self.unit_specific_sound
    }

    #[inline]
    pub const fn academy_classification_type(&self) -> AcademyClassificationType {
        self.academy_classification_type
    }

    #[inline]
    pub fn button_image(&self) -> Option<&str> {
        self.button_image.as_deref()
    }

    /// Resolves button image handle once and clears `button_image_name` (`Upgrade.cpp:206`).
    pub fn cache_button_image(&mut self, mut find_image: impl FnMut(&str) -> bool) {
        if !self.button_image_name.is_empty() {
            if find_image(&self.button_image_name) {
                self.button_image = Some(self.button_image_name.clone());
            } else {
                self.button_image = None;
            }
            self.button_image_name.clear();
        }
    }

    /// Converts this template into a per-object veterancy upgrade (`Upgrade.cpp:192`).
    pub fn friend_make_veterancy_upgrade(
        &mut self,
        v: VeterancyLevel,
        name_keys: &mut NameKeyGenerator,
    ) {
        self.upgrade_type = UpgradeType::Object;
        self.name = get_vet_upgrade_name(v);
        self.name_key = name_keys.name_to_key(&self.name);
        self.display_name_label.clear();
        self.build_time = 0.0_f32;
        self.cost = 0;
    }
}

fn get_vet_upgrade_name(v: VeterancyLevel) -> String {
    let idx = v.as_i32() as usize;
    let suffix = VETERANCY_NAMES.get(idx).copied().unwrap_or("");
    format!("Upgrade_Veterancy_{suffix}")
}

fn parse_display_name(
    ini: &mut IniReader<'_>,
    target: &mut UpgradeTemplate,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_ascii_string(&mut target.display_name_label)
}

fn parse_upgrade_type(
    ini: &mut IniReader<'_>,
    target: &mut UpgradeTemplate,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut raw = 0;
    ini.parse_index_list(&mut raw, &UPGRADE_TYPE_NAMES)?;
    target.upgrade_type = UpgradeType::from_i32(raw).ok_or(IniError::InvalidData)?;
    Ok(())
}

fn parse_build_time(
    ini: &mut IniReader<'_>,
    target: &mut UpgradeTemplate,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_real(&mut target.build_time)
}

fn parse_build_cost(
    ini: &mut IniReader<'_>,
    target: &mut UpgradeTemplate,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_int(&mut target.cost)
}

fn parse_button_image(
    ini: &mut IniReader<'_>,
    target: &mut UpgradeTemplate,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_ascii_string(&mut target.button_image_name)
}

fn parse_audio_event(ini: &mut IniReader<'_>, store: &mut String) -> Result<(), IniError> {
    let tok = ini.next_token(SEPS_DEFAULT)?;
    if !tok.eq_ignore_ascii_case("NoSound") {
        *store = tok.to_owned();
    }
    Ok(())
}

fn parse_research_sound(
    ini: &mut IniReader<'_>,
    target: &mut UpgradeTemplate,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    parse_audio_event(ini, &mut target.research_sound)
}

fn parse_unit_specific_sound(
    ini: &mut IniReader<'_>,
    target: &mut UpgradeTemplate,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    parse_audio_event(ini, &mut target.unit_specific_sound)
}

fn parse_academy_classify(
    ini: &mut IniReader<'_>,
    target: &mut UpgradeTemplate,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut raw = 0;
    ini.parse_index_list(&mut raw, &ACADEMY_CLASSIFICATION_TYPE_NAMES)?;
    target.academy_classification_type =
        AcademyClassificationType::from_i32(raw).ok_or(IniError::InvalidData)?;
    Ok(())
}

static UPGRADE_FIELD_PARSE: &[FieldParseEntry<UpgradeTemplate>] = &[
    FieldParseEntry {
        token: Some("DisplayName"),
        parse: parse_display_name,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("Type"),
        parse: parse_upgrade_type,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("BuildTime"),
        parse: parse_build_time,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("BuildCost"),
        parse: parse_build_cost,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("ButtonImage"),
        parse: parse_button_image,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("ResearchSound"),
        parse: parse_research_sound,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("UnitSpecificSound"),
        parse: parse_unit_specific_sound,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("AcademyClassify"),
        parse: parse_academy_classify,
        user_data: FieldUserData::None,
    },
];

/// Global registry of `UpgradeTemplate` definitions (`TheUpgradeCenter`).
#[derive(Debug, Clone, Default)]
pub struct UpgradeCenter {
    /// Stored head-first (`index 0` is most recently linked), matching `m_upgradeList`.
    upgrades: Vec<UpgradeTemplate>,
    next_template_mask_bit: Int,
    button_images_cached: bool,
}

impl UpgradeCenter {
    /// Creates an empty `UpgradeCenter`.
    pub fn new() -> Self {
        Self {
            upgrades: Vec::new(),
            next_template_mask_bit: 0,
            button_images_cached: false,
        }
    }

    /// Registers the three object-level veterancy upgrades (`UpgradeCenter::init`).
    pub fn init(&mut self, name_keys: &mut NameKeyGenerator) {
        let levels = [
            VeterancyLevel::Veteran,
            VeterancyLevel::Elite,
            VeterancyLevel::Heroic,
        ];
        for level in levels {
            let idx = self.new_upgrade_index("", name_keys);
            self.upgrades[idx].friend_make_veterancy_upgrade(level, name_keys);
        }
    }

    /// Caches mapped button images once across registered templates (`UpgradeCenter::reset`).
    pub fn reset(&mut self, has_image_collection: bool, mut find_image: impl FnMut(&str) -> bool) {
        if has_image_collection && !self.button_images_cached {
            for upgrade in &mut self.upgrades {
                upgrade.cache_button_image(&mut find_image);
            }
            self.button_images_cached = true;
        }
    }

    /// Subsystem update tick (no-op matching C++).
    pub fn update(&mut self) {}

    /// Returns the first template in the LIFO linked list (`firstUpgradeTemplate`).
    pub fn first_upgrade_template(&self) -> Option<&UpgradeTemplate> {
        self.upgrades.first()
    }

    /// Looks up a template by its `NameKeyType`.
    pub fn find_upgrade_by_key(&self, key: NameKeyType) -> Option<&UpgradeTemplate> {
        self.upgrades.iter().find(|u| u.upgrade_name_key() == key)
    }

    /// Looks up a template by internal ASCII name (case-sensitive via `NameKeyGenerator`).
    pub fn find_upgrade(
        &self,
        name: &str,
        name_keys: &mut NameKeyGenerator,
    ) -> Option<&UpgradeTemplate> {
        let key = name_keys.name_to_key(name);
        self.find_upgrade_by_key(key)
    }

    /// Read-only lookup by internal ASCII name without interning unknown names.
    pub fn lookup_upgrade(
        &self,
        name: &str,
        name_keys: &NameKeyGenerator,
    ) -> Option<&UpgradeTemplate> {
        name_keys
            .lookup_key(name)
            .and_then(|key| self.find_upgrade_by_key(key))
    }

    /// Finds the veterancy upgrade template corresponding to `level`.
    pub fn find_veterancy_upgrade(
        &self,
        level: VeterancyLevel,
        name_keys: &mut NameKeyGenerator,
    ) -> Option<&UpgradeTemplate> {
        let name = get_vet_upgrade_name(level);
        self.find_upgrade(&name, name_keys)
    }

    fn new_upgrade_index(&mut self, name: &str, name_keys: &mut NameKeyGenerator) -> usize {
        // Must look up DefaultUpgrade before interning `name` to keep NameKey IDs bit-exact
        // with C++ (`Upgrade.cpp:371-377`).
        let mut tmpl = self
            .find_upgrade("DefaultUpgrade", name_keys)
            .cloned()
            .unwrap_or_default();

        tmpl.set_upgrade_name(name);
        tmpl.set_upgrade_name_key(name_keys.name_to_key(name));

        let mut new_mask = UpgradeMaskType::new();
        new_mask.set(self.next_template_mask_bit, 1);
        self.next_template_mask_bit += 1;
        tmpl.upgrade_mask = new_mask;

        self.upgrades.insert(0, tmpl);
        0
    }

    /// Allocates, links at head, and returns a new `UpgradeTemplate` (`UpgradeCenter::newUpgrade`).
    pub fn new_upgrade(
        &mut self,
        name: &str,
        name_keys: &mut NameKeyGenerator,
    ) -> &mut UpgradeTemplate {
        let idx = self.new_upgrade_index(name, name_keys);
        &mut self.upgrades[idx]
    }

    /// Checks whether `player` has sufficient money for `upgrade_template`.
    pub fn can_afford_upgrade(
        &self,
        player: Option<&dyn UpgradePlayerAccess>,
        upgrade_template: Option<&UpgradeTemplate>,
        display_reason: bool,
        mut on_ui_message: impl FnMut(&str),
    ) -> Bool {
        let (Some(player), Some(tmpl)) = (player, upgrade_template) else {
            return FALSE;
        };
        let cost = tmpl.calc_cost_to_build(Some(player));
        if (player.money_count() as i64) < (cost as i64) {
            if display_reason {
                on_ui_message("GUI:NotEnoughMoneyToUpgrade");
            }
            return FALSE;
        }
        TRUE
    }

    /// Returns all registered upgrade names in head-to-tail LIFO order (`getUpgradeNames`).
    pub fn get_upgrade_names(&self) -> Vec<String> {
        self.upgrades
            .iter()
            .map(|u| u.upgrade_name().to_owned())
            .collect()
    }

    /// Parses one `Upgrade <Name> ... End` INI block (`UpgradeCenter::parseUpgradeDefinition`).
    pub fn parse_upgrade_definition(
        &mut self,
        ini: &mut IniReader<'_>,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<(), IniError> {
        let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let key = name_keys.name_to_key(&name);

        let idx = match self
            .upgrades
            .iter()
            .position(|u| u.upgrade_name_key() == key)
        {
            Some(existing) => existing,
            None => self.new_upgrade_index(&name, name_keys),
        };

        ini.init_from_ini(&mut self.upgrades[idx], UPGRADE_FIELD_PARSE)
    }

    /// Loads all `Upgrade` blocks from `ini`.
    pub fn load_from_ini(
        &mut self,
        ini: &mut IniReader<'_>,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<UnsignedInt, IniError> {
        let _keep_science_unused: ScienceType = SCIENCE_INVALID;
        ini.load_blocks(None, |block_name, reader| {
            if block_name.eq_ignore_ascii_case("Upgrade") {
                let _block_kw = reader.next_token(SEPS_DEFAULT)?;
                self.parse_upgrade_definition(reader, name_keys)?;
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        })
    }
}
