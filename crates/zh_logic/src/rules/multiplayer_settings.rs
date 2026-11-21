//! `PKT-128` & `PKT-104`: Multiplayer color/money rules & INI parsers
//! (`GeneralsMD/Code/GameEngine/Include/Common/MultiplayerSettings.h`,
//! `GeneralsMD/Code/GameEngine/Source/Common/MultiplayerSettings.cpp`, and
//! `Core/GameEngine/Source/Common/INI/INIMultiplayer.cpp`).

use std::collections::BTreeMap;
use zh_common::ini::{FieldParseEntry, FieldUserData, IniError, IniLoadType, IniReader, SEPS_DEFAULT};
use zh_common::math::RGBColor;
use zh_common::types::{Bool, Int, UnsignedInt, FALSE, TRUE};

use crate::economy::money::Money;

pub type Color = Int;

pub const PLAYERTEMPLATE_RANDOM: Int = -1;
pub const PLAYERTEMPLATE_OBSERVER: Int = -2;
pub const PLAYERTEMPLATE_MIN: Int = PLAYERTEMPLATE_OBSERVER;
pub const GAME_COLOR_UNDEFINED: Color = 0x00FF_FFFF;

#[derive(Debug, Clone, PartialEq)]
pub struct MultiplayerColorDefinition {
    tooltip_name: String,
    rgb_value: RGBColor,
    color: Color,
    rgb_value_night: RGBColor,
    color_night: Color,
}

impl Default for MultiplayerColorDefinition {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiplayerColorDefinition {
    pub fn new() -> Self {
        let mut rgb = RGBColor::default();
        rgb.set_from_int(0xFFFF_FFFF_u32 as Int);
        Self {
            tooltip_name: String::new(),
            rgb_value: rgb,
            color: 0xFFFF_FFFF_u32 as Int,
            rgb_value_night: rgb,
            color_night: 0xFFFF_FFFF_u32 as Int,
        }
    }

    #[inline]
    pub fn tooltip_name(&self) -> &str {
        &self.tooltip_name
    }

    #[inline]
    pub fn rgb_value(&self) -> RGBColor {
        self.rgb_value
    }

    #[inline]
    pub fn rgb_night_value(&self) -> RGBColor {
        self.rgb_value_night
    }

    #[inline]
    pub fn color(&self) -> Color {
        self.color
    }

    #[inline]
    pub fn night_color(&self) -> Color {
        self.color_night
    }

    pub fn set_color(&mut self, rgb: RGBColor) {
        self.color = rgb.get_as_int() | ((0xFF_u32 << 24) as Int);
    }

    pub fn set_night_color(&mut self, rgb: RGBColor) {
        self.color_night = rgb.get_as_int() | ((0xFF_u32 << 24) as Int);
    }
}

fn parse_color_tooltip_name(
    ini: &mut IniReader<'_>,
    target: &mut MultiplayerColorDefinition,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_ascii_string(&mut target.tooltip_name)
}

fn parse_color_rgb(
    ini: &mut IniReader<'_>,
    target: &mut MultiplayerColorDefinition,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_rgb_color(&mut target.rgb_value)
}

fn parse_color_rgb_night(
    ini: &mut IniReader<'_>,
    target: &mut MultiplayerColorDefinition,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_rgb_color(&mut target.rgb_value_night)
}

static MULTIPLAYER_COLOR_FIELD_PARSE: &[FieldParseEntry<MultiplayerColorDefinition>] = &[
    FieldParseEntry {
        token: Some("TooltipName"),
        parse: parse_color_tooltip_name,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("RGBColor"),
        parse: parse_color_rgb,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("RGBNightColor"),
        parse: parse_color_rgb_night,
        user_data: FieldUserData::None,
    },
];

pub type MultiplayerColorList = BTreeMap<Int, MultiplayerColorDefinition>;
pub type MultiplayerStartingMoneyList = Vec<Money>;

#[derive(Debug, Clone)]
pub struct MultiplayerSettings {
    initial_credits_min: Int,
    initial_credits_max: Int,
    start_countdown_timer_seconds: Int,
    max_beacons_per_player: Int,
    is_shroud_in_multiplayer: Bool,
    show_random_player_template: Bool,
    show_random_start_pos: Bool,
    show_random_color: Bool,
    color_list: MultiplayerColorList,
    num_colors: Int,
    observer_color: MultiplayerColorDefinition,
    random_color: MultiplayerColorDefinition,
    starting_money_list: MultiplayerStartingMoneyList,
    default_starting_money: Money,
    got_default_starting_money: bool,
}

impl Default for MultiplayerSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiplayerSettings {
    pub fn new() -> Self {
        let mut settings = Self {
            initial_credits_min: 0,
            initial_credits_max: 0,
            start_countdown_timer_seconds: 0,
            max_beacons_per_player: 3,
            is_shroud_in_multiplayer: TRUE,
            show_random_player_template: TRUE,
            show_random_start_pos: TRUE,
            show_random_color: TRUE,
            color_list: BTreeMap::new(),
            num_colors: 0,
            observer_color: MultiplayerColorDefinition::new(),
            random_color: MultiplayerColorDefinition::new(),
            starting_money_list: Vec::new(),
            default_starting_money: Money::new(),
            got_default_starting_money: false,
        };

        for amount in [100_000_u32, 150_000_u32, 200_000_u32] {
            let mut m = Money::new();
            m.set_starting_cash(amount);
            if !settings
                .starting_money_list
                .iter()
                .any(|existing| existing.amount_equal(&m) != FALSE)
            {
                settings.add_starting_money_choice(&m, FALSE);
            }
        }
        settings
            .starting_money_list
            .sort_by_key(Money::count_money);
        settings
    }

    pub fn init(&mut self) {}
    pub fn update(&mut self) {}
    pub fn reset(&mut self) {}

    #[inline]
    pub fn initial_credits_min(&self) -> Int {
        self.initial_credits_min
    }

    #[inline]
    pub fn initial_credits_max(&self) -> Int {
        self.initial_credits_max
    }

    #[inline]
    pub fn start_countdown_timer_seconds(&self) -> Int {
        self.start_countdown_timer_seconds
    }

    #[inline]
    pub fn max_beacons_per_player(&self) -> Int {
        self.max_beacons_per_player
    }

    #[inline]
    pub fn is_shroud_in_multiplayer(&self) -> Bool {
        self.is_shroud_in_multiplayer
    }

    #[inline]
    pub fn show_random_player_template(&self) -> Bool {
        self.show_random_player_template
    }

    #[inline]
    pub fn show_random_start_pos(&self) -> Bool {
        self.show_random_start_pos
    }

    #[inline]
    pub fn show_random_color(&self) -> Bool {
        self.show_random_color
    }

    pub fn get_num_colors(&mut self) -> Int {
        if self.num_colors == 0 {
            self.num_colors = self.color_list.len() as Int;
        }
        self.num_colors
    }

    #[inline]
    pub fn num_colors_cached(&self) -> Int {
        if self.num_colors == 0 {
            self.color_list.len() as Int
        } else {
            self.num_colors
        }
    }

    pub fn get_color(&mut self, which: Int) -> Option<&MultiplayerColorDefinition> {
        if which == PLAYERTEMPLATE_RANDOM {
            return Some(&self.random_color);
        }
        if which == PLAYERTEMPLATE_OBSERVER {
            return Some(&self.observer_color);
        }
        if which < 0 || which >= self.get_num_colors() {
            return None;
        }
        self.color_list.get(&which)
    }

    pub fn find_multiplayer_color_definition_by_name(
        &self,
        name: &str,
    ) -> Option<&MultiplayerColorDefinition> {
        self.color_list
            .values()
            .find(|def| def.tooltip_name == name)
    }

    pub fn find_multiplayer_color_definition_by_name_mut(
        &mut self,
        name: &str,
    ) -> Option<&mut MultiplayerColorDefinition> {
        self.color_list
            .values_mut()
            .find(|def| def.tooltip_name == name)
    }

    pub fn new_multiplayer_color_definition(
        &mut self,
        _name: &str,
    ) -> &mut MultiplayerColorDefinition {
        let num_colors = self.get_num_colors();
        self.color_list
            .insert(num_colors, MultiplayerColorDefinition::new());
        self.num_colors = self.color_list.len() as Int;
        self.color_list
            .get_mut(&num_colors)
            .expect("inserted color key")
    }

    #[inline]
    pub fn got_default_starting_money(&self) -> bool {
        self.got_default_starting_money
    }

    pub fn default_starting_money(&self) -> &Money {
        &self.default_starting_money
    }

    pub fn starting_money_list(&self) -> &[Money] {
        &self.starting_money_list
    }

    pub fn add_starting_money_choice(&mut self, money: &Money, is_default: Bool) {
        self.starting_money_list.push(money.clone());
        if is_default != FALSE {
            self.default_starting_money = money.clone();
            self.got_default_starting_money = true;
        }
    }
}

fn parse_ms_countdown(
    ini: &mut IniReader<'_>,
    target: &mut MultiplayerSettings,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_int(&mut target.start_countdown_timer_seconds)
}

fn parse_ms_beacons(
    ini: &mut IniReader<'_>,
    target: &mut MultiplayerSettings,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_int(&mut target.max_beacons_per_player)
}

fn parse_ms_shroud(
    ini: &mut IniReader<'_>,
    target: &mut MultiplayerSettings,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_bool(&mut target.is_shroud_in_multiplayer)
}

fn parse_ms_rand_pt(
    ini: &mut IniReader<'_>,
    target: &mut MultiplayerSettings,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_bool(&mut target.show_random_player_template)
}

fn parse_ms_rand_pos(
    ini: &mut IniReader<'_>,
    target: &mut MultiplayerSettings,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_bool(&mut target.show_random_start_pos)
}

fn parse_ms_rand_col(
    ini: &mut IniReader<'_>,
    target: &mut MultiplayerSettings,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_bool(&mut target.show_random_color)
}

static MULTIPLAYER_SETTINGS_FIELD_PARSE: &[FieldParseEntry<MultiplayerSettings>] = &[
    FieldParseEntry {
        token: Some("StartCountdownTimer"),
        parse: parse_ms_countdown,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("MaxBeaconsPerPlayer"),
        parse: parse_ms_beacons,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("UseShroud"),
        parse: parse_ms_shroud,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("ShowRandomPlayerTemplate"),
        parse: parse_ms_rand_pt,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("ShowRandomStartPos"),
        parse: parse_ms_rand_pos,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("ShowRandomColor"),
        parse: parse_ms_rand_col,
        user_data: FieldUserData::None,
    },
];

struct MultiplayerStartingMoneySettings {
    money: Money,
    is_default: Bool,
}

fn parse_starting_money_value(
    ini: &mut IniReader<'_>,
    target: &mut MultiplayerStartingMoneySettings,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut amount: UnsignedInt = 0;
    ini.parse_unsigned_int(&mut amount)?;
    target.money.set_starting_cash(amount);
    Ok(())
}

fn parse_starting_money_default(
    ini: &mut IniReader<'_>,
    target: &mut MultiplayerStartingMoneySettings,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_bool(&mut target.is_default)
}

static STARTING_MONEY_FIELD_PARSE: &[FieldParseEntry<MultiplayerStartingMoneySettings>] = &[
    FieldParseEntry {
        token: Some("Value"),
        parse: parse_starting_money_value,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("Default"),
        parse: parse_starting_money_default,
        user_data: FieldUserData::None,
    },
];

/// Parses `MultiplayerSettings ... End` (`INI::parseMultiplayerSettingsDefinition`).
pub fn parse_multiplayer_settings_definition(
    settings: &mut Option<MultiplayerSettings>,
    ini: &mut IniReader<'_>,
) -> Result<(), IniError> {
    let target = settings.get_or_insert_with(MultiplayerSettings::new);
    ini.init_from_ini(target, MULTIPLAYER_SETTINGS_FIELD_PARSE)
}

/// Parses `MultiplayerColor <Name> ... End` (`INI::parseMultiplayerColorDefinition`).
pub fn parse_multiplayer_color_definition(
    settings: &mut MultiplayerSettings,
    ini: &mut IniReader<'_>,
) -> Result<(), IniError> {
    let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
    let existing_key = settings
        .color_list
        .iter()
        .find_map(|(&k, def)| (def.tooltip_name == name).then_some(k));

    let def = if let Some(key) = existing_key {
        settings.color_list.get_mut(&key).expect("existing color key")
    } else {
        settings.new_multiplayer_color_definition(&name)
    };

    ini.init_from_ini(def, MULTIPLAYER_COLOR_FIELD_PARSE)?;
    let rgb = def.rgb_value();
    let night = def.rgb_night_value();
    def.set_color(rgb);
    def.set_night_color(night);
    Ok(())
}

/// Parses `MultiplayerStartingMoneyChoice ... End` (`INI::parseMultiplayerStartingMoneyChoiceDefinition`).
pub fn parse_multiplayer_starting_money_choice_definition(
    settings: &mut MultiplayerSettings,
    ini: &mut IniReader<'_>,
) -> Result<(), IniError> {
    if ini.get_load_type() == IniLoadType::CreateOverrides {
        return Err(IniError::InvalidData);
    }
    let mut parsed = MultiplayerStartingMoneySettings {
        money: Money::new(),
        is_default: FALSE,
    };
    ini.init_from_ini(&mut parsed, STARTING_MONEY_FIELD_PARSE)?;
    settings.add_starting_money_choice(&parsed.money, parsed.is_default);
    Ok(())
}

/// Loads all `MultiplayerSettings`, `MultiplayerColor`, and `MultiplayerStartingMoneyChoice`
/// blocks from `ini`.
pub fn load_multiplayer_ini(
    settings: &mut Option<MultiplayerSettings>,
    ini: &mut IniReader<'_>,
) -> Result<(), IniError> {
    ini.load_blocks(None, |block_name, reader| {
        let _block_kw = reader.next_token(SEPS_DEFAULT)?;
        if block_name.eq_ignore_ascii_case("MultiplayerSettings") {
            parse_multiplayer_settings_definition(settings, reader)
        } else if block_name.eq_ignore_ascii_case("MultiplayerColor") {
            let target = settings.get_or_insert_with(MultiplayerSettings::new);
            parse_multiplayer_color_definition(target, reader)
        } else if block_name.eq_ignore_ascii_case("MultiplayerStartingMoneyChoice") {
            let target = settings.get_or_insert_with(MultiplayerSettings::new);
            parse_multiplayer_starting_money_choice_definition(target, reader)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(())
}
