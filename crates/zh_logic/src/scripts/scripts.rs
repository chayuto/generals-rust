//! `PKT-434`: Zero Hour `Scripts` data structures (`ScriptList`, `ScriptGroup`, `Script`,
//! `OrCondition`, `Condition`, `ScriptAction`, `Parameter`, `Template`, `BorderColor`, `ShellHooks`).
//!
//! Reference C++ source:
//! - `GeneralsMD/Code/GameEngine/Include/GameLogic/Scripts.h`
//! - `GeneralsMD/Code/GameEngine/Source/GameLogic/ScriptEngine/Scripts.cpp`
//! - `GeneralsMD/Code/GameEngine/Include/Common/BorderColors.h`
//! - `Core/GameEngine/Include/GameClient/ShellHooks.h`

#![allow(unused_variables, dead_code)]

use crate::rules::object_status_types::ObjectStatusMaskType;
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
        todo!("PKT-434")
    }

    pub fn get_ui_text(&self) -> String {
        todo!("PKT-434")
    }

    pub fn write_parameter<O: DataChunkOutputStream>(
        &mut self,
        chunk_writer: &mut DataChunkOutput<'_, O>,
    ) -> Result<(), DataChunkError> {
        todo!("PKT-434")
    }

    pub fn read_parameter<Ctx>(input: &mut DataChunkInput<'_, Ctx>) -> Result<Self, DataChunkError> {
        todo!("PKT-434")
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
        todo!("PKT-434")
    }

    pub fn set_condition_type(
        &mut self,
        condition_type: Int,
        provider: &dyn ScriptTemplateProvider,
    ) {
        todo!("PKT-434")
    }

    pub fn duplicate(&self, provider: &dyn ScriptTemplateProvider) -> Self {
        todo!("PKT-434")
    }

    pub fn duplicate_and_qualify(
        &self,
        qualifier: &str,
        player_template_name: &str,
        new_player_name: &str,
        provider: &dyn ScriptTemplateProvider,
    ) -> Self {
        todo!("PKT-434")
    }

    pub fn get_ui_text(&self, provider: &dyn ScriptTemplateProvider) -> String {
        todo!("PKT-434")
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
        todo!("PKT-434")
    }

    pub fn duplicate_and_qualify(
        &self,
        qualifier: &str,
        player_template_name: &str,
        new_player_name: &str,
        provider: &dyn ScriptTemplateProvider,
    ) -> Self {
        todo!("PKT-434")
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
        todo!("PKT-434")
    }

    pub fn set_action_type(&mut self, action_type: Int, provider: &dyn ScriptTemplateProvider) {
        todo!("PKT-434")
    }

    pub fn duplicate(&self, provider: &dyn ScriptTemplateProvider) -> Self {
        todo!("PKT-434")
    }

    pub fn duplicate_and_qualify(
        &self,
        qualifier: &str,
        player_template_name: &str,
        new_player_name: &str,
        provider: &dyn ScriptTemplateProvider,
    ) -> Self {
        todo!("PKT-434")
    }

    pub fn get_ui_text(&self, provider: &dyn ScriptTemplateProvider) -> String {
        todo!("PKT-434")
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
        todo!("PKT-434")
    }

    pub fn duplicate_and_qualify(
        &self,
        qualifier: &str,
        player_template_name: &str,
        new_player_name: &str,
        provider: &dyn ScriptTemplateProvider,
    ) -> Self {
        todo!("PKT-434")
    }

    pub fn update_from(&mut self, src: &mut Script) {
        todo!("PKT-434")
    }

    pub fn get_ui_text(&self, provider: &dyn ScriptTemplateProvider) -> String {
        todo!("PKT-434")
    }
}

impl Snapshot for Script {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let _ = xfer;
        todo!("PKT-434")
    }

    fn load_post_process(&mut self) {}
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
        todo!("PKT-434")
    }

    pub fn delete_script(&mut self, index: usize) {
        todo!("PKT-434")
    }

    pub fn duplicate(&self, provider: &dyn ScriptTemplateProvider) -> Self {
        todo!("PKT-434")
    }

    pub fn duplicate_and_qualify(
        &self,
        qualifier: &str,
        player_template_name: &str,
        new_player_name: &str,
        provider: &dyn ScriptTemplateProvider,
    ) -> Self {
        todo!("PKT-434")
    }
}

impl Snapshot for ScriptGroup {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let _ = xfer;
        todo!("PKT-434")
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
        todo!("PKT-434")
    }

    pub fn add_script(&mut self, scr: Script, ndx: Int) {
        todo!("PKT-434")
    }

    pub fn delete_script(&mut self, index: usize) {
        todo!("PKT-434")
    }

    pub fn delete_group(&mut self, index: usize) {
        todo!("PKT-434")
    }

    pub fn duplicate(&self, provider: &dyn ScriptTemplateProvider) -> Self {
        todo!("PKT-434")
    }

    pub fn duplicate_and_qualify(
        &self,
        qualifier: &str,
        player_template_name: &str,
        new_player_name: &str,
        provider: &dyn ScriptTemplateProvider,
    ) -> Self {
        todo!("PKT-434")
    }

    pub fn write_script_list_data_chunk<O: DataChunkOutputStream>(
        &mut self,
        chunk_writer: &mut DataChunkOutput<'_, O>,
        nkg: &mut NameKeyGenerator,
        provider: &dyn ScriptTemplateProvider,
    ) -> Result<(), DataChunkError> {
        todo!("PKT-434")
    }

    pub fn write_scripts_data_chunk<O: DataChunkOutputStream>(
        chunk_writer: &mut DataChunkOutput<'_, O>,
        script_lists: &mut [ScriptList],
        nkg: &mut NameKeyGenerator,
        provider: &dyn ScriptTemplateProvider,
    ) -> Result<(), DataChunkError> {
        todo!("PKT-434")
    }

    pub fn parse_scripts_data_chunk<S: ChunkInputStream>(
        stream: &mut S,
        nkg: &mut NameKeyGenerator,
        provider: &dyn ScriptTemplateProvider,
    ) -> Result<Vec<ScriptList>, DataChunkError> {
        todo!("PKT-434")
    }
}

impl Snapshot for ScriptList {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let _ = xfer;
        todo!("PKT-434")
    }

    fn load_post_process(&mut self) {}
}
