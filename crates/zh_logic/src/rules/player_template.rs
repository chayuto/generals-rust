//! `PKT-130`: Playable/non-playable faction templates and legacy Zero Hour general remapping
//! (`GeneralsMD/Code/GameEngine/Include/Common/PlayerTemplate.h` &
//! `GeneralsMD/Code/GameEngine/Source/Common/RTS/PlayerTemplate.cpp`).

use std::collections::BTreeMap;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{FieldParseEntry, FieldUserData, IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::RGBColor;
use zh_common::types::{Bool, Int, NameKeyType, Real, UnsignedInt, VeterancyLevel, FALSE};

use crate::economy::handicap::Handicap;
use crate::economy::money::Money;
use crate::rules::science::{ScienceStore, ScienceTextLookup, ScienceVec};

pub const MAX_MP_STARTING_UNITS: usize = 10;

pub const VETERANCY_NAMES: [&str; 4] = ["REGULAR", "VETERAN", "ELITE", "HEROIC"];

pub type ProductionChangeMap = BTreeMap<NameKeyType, Real>;
pub type ProductionVeterancyMap = BTreeMap<NameKeyType, VeterancyLevel>;

/// Callback interface matching `TheMappedImageCollection->findImageByName(name) != nullptr`.
pub trait PlayerTemplateImageLookup {
    fn has_image(&self, name: &str) -> bool;
}

/// Faction template definition parsed from `PlayerTemplate <Name> ... End` INI blocks.
#[derive(Debug, Clone)]
pub struct PlayerTemplate {
    name_key: NameKeyType,
    display_name_utf16: Vec<u16>,
    side: String,
    base_side: String,
    handicap: Handicap,
    money: Money,
    preferred_color: RGBColor,
    starting_building: String,
    starting_units: [String; MAX_MP_STARTING_UNITS],
    production_cost_changes: ProductionChangeMap,
    production_time_changes: ProductionChangeMap,
    production_veterancy_levels: ProductionVeterancyMap,
    intrinsic_sciences: ScienceVec,
    purchase_science_command_set_rank1: String,
    purchase_science_command_set_rank3: String,
    purchase_science_command_set_rank8: String,
    special_power_shortcut_command_set: String,
    special_power_shortcut_win_name: String,
    special_power_shortcut_button_count: Int,
    load_screen_music: String,
    score_screen_music: String,
    tooltip: String,
    observer: Bool,
    playable_side: Bool,
    old_faction: Bool,
    intrinsic_spp: Int,
    score_screen_image: String,
    load_screen_image: String,
    head_water_mark: String,
    flag_water_mark: String,
    enabled_image: String,
    side_icon_image: String,
    general_image: String,
    beacon_template: String,
    str_general_features: String,
    str_medallion_normal: String,
    str_medallion_hilite: String,
    str_medallion_selected: String,
}

impl Default for PlayerTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerTemplate {
    pub fn new() -> Self {
        Self {
            name_key: NameKeyType::INVALID,
            display_name_utf16: Vec::new(),
            side: String::new(),
            base_side: String::new(),
            handicap: Handicap::new(),
            money: Money::new(),
            preferred_color: RGBColor::default(),
            starting_building: String::new(),
            starting_units: Default::default(),
            production_cost_changes: BTreeMap::new(),
            production_time_changes: BTreeMap::new(),
            production_veterancy_levels: BTreeMap::new(),
            intrinsic_sciences: Vec::new(),
            purchase_science_command_set_rank1: String::new(),
            purchase_science_command_set_rank3: String::new(),
            purchase_science_command_set_rank8: String::new(),
            special_power_shortcut_command_set: String::new(),
            special_power_shortcut_win_name: String::new(),
            special_power_shortcut_button_count: 0,
            load_screen_music: String::new(),
            score_screen_music: String::new(),
            tooltip: String::new(),
            observer: FALSE,
            playable_side: FALSE,
            old_faction: FALSE,
            intrinsic_spp: 0,
            score_screen_image: String::new(),
            load_screen_image: String::new(),
            head_water_mark: String::new(),
            flag_water_mark: String::new(),
            enabled_image: String::new(),
            side_icon_image: String::new(),
            general_image: String::new(),
            beacon_template: String::new(),
            str_general_features: String::new(),
            str_medallion_normal: String::new(),
            str_medallion_hilite: String::new(),
            str_medallion_selected: String::new(),
        }
    }

    #[inline]
    pub fn set_name_key(&mut self, name_key: NameKeyType) {
        self.name_key = name_key;
    }

    #[inline]
    pub fn name_key(&self) -> NameKeyType {
        self.name_key
    }

    #[inline]
    pub fn name<'a>(&self, name_keys: &'a NameKeyGenerator) -> &'a str {
        name_keys.key_to_name(self.name_key)
    }

    #[inline]
    pub fn display_name(&self) -> &[u16] {
        &self.display_name_utf16
    }

    #[inline]
    pub fn side(&self) -> &str {
        &self.side
    }

    #[inline]
    pub fn base_side(&self) -> &str {
        &self.base_side
    }

    #[inline]
    pub fn handicap(&self) -> &Handicap {
        &self.handicap
    }

    #[inline]
    pub fn money(&self) -> &Money {
        &self.money
    }

    #[inline]
    pub fn preferred_color(&self) -> &RGBColor {
        &self.preferred_color
    }

    #[inline]
    pub fn starting_building(&self) -> &str {
        &self.starting_building
    }

    pub fn starting_unit(&self, i: Int) -> &str {
        if i < 0 || i >= MAX_MP_STARTING_UNITS as Int {
            ""
        } else {
            &self.starting_units[i as usize]
        }
    }

    #[inline]
    pub fn production_cost_changes(&self) -> &ProductionChangeMap {
        &self.production_cost_changes
    }

    #[inline]
    pub fn production_time_changes(&self) -> &ProductionChangeMap {
        &self.production_time_changes
    }

    #[inline]
    pub fn production_veterancy_levels(&self) -> &ProductionVeterancyMap {
        &self.production_veterancy_levels
    }

    #[inline]
    pub fn is_observer(&self) -> Bool {
        self.observer
    }

    #[inline]
    pub fn is_playable_side(&self) -> Bool {
        self.playable_side
    }

    #[inline]
    pub fn is_old_faction(&self) -> Bool {
        self.old_faction
    }

    #[inline]
    pub fn intrinsic_sciences(&self) -> &ScienceVec {
        &self.intrinsic_sciences
    }

    #[inline]
    pub fn intrinsic_science_purchase_points(&self) -> Int {
        self.intrinsic_spp
    }

    #[inline]
    pub fn special_power_shortcut_button_count(&self) -> Int {
        self.special_power_shortcut_button_count
    }

    #[inline]
    pub fn purchase_science_command_set_rank1(&self) -> &str {
        &self.purchase_science_command_set_rank1
    }

    #[inline]
    pub fn purchase_science_command_set_rank3(&self) -> &str {
        &self.purchase_science_command_set_rank3
    }

    #[inline]
    pub fn purchase_science_command_set_rank8(&self) -> &str {
        &self.purchase_science_command_set_rank8
    }

    #[inline]
    pub fn special_power_shortcut_command_set(&self) -> &str {
        &self.special_power_shortcut_command_set
    }

    #[inline]
    pub fn special_power_shortcut_win_name(&self) -> &str {
        &self.special_power_shortcut_win_name
    }

    #[inline]
    pub fn load_screen_music(&self) -> &str {
        &self.load_screen_music
    }

    #[inline]
    pub fn score_screen_music(&self) -> &str {
        &self.score_screen_music
    }

    #[inline]
    pub fn score_screen(&self) -> &str {
        &self.score_screen_image
    }

    #[inline]
    pub fn load_screen(&self) -> &str {
        &self.load_screen_image
    }

    #[inline]
    pub fn beacon_template(&self) -> &str {
        &self.beacon_template
    }

    #[inline]
    pub fn tooltip(&self) -> &str {
        &self.tooltip
    }

    #[inline]
    pub fn general_features(&self) -> &str {
        &self.str_general_features
    }

    #[inline]
    pub fn medallion_normal(&self) -> &str {
        &self.str_medallion_normal
    }

    #[inline]
    pub fn medallion_hilite(&self) -> &str {
        &self.str_medallion_hilite
    }

    #[inline]
    pub fn medallion_selected(&self) -> &str {
        &self.str_medallion_selected
    }

    pub fn has_head_water_mark_image(&self, lookup: &dyn PlayerTemplateImageLookup) -> bool {
        lookup.has_image(&self.head_water_mark)
    }

    pub fn has_flag_water_mark_image(&self, lookup: &dyn PlayerTemplateImageLookup) -> bool {
        lookup.has_image(&self.flag_water_mark)
    }

    pub fn has_enabled_image(&self, lookup: &dyn PlayerTemplateImageLookup) -> bool {
        lookup.has_image(&self.enabled_image)
    }

    pub fn has_side_icon_image(&self, lookup: &dyn PlayerTemplateImageLookup) -> bool {
        lookup.has_image(&self.side_icon_image)
    }

    pub fn has_general_image(&self, lookup: &dyn PlayerTemplateImageLookup) -> bool {
        lookup.has_image(&self.general_image)
    }
}

#[derive(Debug, Clone)]
enum PtParseOp {
    Side(String),
    BaseSide(String),
    PlayableSide(Bool),
    DisplayName(String),
    StartMoney(UnsignedInt),
    PreferredColor(RGBColor),
    StartingBuilding(String),
    StartingUnit(usize, String),
    ProductionCostChange(String, Real),
    ProductionTimeChange(String, Real),
    ProductionVeterancyLevel(String, VeterancyLevel),
    IntrinsicSciences(Vec<String>),
    PurchaseSciCmd1(String),
    PurchaseSciCmd3(String),
    PurchaseSciCmd8(String),
    ShortcutCmdSet(String),
    ShortcutWinName(String),
    ShortcutBtnCount(Int),
    IsObserver(Bool),
    OldFaction(Bool),
    IntrinsicSpp(Int),
    ScoreScreenImage(String),
    LoadScreenImage(String),
    LoadScreenMusic(String),
    ScoreScreenMusic(String),
    HeadWaterMark(String),
    FlagWaterMark(String),
    EnabledImage(String),
    SideIconImage(String),
    GeneralImage(String),
    BeaconName(String),
    ArmyTooltip(String),
    Features(String),
    MedallionRegular(String),
    MedallionHilite(String),
    MedallionSelect(String),
}

#[derive(Debug, Default)]
struct PtParseTarget {
    ops: Vec<PtParseOp>,
}

fn pt_parse_str(
    ini: &mut IniReader<'_>,
    target: &mut PtParseTarget,
    make: fn(String) -> PtParseOp,
) -> Result<(), IniError> {
    let mut s = String::new();
    ini.parse_ascii_string(&mut s)?;
    target.ops.push(make(s));
    Ok(())
}

fn pt_side(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::Side)
}
fn pt_base_side(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::BaseSide)
}
fn pt_playable(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    let mut b = FALSE;
    ini.parse_bool(&mut b)?;
    t.ops.push(PtParseOp::PlayableSide(b));
    Ok(())
}
fn pt_display_name(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    let tok = ini.next_token(SEPS_DEFAULT)?.to_owned();
    t.ops.push(PtParseOp::DisplayName(tok));
    Ok(())
}
fn pt_start_money(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    let mut amount = 0;
    ini.parse_unsigned_int(&mut amount)?;
    t.ops.push(PtParseOp::StartMoney(amount));
    Ok(())
}
fn pt_pref_color(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    let mut c = RGBColor::default();
    ini.parse_rgb_color(&mut c)?;
    t.ops.push(PtParseOp::PreferredColor(c));
    Ok(())
}
fn pt_start_bldg(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::StartingBuilding)
}
fn pt_start_unit(ini: &mut IniReader<'_>, t: &mut PtParseTarget, ud: &FieldUserData) -> Result<(), IniError> {
    let FieldUserData::Mask(idx) = *ud else {
        return Err(IniError::InvalidData);
    };
    let mut s = String::new();
    ini.parse_ascii_string(&mut s)?;
    t.ops.push(PtParseOp::StartingUnit(idx as usize, s));
    Ok(())
}
fn pt_prod_cost(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    let tmpl = ini.next_token(SEPS_DEFAULT)?.to_owned();
    let pct = IniReader::scan_percent_to_real(ini.next_token(SEPS_DEFAULT)?)?;
    t.ops.push(PtParseOp::ProductionCostChange(tmpl, pct));
    Ok(())
}
fn pt_prod_time(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    let tmpl = ini.next_token(SEPS_DEFAULT)?.to_owned();
    let pct = IniReader::scan_percent_to_real(ini.next_token(SEPS_DEFAULT)?)?;
    t.ops.push(PtParseOp::ProductionTimeChange(tmpl, pct));
    Ok(())
}
fn pt_prod_vet(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    let tmpl = ini.next_token(SEPS_DEFAULT)?.to_owned();
    let lvl = IniReader::scan_index_list(ini.next_token(SEPS_DEFAULT)?, &VETERANCY_NAMES)?;
    t.ops
        .push(PtParseOp::ProductionVeterancyLevel(tmpl, VeterancyLevel::from_i32(lvl)));
    Ok(())
}
fn pt_intrinsic_sci(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    let mut tokens = Vec::new();
    while let Some(tok) = ini.next_token_or_null(SEPS_DEFAULT) {
        if tok.eq_ignore_ascii_case("None") {
            tokens.clear();
            break;
        }
        tokens.push(tok.to_owned());
    }
    t.ops.push(PtParseOp::IntrinsicSciences(tokens));
    Ok(())
}
fn pt_cmd1(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::PurchaseSciCmd1)
}
fn pt_cmd3(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::PurchaseSciCmd3)
}
fn pt_cmd8(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::PurchaseSciCmd8)
}
fn pt_sc_cmd(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::ShortcutCmdSet)
}
fn pt_sc_win(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::ShortcutWinName)
}
fn pt_sc_cnt(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    let mut v = 0;
    ini.parse_int(&mut v)?;
    t.ops.push(PtParseOp::ShortcutBtnCount(v));
    Ok(())
}
fn pt_observer(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    let mut b = FALSE;
    ini.parse_bool(&mut b)?;
    t.ops.push(PtParseOp::IsObserver(b));
    Ok(())
}
fn pt_old_faction(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    let mut b = FALSE;
    ini.parse_bool(&mut b)?;
    t.ops.push(PtParseOp::OldFaction(b));
    Ok(())
}
fn pt_intrinsic_spp(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    let mut v = 0;
    ini.parse_int(&mut v)?;
    t.ops.push(PtParseOp::IntrinsicSpp(v));
    Ok(())
}
fn pt_score_img(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::ScoreScreenImage)
}
fn pt_load_img(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::LoadScreenImage)
}
fn pt_load_mus(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::LoadScreenMusic)
}
fn pt_score_mus(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::ScoreScreenMusic)
}
fn pt_hw_img(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::HeadWaterMark)
}
fn pt_fw_img(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::FlagWaterMark)
}
fn pt_en_img(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::EnabledImage)
}
fn pt_side_img(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::SideIconImage)
}
fn pt_gen_img(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::GeneralImage)
}
fn pt_beacon(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::BeaconName)
}
fn pt_tooltip(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::ArmyTooltip)
}
fn pt_features(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::Features)
}
fn pt_med_reg(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::MedallionRegular)
}
fn pt_med_hilite(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::MedallionHilite)
}
fn pt_med_sel(ini: &mut IniReader<'_>, t: &mut PtParseTarget, _ud: &FieldUserData) -> Result<(), IniError> {
    pt_parse_str(ini, t, PtParseOp::MedallionSelect)
}

static PLAYER_TEMPLATE_FIELD_PARSE: &[FieldParseEntry<PtParseTarget>] = &[
    FieldParseEntry { token: Some("Side"), parse: pt_side, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("BaseSide"), parse: pt_base_side, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("PlayableSide"), parse: pt_playable, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("DisplayName"), parse: pt_display_name, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("StartMoney"), parse: pt_start_money, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("PreferredColor"), parse: pt_pref_color, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("StartingBuilding"), parse: pt_start_bldg, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("StartingUnit0"), parse: pt_start_unit, user_data: FieldUserData::Mask(0) },
    FieldParseEntry { token: Some("StartingUnit1"), parse: pt_start_unit, user_data: FieldUserData::Mask(1) },
    FieldParseEntry { token: Some("StartingUnit2"), parse: pt_start_unit, user_data: FieldUserData::Mask(2) },
    FieldParseEntry { token: Some("StartingUnit3"), parse: pt_start_unit, user_data: FieldUserData::Mask(3) },
    FieldParseEntry { token: Some("StartingUnit4"), parse: pt_start_unit, user_data: FieldUserData::Mask(4) },
    FieldParseEntry { token: Some("StartingUnit5"), parse: pt_start_unit, user_data: FieldUserData::Mask(5) },
    FieldParseEntry { token: Some("StartingUnit6"), parse: pt_start_unit, user_data: FieldUserData::Mask(6) },
    FieldParseEntry { token: Some("StartingUnit7"), parse: pt_start_unit, user_data: FieldUserData::Mask(7) },
    FieldParseEntry { token: Some("StartingUnit8"), parse: pt_start_unit, user_data: FieldUserData::Mask(8) },
    FieldParseEntry { token: Some("StartingUnit9"), parse: pt_start_unit, user_data: FieldUserData::Mask(9) },
    FieldParseEntry { token: Some("ProductionCostChange"), parse: pt_prod_cost, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("ProductionTimeChange"), parse: pt_prod_time, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("ProductionVeterancyLevel"), parse: pt_prod_vet, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("IntrinsicSciences"), parse: pt_intrinsic_sci, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("PurchaseScienceCommandSetRank1"), parse: pt_cmd1, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("PurchaseScienceCommandSetRank3"), parse: pt_cmd3, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("PurchaseScienceCommandSetRank8"), parse: pt_cmd8, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("SpecialPowerShortcutCommandSet"), parse: pt_sc_cmd, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("SpecialPowerShortcutWinName"), parse: pt_sc_win, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("SpecialPowerShortcutButtonCount"), parse: pt_sc_cnt, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("IsObserver"), parse: pt_observer, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("OldFaction"), parse: pt_old_faction, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("IntrinsicSciencePurchasePoints"), parse: pt_intrinsic_spp, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("ScoreScreenImage"), parse: pt_score_img, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("LoadScreenImage"), parse: pt_load_img, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("LoadScreenMusic"), parse: pt_load_mus, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("ScoreScreenMusic"), parse: pt_score_mus, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("HeadWaterMark"), parse: pt_hw_img, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("FlagWaterMark"), parse: pt_fw_img, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("EnabledImage"), parse: pt_en_img, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("SideIconImage"), parse: pt_side_img, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("GeneralImage"), parse: pt_gen_img, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("BeaconName"), parse: pt_beacon, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("ArmyTooltip"), parse: pt_tooltip, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("Features"), parse: pt_features, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("MedallionRegular"), parse: pt_med_reg, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("MedallionHilite"), parse: pt_med_hilite, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("MedallionSelect"), parse: pt_med_sel, user_data: FieldUserData::None },
];

/// Mirrors C++ `static NameKeyType a0..g4` inside `PlayerTemplateStore::findPlayerTemplate`.
#[derive(Debug, Clone)]
struct LegacyFactionKeys {
    a: [NameKeyType; 5],
    c: [NameKeyType; 5],
    g: [NameKeyType; 5],
}

impl LegacyFactionKeys {
    fn intern(name_keys: &mut NameKeyGenerator) -> Self {
        Self {
            a: [
                name_keys.name_to_key("FactionAmerica"),
                name_keys.name_to_key("FactionAmericaChooseAGeneral"),
                name_keys.name_to_key("FactionAmericaTankCommand"),
                name_keys.name_to_key("FactionAmericaSpecialForces"),
                name_keys.name_to_key("FactionAmericaAirForce"),
            ],
            c: [
                name_keys.name_to_key("FactionChina"),
                name_keys.name_to_key("FactionChinaChooseAGeneral"),
                name_keys.name_to_key("FactionChinaRedArmy"),
                name_keys.name_to_key("FactionChinaSpecialWeapons"),
                name_keys.name_to_key("FactionChinaSecretPolice"),
            ],
            g: [
                name_keys.name_to_key("FactionGLA"),
                name_keys.name_to_key("FactionGLAChooseAGeneral"),
                name_keys.name_to_key("FactionGLATerrorCell"),
                name_keys.name_to_key("FactionGLABiowarCommand"),
                name_keys.name_to_key("FactionGLAWarlordCommand"),
            ],
        }
    }

    fn remap(&self, key: NameKeyType) -> NameKeyType {
        if self.a[1..].contains(&key) {
            self.a[0]
        } else if self.c[1..].contains(&key) {
            self.c[0]
        } else if self.g[1..].contains(&key) {
            self.g[0]
        } else {
            key
        }
    }
}

/// Singleton store matching C++ `PlayerTemplateStore` (`ThePlayerTemplateStore`).
#[derive(Debug, Clone, Default)]
pub struct PlayerTemplateStore {
    player_templates: Vec<PlayerTemplate>,
    legacy_keys: Option<LegacyFactionKeys>,
}

impl PlayerTemplateStore {
    pub fn new() -> Self {
        Self {
            player_templates: Vec::new(),
            legacy_keys: None,
        }
    }

    pub fn init(&mut self) {
        self.player_templates.clear();
    }

    /// No-op matching C++ `PlayerTemplateStore::reset()` (retains loaded templates).
    pub fn reset(&mut self) {}

    pub fn update(&mut self) {}

    #[inline]
    pub fn player_template_count(&self) -> Int {
        self.player_templates.len() as Int
    }

    pub fn get_nth_player_template(&self, i: Int) -> Option<&PlayerTemplate> {
        if i >= 0 && (i as usize) < self.player_templates.len() {
            Some(&self.player_templates[i as usize])
        } else {
            None
        }
    }

    pub fn get_template_num_by_name(&self, name: &str, name_keys: &NameKeyGenerator) -> Int {
        for (idx, pt) in self.player_templates.iter().enumerate() {
            if pt.name(name_keys).eq_ignore_ascii_case(name) {
                return idx as Int;
            }
        }
        -1
    }

    fn ensure_legacy_keys(&mut self, name_keys: &mut NameKeyGenerator) -> &LegacyFactionKeys {
        self.legacy_keys
            .get_or_insert_with(|| LegacyFactionKeys::intern(name_keys))
    }

    pub fn find_player_template_index(
        &self,
        name_key: NameKeyType,
        name_keys: &NameKeyGenerator,
    ) -> Option<usize> {
        let effective = if let Some(ref legacy) = self.legacy_keys {
            legacy.remap(name_key)
        } else {
            let name = name_keys.key_to_name(name_key);
            let mapped = match name {
                "FactionAmericaChooseAGeneral"
                | "FactionAmericaTankCommand"
                | "FactionAmericaSpecialForces"
                | "FactionAmericaAirForce" => Some("FactionAmerica"),
                "FactionChinaChooseAGeneral"
                | "FactionChinaRedArmy"
                | "FactionChinaSpecialWeapons"
                | "FactionChinaSecretPolice" => Some("FactionChina"),
                "FactionGLAChooseAGeneral"
                | "FactionGLATerrorCell"
                | "FactionGLABiowarCommand"
                | "FactionGLAWarlordCommand" => Some("FactionGLA"),
                _ => None,
            };
            if let Some(target_name) = mapped {
                return self
                    .player_templates
                    .iter()
                    .position(|pt| pt.name(name_keys) == target_name);
            }
            name_key
        };
        self.player_templates
            .iter()
            .position(|pt| pt.name_key == effective)
    }

    pub fn find_player_template(
        &self,
        name_key: NameKeyType,
        name_keys: &NameKeyGenerator,
    ) -> Option<&PlayerTemplate> {
        self.find_player_template_index(name_key, name_keys)
            .map(|idx| &self.player_templates[idx])
    }

    /// Appends every unique `side` present in `self.player_templates` onto `out_string_list`
    /// without clearing pre-existing elements in `out_string_list`.
    pub fn get_all_side_strings(&self, out_string_list: &mut Vec<String>) {
        let mut tmp_list: Vec<String> = Vec::new();
        for pt in &self.player_templates {
            if !tmp_list.iter().any(|existing| existing == &pt.side) {
                tmp_list.push(pt.side.clone());
            }
        }
        out_string_list.extend(tmp_list);
    }

    /// Parses one `PlayerTemplate <Name> ... End` INI block.
    pub fn parse_player_template_definition(
        &mut self,
        ini: &mut IniReader<'_>,
        science_store: &ScienceStore,
        name_keys: &mut NameKeyGenerator,
        text_lookup: &mut dyn ScienceTextLookup,
    ) -> Result<(), IniError> {
        let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let name_key = name_keys.name_to_key(&name);

        let effective = self.ensure_legacy_keys(name_keys).remap(name_key);
        let existing_idx = self
            .player_templates
            .iter()
            .position(|pt| pt.name_key == effective);

        let mut parsed = PtParseTarget::default();
        ini.init_from_ini(&mut parsed, PLAYER_TEMPLATE_FIELD_PARSE)?;

        let mut target = if let Some(idx) = existing_idx {
            self.player_templates[idx].clone()
        } else {
            PlayerTemplate::new()
        };

        for op in parsed.ops {
            match op {
                PtParseOp::Side(s) => target.side = s,
                PtParseOp::BaseSide(s) => target.base_side = s,
                PtParseOp::PlayableSide(b) => target.playable_side = b,
                PtParseOp::DisplayName(label) => {
                    let translated = text_lookup.fetch_label(&label);
                    if translated.is_empty() {
                        return Err(IniError::InvalidData);
                    }
                    target.display_name_utf16 = translated.bytes().map(u16::from).collect();
                }
                PtParseOp::StartMoney(amount) => {
                    target.money.init();
                    target.money.set_starting_cash(amount);
                }
                PtParseOp::PreferredColor(c) => target.preferred_color = c,
                PtParseOp::StartingBuilding(s) => target.starting_building = s,
                PtParseOp::StartingUnit(idx, s) => {
                    if idx < MAX_MP_STARTING_UNITS {
                        target.starting_units[idx] = s;
                    }
                }
                PtParseOp::ProductionCostChange(tmpl, pct) => {
                    let key = name_keys.name_to_key(&tmpl);
                    target.production_cost_changes.insert(key, pct);
                }
                PtParseOp::ProductionTimeChange(tmpl, pct) => {
                    let key = name_keys.name_to_key(&tmpl);
                    target.production_time_changes.insert(key, pct);
                }
                PtParseOp::ProductionVeterancyLevel(tmpl, lvl) => {
                    let key = name_keys.name_to_key(&tmpl);
                    target.production_veterancy_levels.insert(key, lvl);
                }
                PtParseOp::IntrinsicSciences(tokens) => {
                    target.intrinsic_sciences.clear();
                    for tok in tokens {
                        target
                            .intrinsic_sciences
                            .push(science_store.friend_lookup_science(&tok, name_keys)?);
                    }
                }
                PtParseOp::PurchaseSciCmd1(s) => target.purchase_science_command_set_rank1 = s,
                PtParseOp::PurchaseSciCmd3(s) => target.purchase_science_command_set_rank3 = s,
                PtParseOp::PurchaseSciCmd8(s) => target.purchase_science_command_set_rank8 = s,
                PtParseOp::ShortcutCmdSet(s) => target.special_power_shortcut_command_set = s,
                PtParseOp::ShortcutWinName(s) => target.special_power_shortcut_win_name = s,
                PtParseOp::ShortcutBtnCount(c) => target.special_power_shortcut_button_count = c,
                PtParseOp::IsObserver(b) => target.observer = b,
                PtParseOp::OldFaction(b) => target.old_faction = b,
                PtParseOp::IntrinsicSpp(v) => target.intrinsic_spp = v,
                PtParseOp::ScoreScreenImage(s) => target.score_screen_image = s,
                PtParseOp::LoadScreenImage(s) => target.load_screen_image = s,
                PtParseOp::LoadScreenMusic(s) => target.load_screen_music = s,
                PtParseOp::ScoreScreenMusic(s) => target.score_screen_music = s,
                PtParseOp::HeadWaterMark(s) => target.head_water_mark = s,
                PtParseOp::FlagWaterMark(s) => target.flag_water_mark = s,
                PtParseOp::EnabledImage(s) => target.enabled_image = s,
                PtParseOp::SideIconImage(s) => target.side_icon_image = s,
                PtParseOp::GeneralImage(s) => target.general_image = s,
                PtParseOp::BeaconName(s) => target.beacon_template = s,
                PtParseOp::ArmyTooltip(s) => target.tooltip = s,
                PtParseOp::Features(s) => target.str_general_features = s,
                PtParseOp::MedallionRegular(s) => target.str_medallion_normal = s,
                PtParseOp::MedallionHilite(s) => target.str_medallion_hilite = s,
                PtParseOp::MedallionSelect(s) => target.str_medallion_selected = s,
            }
        }

        target.set_name_key(name_key);
        if let Some(idx) = existing_idx {
            self.player_templates[idx] = target;
        } else {
            self.player_templates.push(target);
        }
        Ok(())
    }

    pub fn load_from_ini(
        &mut self,
        ini: &mut IniReader<'_>,
        science_store: &ScienceStore,
        name_keys: &mut NameKeyGenerator,
        text_lookup: &mut dyn ScienceTextLookup,
    ) -> Result<(), IniError> {
        ini.load_blocks(None, |block_name, reader| {
            let _block_kw = reader.next_token(SEPS_DEFAULT)?;
            if block_name.eq_ignore_ascii_case("PlayerTemplate") {
                self.parse_player_template_definition(
                    reader,
                    science_store,
                    name_keys,
                    text_lookup,
                )
            } else {
                Err(IniError::UnknownToken)
            }
        })?;
        Ok(())
    }
}
