//! `PKT-434`: Zero Hour `Scripts` data structures (`ScriptList`, `ScriptGroup`, `Script`,
//! `OrCondition`, `Condition`, `ScriptAction`, `Parameter`, `Template`, `BorderColor`, `ShellHooks`).
//!
//! Reference C++ source:
//! - `GeneralsMD/Code/GameEngine/Include/GameLogic/Scripts.h`
//! - `GeneralsMD/Code/GameEngine/Source/GameLogic/ScriptEngine/Scripts.cpp`
//! - `GeneralsMD/Code/GameEngine/Include/Common/BorderColors.h`
//! - `Core/GameEngine/Include/GameClient/ShellHooks.h`

#![allow(unused_variables, dead_code)]

use crate::rules::kind_of::{KindOfMaskType, KINDOF_COUNT, KINDOF_NAMES};
use crate::rules::object_status_types::{
    ObjectStatusMaskType, OBJECT_STATUS_COUNT, OBJECT_STATUS_NAMES,
};
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::io::data_chunk::{
    ChunkInputStream, DataChunkError, DataChunkInput, DataChunkOutput, DataChunkOutputStream,
};
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, NameKeyType, Real, UnsignedInt, UnsignedShort, FALSE, TRUE,
};
use zh_common::xfer::{Snapshot, Xfer, XferError};

pub const MAX_PARMS: usize = 12;
pub const MAX_PLAYER_COUNT: usize = 16;
pub const AT_END: Int = 0x00FF_FFFF;

pub const THIS_TEAM: &str = "<This Team>";
pub const ANY_TEAM: &str = "<Any Team>";
pub const THIS_OBJECT: &str = "<This Object>";
pub const ANY_OBJECT: &str = "<Any Object>";
pub const THIS_PLAYER: &str = "<This Player>";
pub const LOCAL_PLAYER: &str = "<Local Player>";
pub const THE_PLAYER: &str = "ThePlayer";
pub const TEAM_THE_PLAYER: &str = "teamThePlayer";
pub const THIS_PLAYER_ENEMY: &str = "<This Player's Enemy>";
pub const WATER_GRID: &str = "Water Grid";

pub const SKIRMISH_CENTER: &str = "Center";
pub const SKIRMISH_FLANK: &str = "Flank";
pub const SKIRMISH_BACKDOOR: &str = "Backdoor";
pub const SKIRMISH_SPECIAL: &str = "Special";
pub const SKIRMISH_AREA_HOME_BASE: &str = "Home Base";
pub const SKIRMISH_AREA_ANY_SUPPLYDEPOT: &str = "Any Supply Depot";

pub const MY_INNER_PERIMETER: &str = "[Skirmish]MyInnerPerimeter";
pub const MY_OUTER_PERIMETER: &str = "[Skirmish]MyOuterPerimeter";
pub const ENEMY_OUTER_PERIMETER: &str = "[Skirmish]EnemyOuterPerimeter";
pub const ENEMY_INNER_PERIMETER: &str = "[Skirmish]EnemyInnerPerimeter";
pub const INNER_PERIMETER: &str = "InnerPerimeter";
pub const OUTER_PERIMETER: &str = "OuterPerimeter";

pub const K_SCRIPT_LIST_DATA_VERSION_1: UnsignedShort = 1;
pub const K_SCRIPT_GROUP_DATA_VERSION_1: UnsignedShort = 1;
pub const K_SCRIPT_GROUP_DATA_VERSION_2: UnsignedShort = 2;
pub const K_SCRIPT_DATA_VERSION_1: UnsignedShort = 1;
pub const K_SCRIPT_DATA_VERSION_2: UnsignedShort = 2;
pub const K_SCRIPT_OR_CONDITION_DATA_VERSION_1: UnsignedShort = 1;
pub const K_SCRIPT_ACTION_VERSION_1: UnsignedShort = 1;
pub const K_SCRIPT_ACTION_VERSION_2: UnsignedShort = 2;
pub const K_SCRIPT_CONDITION_VERSION_1: UnsignedShort = 1;
pub const K_SCRIPT_CONDITION_VERSION_2: UnsignedShort = 2;
pub const K_SCRIPT_CONDITION_VERSION_3: UnsignedShort = 3;
pub const K_SCRIPT_CONDITION_VERSION_4: UnsignedShort = 4;
pub const K_SCRIPTS_DATA_VERSION_1: UnsignedShort = 5;

pub const PARAMETER_NUM_ITEMS: Int = 52;
pub const CONDITION_NUM_ITEMS: Int = 109;
pub const ACTION_NUM_ITEMS: Int = 344;
pub const SHELL_SCRIPT_HOOK_TOTAL: usize = 29;
pub const BORDER_COLORS_SIZE: usize = 8;

pub const SURFACES: [&str; 3] = ["Ground", "Air", "Ground or Air"];

pub const SHAKE_INTENSITIES: [&str; 6] = [
    "Subtle",
    "Normal",
    "Strong",
    "Severe",
    "Cine_Extreme",
    "Cine_Insane",
];

pub const BUILDABLE_STATUS_NAMES: [&str; 4] =
    ["Yes", "Ignore_Prerequisites", "No", "Only_By_AI"];

pub const THE_OBJECT_FLAGS_NAMES: [&str; 7] = [
    "Enabled",
    "Powered",
    "Indestructible",
    "Unsellable",
    "Selectable",
    "AI Recruitable",
    "Player Targetable",
];

pub const THE_SHELL_HOOK_NAMES: [&str; SHELL_SCRIPT_HOOK_TOTAL] = [
    "ShellMainMenuCampaignPushed",
    "ShellMainMenuCampaignHighlighted",
    "ShellMainMenuCampaignUnhighlighted",
    "ShellMainMenuSkirmishPushed",
    "ShellMainMenuSkirmishHighlighted",
    "ShellMainMenuSkirmishUnhighlighted",
    "ShellMainMenuOptionsPushed",
    "ShellMainMenuOptionsHighlighted",
    "ShellMainMenuOptionsUnhighlighted",
    "ShellMainMenuOnlinePushed",
    "ShellMainMenuOnlineHighlighted",
    "ShellMainMenuOnlineUnhighlighted",
    "ShellMainMenuNetworkPushed",
    "ShellMainMenuNetworkHighlighted",
    "ShellMainMenuNetworkUnhighlighted",
    "ShellMainMenuExitPushed",
    "ShellMainMenuExitHighlighted",
    "ShellMainMenuExitUnhighlighted",
    "ShellGeneralsOnlineLogin",
    "ShellGeneralsOnlineLogout",
    "ShellGeneralsOnlineEnteredFromGame",
    "ShellOptionsOpened",
    "ShellOptionsClosed",
    "ShellSkirmishOpened",
    "ShellSkirmishClosed",
    "ShellSkirmishEnteredFromGame",
    "ShellLANOpened",
    "ShellLANClosed",
    "ShellLANEnteredFromGame",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderColor {
    pub color_name: &'static str,
    pub border_color: u32,
}

pub const BORDER_COLORS: [BorderColor; BORDER_COLORS_SIZE] = [
    BorderColor {
        color_name: "Orange",
        border_color: 0xFFFF_8700,
    },
    BorderColor {
        color_name: "Green",
        border_color: 0xFF00_FF00,
    },
    BorderColor {
        color_name: "Blue",
        border_color: 0xFF00_00FF,
    },
    BorderColor {
        color_name: "Cyan",
        border_color: 0xFF00_FFFF,
    },
    BorderColor {
        color_name: "Magenta",
        border_color: 0xFFFF_00FF,
    },
    BorderColor {
        color_name: "Yellow",
        border_color: 0xFFFF_FF00,
    },
    BorderColor {
        color_name: "Purple",
        border_color: 0xFF9E_00FF,
    },
    BorderColor {
        color_name: "Pink",
        border_color: 0xFFFF_8670,
    },
];

/// Parameter type tag (`Parameter::ParameterType`, `0..52`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParameterType {
    #[default]
    Int = 0,
    Real = 1,
    Script = 2,
    Team = 3,
    Counter = 4,
    Flag = 5,
    Comparison = 6,
    Waypoint = 7,
    Boolean = 8,
    TriggerArea = 9,
    TextString = 10,
    Side = 11,
    Sound = 12,
    ScriptSubroutine = 13,
    Unit = 14,
    ObjectType = 15,
    Coord3D = 16,
    Angle = 17,
    TeamState = 18,
    Relation = 19,
    AiMood = 20,
    Dialog = 21,
    Music = 22,
    Movie = 23,
    WaypointPath = 24,
    LocalizedText = 25,
    Bridge = 26,
    KindOfParam = 27,
    AttackPrioritySet = 28,
    RadarEventType = 29,
    SpecialPower = 30,
    Science = 31,
    Upgrade = 32,
    CommandbuttonAbility = 33,
    Boundary = 34,
    Buildable = 35,
    SurfacesAllowed = 36,
    ShakeIntensity = 37,
    CommandButton = 38,
    FontName = 39,
    ObjectStatus = 40,
    CommandbuttonAllAbilities = 41,
    SkirmishWaypointPath = 42,
    Color = 43,
    Emoticon = 44,
    ObjectPanelFlag = 45,
    FactionName = 46,
    ObjectTypeList = 47,
    Revealname = 48,
    ScienceAvailability = 49,
    LeftOrRight = 50,
    Percent = 51,
    NumItems = 52,
}

impl ParameterType {
    pub const fn from_i32(v: Int) -> Self {
        match v {
            0 => Self::Int,
            1 => Self::Real,
            2 => Self::Script,
            3 => Self::Team,
            4 => Self::Counter,
            5 => Self::Flag,
            6 => Self::Comparison,
            7 => Self::Waypoint,
            8 => Self::Boolean,
            9 => Self::TriggerArea,
            10 => Self::TextString,
            11 => Self::Side,
            12 => Self::Sound,
            13 => Self::ScriptSubroutine,
            14 => Self::Unit,
            15 => Self::ObjectType,
            16 => Self::Coord3D,
            17 => Self::Angle,
            18 => Self::TeamState,
            19 => Self::Relation,
            20 => Self::AiMood,
            21 => Self::Dialog,
            22 => Self::Music,
            23 => Self::Movie,
            24 => Self::WaypointPath,
            25 => Self::LocalizedText,
            26 => Self::Bridge,
            27 => Self::KindOfParam,
            28 => Self::AttackPrioritySet,
            29 => Self::RadarEventType,
            30 => Self::SpecialPower,
            31 => Self::Science,
            32 => Self::Upgrade,
            33 => Self::CommandbuttonAbility,
            34 => Self::Boundary,
            35 => Self::Buildable,
            36 => Self::SurfacesAllowed,
            37 => Self::ShakeIntensity,
            38 => Self::CommandButton,
            39 => Self::FontName,
            40 => Self::ObjectStatus,
            41 => Self::CommandbuttonAllAbilities,
            42 => Self::SkirmishWaypointPath,
            43 => Self::Color,
            44 => Self::Emoticon,
            45 => Self::ObjectPanelFlag,
            46 => Self::FactionName,
            47 => Self::ObjectTypeList,
            48 => Self::Revealname,
            49 => Self::ScienceAvailability,
            50 => Self::LeftOrRight,
            51 => Self::Percent,
            _ => Self::NumItems,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Template metadata for a script `Condition` or `ScriptAction` (`Template` in `Scripts.h`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptTemplate {
    pub ui_name: String,
    pub ui_name2: String,
    pub internal_name: String,
    pub internal_name_key: NameKeyType,
    pub num_ui_strings: Int,
    pub ui_strings: [String; MAX_PARMS],
    pub num_parameters: Int,
    pub parameters: [ParameterType; MAX_PARMS],
    pub help_text: String,
}

impl Default for ScriptTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptTemplate {
    pub fn new() -> Self {
        Self {
            ui_name: "UNUSED/(placeholder)/placeholder".to_string(),
            ui_name2: String::new(),
            internal_name: String::new(),
            internal_name_key: NameKeyType::INVALID,
            num_ui_strings: 0,
            ui_strings: std::array::from_fn(|_| String::new()),
            num_parameters: 0,
            parameters: [ParameterType::Int; MAX_PARMS],
            help_text: String::new(),
        }
    }

    pub fn get_ui_strings(&self, out: &mut [String; MAX_PARMS]) -> Int {
        let count = (self.num_ui_strings.max(0) as usize).min(MAX_PARMS);
        for i in 0..count {
            out[i] = self.ui_strings[i].clone();
        }
        self.num_ui_strings
    }

    pub fn get_parameter_type(&self, ndx: Int) -> ParameterType {
        if ndx >= 0 && ndx < self.num_parameters && (ndx as usize) < MAX_PARMS {
            self.parameters[ndx as usize]
        } else {
            ParameterType::Int
        }
    }
}

/// Trait providing `ConditionTemplate` and `ActionTemplate` lookup (`TheScriptEngine->getConditionTemplate` / `getActionTemplate`).
pub trait ScriptTemplateProvider {
    fn get_condition_template(&self, condition_type: Int) -> Option<&ScriptTemplate>;
    fn get_action_template(&self, action_type: Int) -> Option<&ScriptTemplate>;
}

/// Concrete registry holding `CONDITION_NUM_ITEMS` condition templates and `ACTION_NUM_ITEMS` action templates.
#[derive(Debug, Clone)]
pub struct ScriptTemplateRegistry {
    pub conditions: Vec<ScriptTemplate>,
    pub actions: Vec<ScriptTemplate>,
}

impl Default for ScriptTemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptTemplateRegistry {
    pub fn new() -> Self {
        Self {
            conditions: (0..CONDITION_NUM_ITEMS as usize)
                .map(|_| ScriptTemplate::new())
                .collect(),
            actions: (0..ACTION_NUM_ITEMS as usize)
                .map(|_| ScriptTemplate::new())
                .collect(),
        }
    }
}

impl ScriptTemplateProvider for ScriptTemplateRegistry {
    fn get_condition_template(&self, condition_type: Int) -> Option<&ScriptTemplate> {
        if condition_type < 0 || condition_type >= CONDITION_NUM_ITEMS {
            return None;
        }
        self.conditions.get(condition_type as usize)
    }

    fn get_action_template(&self, action_type: Int) -> Option<&ScriptTemplate> {
        if action_type < 0 || action_type >= ACTION_NUM_ITEMS {
            return None;
        }
        self.actions.get(action_type as usize)
    }
}

/// Script condition or action parameter (`Parameter` in `Scripts.h`).
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub param_type: ParameterType,
    pub initialized: Bool,
    pub int_val: Int,
    pub real_val: Real,
    pub string_val: String,
    pub coord: Coord3D,
    pub object_status: ObjectStatusMaskType,
}

impl Parameter {
    pub fn new(param_type: ParameterType, val: Int) -> Self {
        Self {
            param_type,
            initialized: FALSE,
            int_val: val,
            real_val: 0.0,
            string_val: String::new(),
            coord: Coord3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            object_status: ObjectStatusMaskType::new(),
        }
    }

    pub fn get_coord3d(&self) -> Coord3D {
        if self.param_type == ParameterType::Coord3D {
            self.coord
        } else {
            Coord3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }
        }
    }

    pub fn set_coord3d(&mut self, loc: &Coord3D) {
        if self.param_type == ParameterType::Coord3D {
            self.coord = *loc;
        }
    }

    pub fn qualify(
        &mut self,
        qualifier: &str,
        player_template_name: &str,
        new_player_name: &str,
    ) {
        match self.param_type {
            ParameterType::Side => {
                let tmp = format!("{}{}", self.string_val, qualifier);
                if tmp == player_template_name {
                    self.string_val = new_player_name.to_string();
                }
            }
            ParameterType::Team => {
                if self.string_val != THIS_TEAM {
                    self.string_val.push_str(qualifier);
                }
            }
            ParameterType::Script
            | ParameterType::Counter
            | ParameterType::Flag
            | ParameterType::ScriptSubroutine => self.string_val.push_str(qualifier),
            _ => {}
        }
    }

    pub fn get_ui_text(&self) -> String {
        let ui_string = if self.string_val.is_empty() {
            "???"
        } else {
            &self.string_val
        };
        match self.param_type {
            ParameterType::Sound => format!("Sound '{}'", ui_string),
            ParameterType::Script => format!("Script '{}'", ui_string),
            ParameterType::TeamState => format!("'{}'", ui_string),
            ParameterType::ScriptSubroutine => format!("Subroutine '{}'", ui_string),
            ParameterType::AttackPrioritySet => format!("Attack priority set '{}'", ui_string),
            ParameterType::Waypoint => format!("Waypoint '{}'", ui_string),
            ParameterType::WaypointPath => format!("Waypoint Path '{}'", ui_string),
            ParameterType::TriggerArea => format!(" area '{}'", ui_string),
            ParameterType::CommandButton => format!("Command button: '{}'", ui_string),
            ParameterType::FontName => format!("Font: '{}'", ui_string),
            ParameterType::LocalizedText => format!("Localized String: '{}'", ui_string),
            ParameterType::TextString => format!("String: '{}'", ui_string),
            ParameterType::Team => format!("Team '{}'", ui_string),
            ParameterType::Unit => format!("Unit '{}'", ui_string),
            ParameterType::Bridge => format!("Bridge '{}'", ui_string),
            ParameterType::Angle => {
                format!("{:.2} degrees", self.real_val * 180.0 / 3.14159265359_f32)
            }
            ParameterType::Percent => format!("{:.2}%", self.real_val * 100.0),
            ParameterType::Coord3D => {
                let pos = self.coord;
                format!("({:.2},{:.2},{:.2})", pos.x, pos.y, pos.z)
            }
            ParameterType::ObjectType => format!("'{}'", ui_string),
            ParameterType::KindOfParam => {
                if self.int_val >= 0 && (self.int_val as usize) < KINDOF_COUNT {
                    let name =
                        KindOfMaskType::get_name_from_single_bit(&KINDOF_NAMES, self.int_val)
                            .unwrap_or("???");
                    format!("Kind is '{}'", name)
                } else {
                    "Kind is ???".to_string()
                }
            }
            ParameterType::Side => format!("Player '{}'", ui_string),
            ParameterType::Counter => format!("'{}'", ui_string),
            ParameterType::Int => format!(" {} ", self.int_val),
            ParameterType::Boolean => if self.int_val != 0 { "TRUE".to_string() } else { "FALSE".to_string() },
            ParameterType::Real => format!("{:.2}", self.real_val),
            ParameterType::Flag => format!("Flag named '{}'", ui_string),
            ParameterType::Comparison => match self.int_val {
                0 => "Less Than".to_string(),
                1 => "Less Than or Equal".to_string(),
                2 => "Equal To".to_string(),
                3 => "Greater Than or Equal To".to_string(),
                4 => "Greater Than".to_string(),
                5 => "Not Equal To".to_string(),
                _ => String::new(),
            },
            ParameterType::Relation => match self.int_val {
                0 => "Enemy".to_string(),
                1 => "Neutral".to_string(),
                2 => "Friend".to_string(),
                _ => String::new(),
            },
            ParameterType::AiMood => match self.int_val {
                -2 => "Sleep".to_string(),
                -1 => "Passive".to_string(),
                0 => "Normal".to_string(),
                1 => "Alert".to_string(),
                2 => "Aggressive".to_string(),
                _ => String::new(),
            },
            ParameterType::RadarEventType => match self.int_val {
                0 | 1 => "Construction".to_string(),
                2 => "Upgrade".to_string(),
                3 => "Under Attack".to_string(),
                4 => "Information".to_string(),
                6 => "Infiltration".to_string(),
                _ => String::new(),
            },
            ParameterType::LeftOrRight => match self.int_val {
                1 => "left".to_string(),
                2 => "right".to_string(),
                3 => "normal (burst from center)".to_string(),
                _ => "unspecified".to_string(),
            },
            ParameterType::Dialog => format!("'{}'", ui_string),
            ParameterType::SkirmishWaypointPath => format!("'{}'", ui_string),
            ParameterType::Color => format!(
                " R:{} G:{} B:{} ",
                (self.int_val & 0x00ff0000) >> 16,
                (self.int_val & 0x0000ff00) >> 8,
                self.int_val & 0x000000ff
            ),
            ParameterType::Music | ParameterType::Movie => format!("'{}'", ui_string),
            ParameterType::SpecialPower => format!("Special power '{}'", ui_string),
            ParameterType::Science => format!("Science '{}'", ui_string),
            ParameterType::ScienceAvailability => format!("Science availability '{}'", ui_string),
            ParameterType::Upgrade => format!("Upgrade '{}'", ui_string),
            ParameterType::CommandbuttonAbility | ParameterType::CommandbuttonAllAbilities => {
                format!("Ability '{}'", ui_string)
            }
            ParameterType::Emoticon => format!("Emoticon '{}'", ui_string),
            ParameterType::Boundary => {
                let idx = self.int_val.rem_euclid(BORDER_COLORS_SIZE as i32) as usize;
                format!("Boundary {}", BORDER_COLORS[idx].color_name)
            }
            ParameterType::Buildable => match self.int_val {
                0 => "Buildable (Yes)",
                1 => "Buildable (Ignore_Prerequisites)",
                2 => "Buildable (No)",
                3 => "Buildable (Only_By_AI)",
                _ => "Buildable ???",
            }
            .to_string(),
            ParameterType::SurfacesAllowed => match self.int_val {
                1 => "Surfaces Allowed: Ground".to_string(),
                2 => "Surfaces Allowed: Air".to_string(),
                3 => "Surfaces Allowed: Ground or Air".to_string(),
                _ => "Surfaces Allowed: ???".to_string(),
            },
            ParameterType::ShakeIntensity => {
                if self.int_val > 0 && (self.int_val as usize) < SHAKE_INTENSITIES.len() {
                    format!("Shake Intensity: {}", SHAKE_INTENSITIES[self.int_val as usize])
                } else {
                    "Shake Intensity: ???".to_string()
                }
            }
            ParameterType::ObjectStatus => {
                if self.string_val.is_empty() {
                    "Object Status is ???".to_string()
                } else {
                    format!("Object Status is '{}'", self.string_val)
                }
            }
            ParameterType::FactionName => format!("Faction Name: {}", ui_string),
            ParameterType::ObjectTypeList => format!("'{}'", ui_string),
            ParameterType::Revealname => format!("Reveal Name: {}", ui_string),
            ParameterType::ObjectPanelFlag => format!("Object Flag: {}", ui_string),
            ParameterType::NumItems => String::new(),
        }
    }

    pub fn write_parameter<O: DataChunkOutputStream>(
        &mut self,
        chunk_writer: &mut DataChunkOutput<'_, O>,
    ) -> Result<(), DataChunkError> {
        chunk_writer.write_int(self.param_type.as_i32())?;
        if self.param_type == ParameterType::KindOfParam {
            self.string_val =
                KindOfMaskType::get_name_from_single_bit(&KINDOF_NAMES, self.int_val)
                    .unwrap_or("")
                    .to_string();
        }
        if self.param_type == ParameterType::Coord3D {
            chunk_writer.write_real(self.coord.x)?;
            chunk_writer.write_real(self.coord.y)?;
            chunk_writer.write_real(self.coord.z)?;
        } else {
            chunk_writer.write_int(self.int_val)?;
            chunk_writer.write_real(self.real_val)?;
            chunk_writer.write_ascii_string(&self.string_val)?;
        }
        Ok(())
    }

    pub fn read_parameter<Ctx>(input: &mut DataChunkInput<'_, Ctx>) -> Result<Self, DataChunkError> {
        let param_type = ParameterType::from_i32(input.read_int()?);
        let mut p = Parameter::new(param_type, 0);
        p.initialized = TRUE;
        if param_type == ParameterType::Coord3D {
            let pos = Coord3D {
                x: input.read_real()?,
                y: input.read_real()?,
                z: input.read_real()?,
            };
            p.set_coord3d(&pos);
        } else {
            p.int_val = input.read_int()?;
            p.real_val = input.read_real()?;
            p.string_val = input.read_ascii_string()?;
        }

        if param_type == ParameterType::ObjectType
            && p.string_val.starts_with("Fundamentalist")
        {
            p.string_val = format!("GLA{}", &p.string_val["Fundamentalist".len()..]);
        }
        if param_type == ParameterType::Upgrade
            && matches!(
                p.string_val.as_str(),
                "Upgrade_AmericaRangerCaptureBuilding"
                    | "Upgrade_ChinaRedguardCaptureBuilding"
                    | "Upgrade_GLARebelCaptureBuilding"
            )
        {
            p.string_val = "Upgrade_InfantryCaptureBuilding".to_string();
        }
        if param_type == ParameterType::ObjectStatus {
            for i in 0..OBJECT_STATUS_COUNT {
                if p.string_val.eq_ignore_ascii_case(OBJECT_STATUS_NAMES[i]) {
                    p.object_status.set(i as Int, TRUE as Int);
                    break;
                }
            }
        }
        if param_type == ParameterType::KindOfParam {
            if !p.string_val.is_empty() {
                for i in 0..KINDOF_COUNT {
                    if p.string_val.eq_ignore_ascii_case(KINDOF_NAMES[i]) {
                        p.int_val = i as Int;
                        break;
                    }
                    if p.string_val.eq_ignore_ascii_case("CRUSHER")
                        || p.string_val.eq_ignore_ascii_case("CRUSHABLE")
                        || p.string_val.eq_ignore_ascii_case("OVERLAPPABLE")
                    {
                        p.int_val = 0;
                        break;
                    }
                    if p.string_val.eq_ignore_ascii_case("MISSILE") {
                        p.string_val = "SMALL_MISSILE".to_string();
                        for j in 0..KINDOF_COUNT {
                            if KINDOF_NAMES[j].eq_ignore_ascii_case("SMALL_MISSILE") {
                                p.int_val = j as Int;
                                break;
                            }
                        }
                        break;
                    }
                }
            } else {
                p.string_val =
                    KindOfMaskType::get_name_from_single_bit(&KINDOF_NAMES, p.int_val)
                        .unwrap_or("")
                        .to_string();
            }
        }
        Ok(p)
    }
}

/// Single condition inside an `OrCondition` (`Condition` in `Scripts.h`).
#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub condition_type: Int,
    pub parms: Vec<Parameter>,
    pub has_warnings: Int,
    pub custom_data: Int,
    pub custom_frame: UnsignedInt,
}

impl Condition {
    pub fn new(condition_type: Int, provider: &dyn ScriptTemplateProvider) -> Self {
        let mut c = Self {
            condition_type,
            parms: Vec::new(),
            has_warnings: 0,
            custom_data: 0,
            custom_frame: 0,
        };
        c.set_condition_type(condition_type, provider);
        c
    }

    pub fn set_condition_type(
        &mut self,
        condition_type: Int,
        provider: &dyn ScriptTemplateProvider,
    ) {
        self.condition_type = condition_type;
        self.parms.clear();
        if let Some(ct) = provider.get_condition_template(condition_type) {
            let count = ct.num_parameters.max(0).min(MAX_PARMS as Int) as usize;
            for i in 0..count {
                self.parms
                    .push(Parameter::new(ct.parameters[i], 0));
            }
        }
    }

    pub fn duplicate(&self, provider: &dyn ScriptTemplateProvider) -> Self {
        let mut c = Condition::new(self.condition_type, provider);
        for i in 0..self.parms.len().min(c.parms.len()) {
            c.parms[i] = self.parms[i].clone();
        }
        c
    }

    pub fn duplicate_and_qualify(
        &self,
        qualifier: &str,
        player_template_name: &str,
        new_player_name: &str,
        provider: &dyn ScriptTemplateProvider,
    ) -> Self {
        let mut c = self.duplicate(provider);
        for p in &mut c.parms {
            p.qualify(qualifier, player_template_name, new_player_name);
        }
        c
    }

    pub fn get_ui_text(&self, provider: &dyn ScriptTemplateProvider) -> String {
        let mut out = String::new();
        if self.has_warnings != 0 {
            out.push_str("[???]");
        }
        if let Some(ct) = provider.get_condition_template(self.condition_type) {
            let count = ct.num_ui_strings.max(0).min(MAX_PARMS as Int) as usize;
            for i in 0..MAX_PARMS {
                if i < count {
                    out.push_str(&ct.ui_strings[i]);
                }
                if i < self.parms.len() {
                    out.push_str(&self.parms[i].get_ui_text());
                }
            }
        }
        out
    }
}

/// Disjunctive clause containing a list of conjunctive [`Condition`]s (`OrCondition` in `Scripts.h`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OrCondition {
    pub conditions: Vec<Condition>,
}

impl OrCondition {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn duplicate(&self, provider: &dyn ScriptTemplateProvider) -> Self {
        Self {
            conditions: self
                .conditions
                .iter()
                .map(|c| c.duplicate(provider))
                .collect(),
        }
    }

    pub fn duplicate_and_qualify(
        &self,
        qualifier: &str,
        player_template_name: &str,
        new_player_name: &str,
        provider: &dyn ScriptTemplateProvider,
    ) -> Self {
        Self {
            conditions: self
                .conditions
                .iter()
                .map(|c| {
                    c.duplicate_and_qualify(
                        qualifier,
                        player_template_name,
                        new_player_name,
                        provider,
                    )
                })
                .collect(),
        }
    }
}

/// Single action inside a [`Script`] (`ScriptAction` in `Scripts.h`).
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptAction {
    pub action_type: Int,
    pub parms: Vec<Parameter>,
    pub has_warnings: Bool,
}

impl ScriptAction {
    pub fn new(action_type: Int, provider: &dyn ScriptTemplateProvider) -> Self {
        let mut a = Self {
            action_type,
            parms: Vec::new(),
            has_warnings: FALSE,
        };
        a.set_action_type(action_type, provider);
        a
    }

    pub fn set_action_type(&mut self, action_type: Int, provider: &dyn ScriptTemplateProvider) {
        self.action_type = action_type;
        self.parms.clear();
        if let Some(at) = provider.get_action_template(action_type) {
            let count = at.num_parameters.max(0).min(MAX_PARMS as Int) as usize;
            for i in 0..count {
                self.parms.push(Parameter::new(at.parameters[i], 0));
            }
        }
    }

    pub fn duplicate(&self, provider: &dyn ScriptTemplateProvider) -> Self {
        let mut a = ScriptAction::new(self.action_type, provider);
        for i in 0..self.parms.len().min(a.parms.len()) {
            a.parms[i] = self.parms[i].clone();
        }
        a
    }

    pub fn duplicate_and_qualify(
        &self,
        qualifier: &str,
        player_template_name: &str,
        new_player_name: &str,
        provider: &dyn ScriptTemplateProvider,
    ) -> Self {
        let mut a = self.duplicate(provider);
        for p in &mut a.parms {
            p.qualify(qualifier, player_template_name, new_player_name);
        }
        a
    }

    pub fn get_ui_text(&self, provider: &dyn ScriptTemplateProvider) -> String {
        let mut out = String::new();
        if self.has_warnings != FALSE {
            out.push_str("[???]");
        }
        if let Some(at) = provider.get_action_template(self.action_type) {
            let count = at.num_ui_strings.max(0).min(MAX_PARMS as Int) as usize;
            for i in 0..MAX_PARMS {
                if i < count {
                    out.push_str(&at.ui_strings[i]);
                }
                if i < self.parms.len() {
                    out.push_str(&self.parms[i].get_ui_text());
                }
            }
        }
        out
    }
}

/// Script node containing conditions, true actions, and false actions (`Script` in `Scripts.h`).
#[derive(Debug, Clone, PartialEq)]
pub struct Script {
    pub script_name: String,
    pub comment: String,
    pub condition_comment: String,
    pub action_comment: String,
    pub delay_evaluation_seconds: Int,
    pub is_active: Bool,
    pub is_one_shot: Bool,
    pub is_subroutine: Bool,
    pub easy: Bool,
    pub normal: Bool,
    pub hard: Bool,
    pub or_conditions: Vec<OrCondition>,
    pub actions: Vec<ScriptAction>,
    pub actions_false: Vec<ScriptAction>,
    pub frame_to_evaluate_at: UnsignedInt,
    pub has_warnings: Bool,
    pub condition_team_name: String,
    pub condition_time: Real,
    pub cur_time: Real,
    pub condition_executed_count: Int,
}

impl Default for Script {
    fn default() -> Self {
        Self::new()
    }
}

impl Script {
    pub fn new() -> Self {
        Self {
            script_name: String::new(),
            comment: String::new(),
            condition_comment: String::new(),
            action_comment: String::new(),
            delay_evaluation_seconds: 0,
            is_active: TRUE,
            is_one_shot: TRUE,
            is_subroutine: FALSE,
            easy: TRUE,
            normal: TRUE,
            hard: TRUE,
            or_conditions: Vec::new(),
            actions: Vec::new(),
            actions_false: Vec::new(),
            frame_to_evaluate_at: 0,
            has_warnings: FALSE,
            condition_team_name: String::new(),
            condition_time: 0.0,
            cur_time: 0.0,
            condition_executed_count: 0,
        }
    }

    pub fn duplicate(&self, provider: &dyn ScriptTemplateProvider) -> Self {
        let mut s = Script::new();
        s.script_name = self.script_name.clone();
        s.comment = self.comment.clone();
        s.condition_comment = self.condition_comment.clone();
        s.action_comment = self.action_comment.clone();
        s.is_active = self.is_active;
        s.is_one_shot = self.is_one_shot;
        s.is_subroutine = self.is_subroutine;
        s.easy = self.easy;
        s.normal = self.normal;
        s.hard = self.hard;
        s.delay_evaluation_seconds = self.delay_evaluation_seconds;
        s.or_conditions = self
            .or_conditions
            .iter()
            .map(|o| o.duplicate(provider))
            .collect();
        s.actions = self
            .actions
            .iter()
            .map(|a| a.duplicate(provider))
            .collect();
        s.actions_false = self
            .actions_false
            .iter()
            .map(|a| a.duplicate(provider))
            .collect();
        s
    }

    pub fn duplicate_and_qualify(
        &self,
        qualifier: &str,
        player_template_name: &str,
        new_player_name: &str,
        provider: &dyn ScriptTemplateProvider,
    ) -> Self {
        let mut s = Script::new();
        s.script_name = format!("{}{}", self.script_name, qualifier);
        s.comment = self.comment.clone();
        s.condition_comment = self.condition_comment.clone();
        s.action_comment = self.action_comment.clone();
        s.is_active = self.is_active;
        s.is_one_shot = self.is_one_shot;
        s.is_subroutine = self.is_subroutine;
        s.easy = self.easy;
        s.normal = self.normal;
        s.hard = self.hard;
        s.delay_evaluation_seconds = self.delay_evaluation_seconds;
        s.or_conditions = self
            .or_conditions
            .iter()
            .map(|o| {
                o.duplicate_and_qualify(
                    qualifier,
                    player_template_name,
                    new_player_name,
                    provider,
                )
            })
            .collect();
        s.actions = self
            .actions
            .iter()
            .map(|a| {
                a.duplicate_and_qualify(
                    qualifier,
                    player_template_name,
                    new_player_name,
                    provider,
                )
            })
            .collect();
        s.actions_false = self
            .actions_false
            .iter()
            .map(|a| {
                a.duplicate_and_qualify(
                    qualifier,
                    player_template_name,
                    new_player_name,
                    provider,
                )
            })
            .collect();
        s
    }

    pub fn update_from(&mut self, src: &mut Script) {
        self.script_name = src.script_name.clone();
        self.comment = src.comment.clone();
        self.condition_comment = src.condition_comment.clone();
        self.action_comment = src.action_comment.clone();
        self.is_active = src.is_active;
        self.is_subroutine = src.is_subroutine;
        self.delay_evaluation_seconds = src.delay_evaluation_seconds;
        self.is_one_shot = src.is_one_shot;
        self.easy = src.easy;
        self.normal = src.normal;
        self.hard = src.hard;
        self.or_conditions = std::mem::take(&mut src.or_conditions);
        self.actions = std::mem::take(&mut src.actions);
        self.actions_false = std::mem::take(&mut src.actions_false);
    }

    pub fn get_ui_text(&self, provider: &dyn ScriptTemplateProvider) -> String {
        let mut out = "*** IF ***\r\n".to_string();
        let mut count = 0usize;
        for orc in &self.or_conditions {
            if count > 0 {
                out.push_str("  *** OR ***\r\n");
            }
            count = 0;
            for cond in &orc.conditions {
                if count > 0 {
                    out.push_str("    *AND* ");
                } else {
                    out.push_str("    ");
                }
                out.push_str(&cond.get_ui_text(provider));
                out.push_str("\r\n");
                count += 1;
            }
        }
        out.push_str("*** THEN ***\r\n");
        for action in &self.actions {
            out.push_str("  ");
            out.push_str(&action.get_ui_text(provider));
            out.push_str("\r\n");
        }
        if !self.actions_false.is_empty() {
            out.push_str("*** ELSE ***\r\n");
            for action in &self.actions_false {
                out.push_str("  ");
                out.push_str(&action.get_ui_text(provider));
                out.push_str("\r\n");
            }
        }
        out
    }
}

impl Snapshot for Script {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version = 1;
        xfer.xfer_version(&mut version, 1)?;
        xfer.xfer_bool(&mut self.is_active)?;
        if version >= 2 {
            xfer.xfer_unsigned_int(&mut self.frame_to_evaluate_at)?;
        }
        Ok(())
    }

    fn load_post_process(&mut self) {}
}

fn write_condition_data_chunk<O: DataChunkOutputStream>(
    chunk_writer: &mut DataChunkOutput<'_, O>,
    condition: &mut Condition,
    nkg: &mut NameKeyGenerator,
    provider: &dyn ScriptTemplateProvider,
) -> Result<(), DataChunkError> {
    chunk_writer.open_data_chunk("Condition", K_SCRIPT_CONDITION_VERSION_4)?;
    chunk_writer.write_int(condition.condition_type)?;
    let key = provider
        .get_condition_template(condition.condition_type)
        .map(|ct| ct.internal_name_key)
        .unwrap_or_else(|| nkg.name_to_key("Bogus"));
    chunk_writer.write_name_key(key, nkg)?;
    chunk_writer.write_int(condition.parms.len() as Int)?;
    for parm in &mut condition.parms {
        parm.write_parameter(chunk_writer)?;
    }
    chunk_writer.close_data_chunk()
}

fn write_or_condition_data_chunk<O: DataChunkOutputStream>(
    chunk_writer: &mut DataChunkOutput<'_, O>,
    or_condition: &mut OrCondition,
    nkg: &mut NameKeyGenerator,
    provider: &dyn ScriptTemplateProvider,
) -> Result<(), DataChunkError> {
    chunk_writer.open_data_chunk("OrCondition", K_SCRIPT_OR_CONDITION_DATA_VERSION_1)?;
    for condition in &mut or_condition.conditions {
        write_condition_data_chunk(chunk_writer, condition, nkg, provider)?;
    }
    chunk_writer.close_data_chunk()
}

fn write_action_data_chunk<O: DataChunkOutputStream>(
    chunk_writer: &mut DataChunkOutput<'_, O>,
    label: &str,
    action: &mut ScriptAction,
    nkg: &mut NameKeyGenerator,
    provider: &dyn ScriptTemplateProvider,
) -> Result<(), DataChunkError> {
    chunk_writer.open_data_chunk(label, K_SCRIPT_ACTION_VERSION_2)?;
    chunk_writer.write_int(action.action_type)?;
    let key = provider
        .get_action_template(action.action_type)
        .map(|at| at.internal_name_key)
        .unwrap_or_else(|| nkg.name_to_key("Bogus"));
    chunk_writer.write_name_key(key, nkg)?;
    chunk_writer.write_int(action.parms.len() as Int)?;
    for parm in &mut action.parms {
        parm.write_parameter(chunk_writer)?;
    }
    chunk_writer.close_data_chunk()
}

fn write_script_data_chunk<O: DataChunkOutputStream>(
    chunk_writer: &mut DataChunkOutput<'_, O>,
    script: &mut Script,
    nkg: &mut NameKeyGenerator,
    provider: &dyn ScriptTemplateProvider,
) -> Result<(), DataChunkError> {
    chunk_writer.open_data_chunk("Script", K_SCRIPT_DATA_VERSION_2)?;
    chunk_writer.write_ascii_string(&script.script_name)?;
    chunk_writer.write_ascii_string(&script.comment)?;
    chunk_writer.write_ascii_string(&script.condition_comment)?;
    chunk_writer.write_ascii_string(&script.action_comment)?;
    chunk_writer.write_byte(if script.is_active != FALSE { 1 } else { 0 })?;
    chunk_writer.write_byte(if script.is_one_shot != FALSE { 1 } else { 0 })?;
    chunk_writer.write_byte(if script.easy != FALSE { 1 } else { 0 })?;
    chunk_writer.write_byte(if script.normal != FALSE { 1 } else { 0 })?;
    chunk_writer.write_byte(if script.hard != FALSE { 1 } else { 0 })?;
    chunk_writer.write_byte(if script.is_subroutine != FALSE { 1 } else { 0 })?;
    chunk_writer.write_int(script.delay_evaluation_seconds)?;
    for or_condition in &mut script.or_conditions {
        write_or_condition_data_chunk(chunk_writer, or_condition, nkg, provider)?;
    }
    for action in &mut script.actions {
        write_action_data_chunk(
            chunk_writer,
            "ScriptAction",
            action,
            nkg,
            provider,
        )?;
    }
    for action in &mut script.actions_false {
        write_action_data_chunk(
            chunk_writer,
            "ScriptActionFalse",
            action,
            nkg,
            provider,
        )?;
    }
    chunk_writer.close_data_chunk()
}

fn parse_condition_chunk<Ctx>(
    input: &mut DataChunkInput<'_, Ctx>,
    version: UnsignedShort,
    nkg: &mut NameKeyGenerator,
    provider: &dyn ScriptTemplateProvider,
) -> Result<Condition, DataChunkError> {
    let mut condition_type = input.read_int()?;
    if version >= K_SCRIPT_CONDITION_VERSION_4 {
        let key = input.read_name_key(nkg)?;
        let mut matched = false;
        if let Some(ct) = provider.get_condition_template(condition_type) {
            if ct.internal_name_key == key {
                matched = true;
            }
        }
        if !matched {
            for i in 0..CONDITION_NUM_ITEMS {
                if let Some(ct) = provider.get_condition_template(i) {
                    if ct.internal_name_key == key {
                        condition_type = i;
                        matched = true;
                        break;
                    }
                }
            }
        }
        if !matched {
            condition_type = 0;
        }
    }
    let num_parms = input.read_int()?.max(0).min(MAX_PARMS as Int) as usize;
    let mut parms = Vec::with_capacity(num_parms);
    for _ in 0..num_parms {
        parms.push(Parameter::read_parameter(input)?);
    }
    if version < K_SCRIPT_CONDITION_VERSION_2
        && matches!(condition_type, 7 | 17 | 18 | 39 | 40 | 41 | 42)
    {
        parms.push(Parameter::new(ParameterType::SurfacesAllowed, 3));
    }
    if condition_type == 84 && parms.len() == 1 {
        let old = parms.pop().unwrap();
        let mut side = Parameter::new(ParameterType::Side, 0);
        side.string_val = THIS_PLAYER.to_string();
        parms = vec![side, old];
    }
    let mut condition = Condition::new(condition_type, provider);
    if provider
        .get_condition_template(condition_type)
        .is_none_or(|ct| ct.num_parameters != parms.len() as Int)
    {
        condition.condition_type = 0;
        condition.parms.clear();
    } else {
        for i in 0..parms.len().min(condition.parms.len()) {
            condition.parms[i] = parms[i].clone();
        }
    }
    Ok(condition)
}

fn parse_action_chunk<Ctx>(
    input: &mut DataChunkInput<'_, Ctx>,
    version: UnsignedShort,
    nkg: &mut NameKeyGenerator,
    provider: &dyn ScriptTemplateProvider,
) -> Result<ScriptAction, DataChunkError> {
    let mut action_type = input.read_int()?;
    if version >= K_SCRIPT_ACTION_VERSION_2 {
        let key = input.read_name_key(nkg)?;
        let mut matched = false;
        if let Some(at) = provider.get_action_template(action_type) {
            if at.internal_name_key == key {
                matched = true;
            }
        }
        if !matched {
            for i in 0..ACTION_NUM_ITEMS {
                if let Some(at) = provider.get_action_template(i) {
                    if at.internal_name_key == key {
                        action_type = i;
                        matched = true;
                        break;
                    }
                }
            }
        }
        if !matched {
            action_type = 5;
        }
    }
    let num_parms = input.read_int()?.max(0).min(MAX_PARMS as Int) as usize;
    let mut parms = Vec::with_capacity(num_parms);
    for _ in 0..num_parms {
        parms.push(Parameter::read_parameter(input)?);
    }

    match action_type {
        251 if parms.len() == 1 => {
            let old = parms.pop().unwrap();
            let mut side = Parameter::new(ParameterType::Side, 0);
            side.string_val = THIS_PLAYER.to_string();
            parms = vec![side, old];
        }
        36 if parms.len() == 2 => parms.push(Parameter::new(ParameterType::Boolean, 1)),
        250 if parms.len() == 1 => {
            if parms[0].int_val != 0 {
                action_type = 254;
            }
            parms.clear();
        }
        45 | 46 if parms.len() >= 2 && parms[1].param_type == ParameterType::Int => {
            parms[1] = Parameter::new(ParameterType::AiMood, parms[1].int_val);
        }
        90 | 145 if parms.len() == 2 => parms.push(Parameter::new(ParameterType::Side, 0)),
        102 | 146 | 229 | 230 if parms.is_empty() => {
            parms.push(Parameter::new(ParameterType::Side, 0));
        }
        83 if parms.len() == 1 => parms.push(Parameter::new(ParameterType::Boolean, 1)),
        26 | 27 if parms.len() == 1 => {
            parms.push(Parameter::new(ParameterType::Percent, 0));
            parms.push(Parameter::new(ParameterType::Percent, 0));
        }
        14 | 17 | 181 if parms.len() == 3 => {
            parms.push(Parameter::new(ParameterType::Real, 0));
            parms.push(Parameter::new(ParameterType::Real, 0));
        }
        18 | 19 | 117 | 118 if parms.len() == 2 => {
            parms.push(Parameter::new(ParameterType::Real, 0));
            parms.push(Parameter::new(ParameterType::Real, 0));
        }
        206 if parms.len() == 2 => {
            parms.push(Parameter::new(ParameterType::Real, 0));
            parms.push(Parameter::new(ParameterType::Real, 0));
            parms.push(Parameter::new(ParameterType::Boolean, 0));
        }
        206 if parms.len() == 4 => {
            parms.push(Parameter::new(ParameterType::Boolean, 0));
        }
        _ => {}
    }

    let mut action = ScriptAction::new(action_type, provider);
    if provider
        .get_action_template(action_type)
        .is_none_or(|at| at.num_parameters != parms.len() as Int)
    {
        action.action_type = 5;
        action.parms.clear();
    } else {
        for i in 0..parms.len().min(action.parms.len()) {
            action.parms[i] = parms[i].clone();
        }
    }
    Ok(action)
}

fn parse_script_chunk<Ctx>(
    input: &mut DataChunkInput<'_, Ctx>,
    version: UnsignedShort,
    nkg: &mut NameKeyGenerator,
    provider: &dyn ScriptTemplateProvider,
) -> Result<Script, DataChunkError> {
    let mut script = Script::new();
    script.script_name = input.read_ascii_string()?;
    script.comment = input.read_ascii_string()?;
    script.condition_comment = input.read_ascii_string()?;
    script.action_comment = input.read_ascii_string()?;
    script.is_active = if input.read_byte()? != 0 { TRUE } else { FALSE };
    script.is_one_shot = if input.read_byte()? != 0 { TRUE } else { FALSE };
    script.easy = if input.read_byte()? != 0 { TRUE } else { FALSE };
    script.normal = if input.read_byte()? != 0 { TRUE } else { FALSE };
    script.hard = if input.read_byte()? != 0 { TRUE } else { FALSE };
    script.is_subroutine = if input.read_byte()? != 0 { TRUE } else { FALSE };
    if version >= K_SCRIPT_DATA_VERSION_2 {
        script.delay_evaluation_seconds = input.read_int()?;
    }
    while !input.at_end_of_chunk() {
        let (label, child_version) = input.open_data_chunk()?;
        match label.as_str() {
            "OrCondition" => {
                let mut or_condition = OrCondition::new();
                while !input.at_end_of_chunk() {
                    let (condition_label, condition_version) = input.open_data_chunk()?;
                    if condition_label == "Condition" {
                        or_condition.conditions.push(parse_condition_chunk(
                            input,
                            condition_version,
                            nkg,
                            provider,
                        )?);
                    }
                    input.close_data_chunk()?;
                }
                script.or_conditions.push(or_condition);
            }
            "ScriptAction" => {
                let action = parse_action_chunk(input, child_version, nkg, provider)?;
                script.actions.push(action);
            }
            "ScriptActionFalse" => {
                let action = parse_action_chunk(input, child_version, nkg, provider)?;
                script.actions_false.push(action);
            }
            _ => {}
        }
        input.close_data_chunk()?;
    }
    Ok(script)
}

fn parse_script_group_chunk<Ctx>(
    input: &mut DataChunkInput<'_, Ctx>,
    version: UnsignedShort,
    nkg: &mut NameKeyGenerator,
    provider: &dyn ScriptTemplateProvider,
) -> Result<ScriptGroup, DataChunkError> {
    let mut group = ScriptGroup::new();
    group.group_name = input.read_ascii_string()?;
    group.is_group_active = if input.read_byte()? != 0 { TRUE } else { FALSE };
    if version >= K_SCRIPT_GROUP_DATA_VERSION_2 {
        group.is_group_subroutine = if input.read_byte()? != 0 { TRUE } else { FALSE };
    }
    while !input.at_end_of_chunk() {
        let (label, script_version) = input.open_data_chunk()?;
        if label == "Script" {
            group.scripts.push(parse_script_chunk(
                input,
                script_version,
                nkg,
                provider,
            )?);
        }
        input.close_data_chunk()?;
    }
    Ok(group)
}

/// Named group of scripts (`ScriptGroup` in `Scripts.h`).
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptGroup {
    pub scripts: Vec<Script>,
    pub group_name: String,
    pub is_group_active: Bool,
    pub is_group_subroutine: Bool,
    pub has_warnings: Bool,
}

impl Default for ScriptGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptGroup {
    pub fn new() -> Self {
        Self {
            scripts: Vec::new(),
            group_name: String::new(),
            is_group_active: TRUE,
            is_group_subroutine: FALSE,
            has_warnings: FALSE,
        }
    }

    pub fn add_script(&mut self, scr: Script, ndx: Int) {
        let mut remaining = ndx.max(0);
        let mut cur = 0usize;
        while remaining > 0 && cur < self.scripts.len() {
            cur += 1;
            remaining -= 1;
        }
        self.scripts.insert(cur, scr);
    }

    pub fn delete_script(&mut self, index: usize) {
        if index < self.scripts.len() {
            self.scripts.remove(index);
        }
    }

    pub fn duplicate(&self, provider: &dyn ScriptTemplateProvider) -> Self {
        Self {
            scripts: self
                .scripts
                .iter()
                .map(|s| s.duplicate(provider))
                .collect(),
            group_name: self.group_name.clone(),
            is_group_active: self.is_group_active,
            is_group_subroutine: self.is_group_subroutine,
            has_warnings: FALSE,
        }
    }

    pub fn duplicate_and_qualify(
        &self,
        qualifier: &str,
        player_template_name: &str,
        new_player_name: &str,
        provider: &dyn ScriptTemplateProvider,
    ) -> Self {
        Self {
            scripts: self
                .scripts
                .iter()
                .map(|s| {
                    s.duplicate_and_qualify(
                        qualifier,
                        player_template_name,
                        new_player_name,
                        provider,
                    )
                })
                .collect(),
            group_name: format!("{}{}", self.group_name, qualifier),
            is_group_active: self.is_group_active,
            is_group_subroutine: self.is_group_subroutine,
            has_warnings: FALSE,
        }
    }
}

impl Snapshot for ScriptGroup {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version = 2;
        xfer.xfer_version(&mut version, 2)?;
        if version >= 2 {
            xfer.xfer_bool(&mut self.is_group_active)?;
        }
        let mut script_count = self.scripts.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut script_count)?;
        for script in &mut self.scripts {
            if script_count == 0 {
                break;
            }
            xfer.xfer_snapshot(script)?;
            script_count -= 1;
        }
        while script_count > 0 {
            let mut dummy = Script::new();
            xfer.xfer_snapshot(&mut dummy)?;
            script_count -= 1;
        }
        Ok(())
    }

    fn load_post_process(&mut self) {}
}

/// Per-player list of top-level [`Script`]s and [`ScriptGroup`]s (`ScriptList` in `Scripts.h`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScriptList {
    pub groups: Vec<ScriptGroup>,
    pub scripts: Vec<Script>,
}

impl ScriptList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_group(&mut self, grp: ScriptGroup, ndx: Int) {
        let mut remaining = ndx.max(0);
        let mut cur = 0usize;
        while remaining > 0 && cur < self.groups.len() {
            cur += 1;
            remaining -= 1;
        }
        self.groups.insert(cur, grp);
    }

    pub fn add_script(&mut self, scr: Script, ndx: Int) {
        let mut remaining = ndx.max(0);
        let mut cur = 0usize;
        while remaining > 0 && cur < self.scripts.len() {
            cur += 1;
            remaining -= 1;
        }
        self.scripts.insert(cur, scr);
    }

    pub fn delete_script(&mut self, index: usize) {
        if index < self.scripts.len() {
            self.scripts.remove(index);
        }
    }

    pub fn delete_group(&mut self, index: usize) {
        if index < self.groups.len() {
            self.groups.remove(index);
        }
    }

    pub fn duplicate(&self, provider: &dyn ScriptTemplateProvider) -> Self {
        Self {
            groups: self
                .groups
                .iter()
                .map(|g| g.duplicate(provider))
                .collect(),
            scripts: self
                .scripts
                .iter()
                .map(|s| s.duplicate(provider))
                .collect(),
        }
    }

    pub fn duplicate_and_qualify(
        &self,
        qualifier: &str,
        player_template_name: &str,
        new_player_name: &str,
        provider: &dyn ScriptTemplateProvider,
    ) -> Self {
        Self {
            groups: self
                .groups
                .iter()
                .map(|g| {
                    g.duplicate_and_qualify(
                        qualifier,
                        player_template_name,
                        new_player_name,
                        provider,
                    )
                })
                .collect(),
            scripts: self
                .scripts
                .iter()
                .map(|s| {
                    s.duplicate_and_qualify(
                        qualifier,
                        player_template_name,
                        new_player_name,
                        provider,
                    )
                })
                .collect(),
        }
    }

    pub fn write_script_list_data_chunk<O: DataChunkOutputStream>(
        &mut self,
        chunk_writer: &mut DataChunkOutput<'_, O>,
        nkg: &mut NameKeyGenerator,
        provider: &dyn ScriptTemplateProvider,
    ) -> Result<(), DataChunkError> {
        for script in &mut self.scripts {
            write_script_data_chunk(chunk_writer, script, nkg, provider)?;
        }
        for group in &mut self.groups {
            chunk_writer.open_data_chunk("ScriptGroup", K_SCRIPT_GROUP_DATA_VERSION_2)?;
            chunk_writer.write_ascii_string(&group.group_name)?;
            chunk_writer.write_byte(if group.is_group_active != FALSE { 1 } else { 0 })?;
            chunk_writer.write_byte(if group.is_group_subroutine != FALSE { 1 } else { 0 })?;
            for script in &mut group.scripts {
                write_script_data_chunk(chunk_writer, script, nkg, provider)?;
            }
            chunk_writer.close_data_chunk()?;
        }
        Ok(())
    }

    pub fn write_scripts_data_chunk<O: DataChunkOutputStream>(
        chunk_writer: &mut DataChunkOutput<'_, O>,
        script_lists: &mut [ScriptList],
        nkg: &mut NameKeyGenerator,
        provider: &dyn ScriptTemplateProvider,
    ) -> Result<(), DataChunkError> {
        chunk_writer.open_data_chunk("PlayerScriptsList", K_SCRIPTS_DATA_VERSION_1)?;
        for list in script_lists {
            chunk_writer.open_data_chunk("ScriptList", K_SCRIPT_LIST_DATA_VERSION_1)?;
            list.write_script_list_data_chunk(chunk_writer, nkg, provider)?;
            chunk_writer.close_data_chunk()?;
        }
        chunk_writer.close_data_chunk()
    }

    /// The `ScriptList` chunks of an open `PlayerScriptsList` chunk, which a map
    /// nests inside its `SidesList` chunk.
    pub fn read_player_scripts_chunk<Ctx>(
        input: &mut DataChunkInput<'_, Ctx>,
        lists: &mut Vec<ScriptList>,
        nkg: &mut NameKeyGenerator,
        provider: &dyn ScriptTemplateProvider,
    ) -> Result<(), DataChunkError> {
        while !input.at_end_of_chunk() {
            let (label, _) = input.open_data_chunk()?;
            if label == "ScriptList" {
                if lists.len() >= MAX_PLAYER_COUNT {
                    return Err(DataChunkError::CorruptFormat);
                }
                let mut list = ScriptList::new();
                while !input.at_end_of_chunk() {
                    let (child_label, child_version) = input.open_data_chunk()?;
                    match child_label.as_str() {
                        "Script" => {
                            let script = parse_script_chunk(input, child_version, nkg, provider)?;
                            list.scripts.push(script);
                        }
                        "ScriptGroup" => {
                            let group =
                                parse_script_group_chunk(input, child_version, nkg, provider)?;
                            list.groups.push(group);
                        }
                        _ => {}
                    }
                    input.close_data_chunk()?;
                }
                lists.push(list);
            }
            input.close_data_chunk()?;
        }
        Ok(())
    }

    pub fn parse_scripts_data_chunk<S: ChunkInputStream>(
        stream: &mut S,
        nkg: &mut NameKeyGenerator,
        provider: &dyn ScriptTemplateProvider,
    ) -> Result<Vec<ScriptList>, DataChunkError> {
        let mut input: DataChunkInput<'_, ()> = DataChunkInput::new(stream);
        if !input.is_valid_file_type() {
            return Err(DataChunkError::InvalidHeader);
        }
        input.reset()?;
        let mut lists = Vec::new();
        while !input.at_end_of_file() {
            let (root_label, root_version) = input.open_data_chunk()?;
            if root_label == "PlayerScriptsList" {
                if root_version != K_SCRIPTS_DATA_VERSION_1 {
                    // Continue supporting the same nested layout under legacy root versions.
                }
                Self::read_player_scripts_chunk(&mut input, &mut lists, nkg, provider)?;
            }
            input.close_data_chunk()?;
        }
        Ok(lists)
    }
}

impl Snapshot for ScriptList {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut script_count = self.scripts.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut script_count)?;
        for script in &mut self.scripts {
            if script_count == 0 {
                break;
            }
            xfer.xfer_snapshot(script)?;
            script_count -= 1;
        }
        while script_count > 0 {
            let mut dummy = Script::new();
            xfer.xfer_snapshot(&mut dummy)?;
            script_count -= 1;
        }
        let mut group_count = self.groups.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut group_count)?;
        for group in &mut self.groups {
            if group_count == 0 {
                break;
            }
            xfer.xfer_snapshot(group)?;
            group_count -= 1;
        }
        while group_count > 0 {
            let mut dummy = ScriptGroup::new();
            xfer.xfer_snapshot(&mut dummy)?;
            group_count -= 1;
        }
        Ok(())
    }

    fn load_post_process(&mut self) {}
}
