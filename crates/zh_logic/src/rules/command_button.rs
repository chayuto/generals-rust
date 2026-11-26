//! `CommandButton` and `CommandSet` (`GameClient/ControlBar.h`): the parts the
//! simulation reads. The `CommandButton` / `CommandSet` INI parser belongs to
//! the unported control bar; callers fill the store.

use crate::object::weapon_set::WeaponSlotType;
use zh_common::types::{Int, UnsignedInt};

/// `MAX_COMMANDS_PER_SET`.
pub const MAX_COMMANDS_PER_SET: usize = 18;
/// `CONTEXTMODE_COMMAND` (`CommandOption`).
pub const CONTEXTMODE_COMMAND: UnsignedInt = 0x0000_0200;
/// `COMMAND_FIRED_BY_SCRIPT` (`CommandOption`).
pub const COMMAND_FIRED_BY_SCRIPT: UnsignedInt = 0x0004_0000;

/// `GUICommandType`, without the `ALLOW_SURRENDER` entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum GuiCommandType {
    #[default]
    None = 0,
    DozerConstruct = 1,
    DozerConstructCancel = 2,
    UnitBuild = 3,
    CancelUnitBuild = 4,
    PlayerUpgrade = 5,
    ObjectUpgrade = 6,
    CancelUpgrade = 7,
    AttackMove = 8,
    Guard = 9,
    GuardWithoutPursuit = 10,
    GuardFlyingUnitsOnly = 11,
    Stop = 12,
    Waypoints = 13,
    ExitContainer = 14,
    Evacuate = 15,
    ExecuteRailedTransport = 16,
    BeaconDelete = 17,
    SetRallyPoint = 18,
    Sell = 19,
    FireWeapon = 20,
    SpecialPower = 21,
    PurchaseScience = 22,
    HackInternet = 23,
    ToggleOvercharge = 24,
    CombatDrop = 25,
    SwitchWeapon = 26,
    HijackVehicle = 27,
    ConvertToCarBomb = 28,
    SabotageBuilding = 29,
    PlaceBeacon = 30,
    SpecialPowerFromShortcut = 31,
    SpecialPowerConstruct = 32,
    SpecialPowerConstructFromShortcut = 33,
    SelectAllUnitsOfType = 34,
}

#[derive(Debug, Clone)]
pub struct CommandButton {
    pub name: String,
    pub command: GuiCommandType,
    pub options: UnsignedInt,
    /// `SpecialPowerTemplate::getID`; `None` without a `SpecialPower` entry.
    pub special_power_id: Option<UnsignedInt>,
    pub weapon_slot: WeaponSlotType,
}

impl CommandButton {
    pub fn new(name: &str, command: GuiCommandType) -> Self {
        Self {
            name: name.to_owned(),
            command,
            options: 0,
            special_power_id: None,
            weapon_slot: WeaponSlotType::Primary,
        }
    }
}

/// Index of a button in the [`CommandStore`].
pub type CommandButtonId = usize;

#[derive(Debug, Clone)]
pub struct CommandSet {
    pub name: String,
    pub buttons: [Option<CommandButtonId>; MAX_COMMANDS_PER_SET],
}

/// The buttons and sets `TheControlBar` owns.
#[derive(Debug, Clone, Default)]
pub struct CommandStore {
    buttons: Vec<CommandButton>,
    sets: Vec<CommandSet>,
}

impl CommandStore {
    pub fn add_button(&mut self, button: CommandButton) -> CommandButtonId {
        self.buttons.push(button);
        self.buttons.len() - 1
    }

    pub fn add_set(&mut self, name: &str, buttons: &[(usize, CommandButtonId)]) {
        let mut set = CommandSet {
            name: name.to_owned(),
            buttons: [None; MAX_COMMANDS_PER_SET],
        };
        for &(slot, id) in buttons {
            set.buttons[slot] = Some(id);
        }
        self.sets.push(set);
    }

    pub fn button_count(&self) -> usize {
        self.buttons.len()
    }

    pub fn button(&self, id: CommandButtonId) -> Option<&CommandButton> {
        self.buttons.get(id)
    }

    /// `ControlBar::findCommandSet`.
    pub fn find_command_set(&self, name: &str) -> Option<&CommandSet> {
        self.sets.iter().find(|s| s.name == name)
    }
}

/// Key of `GameLogic::m_controlBarOverrides`.
pub type ControlBarOverrideKey = (String, Int);
