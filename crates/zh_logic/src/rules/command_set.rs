//! The part of `CommandSet` / `CommandButton` (`ControlBar.h`) that the simulation reads:
//! which templates and upgrades a command set offers. `BuildAssistant::isPossibleToMakeUnit`
//! and `Object::canProduceUpgrade` consult it. The INI parser of the control bar is not
//! ported: the store is filled directly.

use std::collections::BTreeMap;

/// `GUICommandType`, reduced to the distinction the build checks make.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommandButtonType {
    /// `GUI_COMMAND_UNIT_BUILD`
    UnitBuild,
    /// `GUI_COMMAND_DOZER_CONSTRUCT`
    DozerConstruct,
    /// Any other command.
    #[default]
    Other,
}

/// One button: its command, `Object` template name and `Upgrade` name (empty if absent).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandButton {
    pub command: CommandButtonType,
    pub object: String,
    pub upgrade: String,
}

/// Command sets by name (`ControlBar::findCommandSet`, case sensitive).
#[derive(Debug, Clone, Default)]
pub struct CommandSetStore {
    sets: BTreeMap<String, Vec<CommandButton>>,
}

impl CommandSetStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: &str, buttons: Vec<CommandButton>) {
        self.sets.insert(name.to_owned(), buttons);
    }

    pub fn find(&self, name: &str) -> Option<&[CommandButton]> {
        self.sets.get(name).map(Vec::as_slice)
    }
}
