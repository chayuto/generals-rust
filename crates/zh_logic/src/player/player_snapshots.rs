//! `PKT-404` the snapshots `Player::xfer` calls into: the upgrade list
//! (`Upgrade.cpp`), and `m_ai` and `m_tunnelSystem`, which the sim keeps outside
//! [`Player`].
//!
//! Golden test: `tests/golden_player_xfer.rs`.

use std::collections::BTreeMap;

use zh_common::types::{Int, UnsignedShort, XferVersion};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

use crate::ai::ai_player::AiPlayer;
use crate::object::thing_factory::ThingFactory;
use crate::player::player::Player;
use crate::player::seams::{PlayerIndex, PlayerPart};
use crate::player::team::TeamFactory;
use crate::rules::upgrade::UpgradeMaskType;
use crate::sim::TunnelTrackerState;

/// `UpgradeStatusType`.
pub const UPGRADE_STATUS_INVALID: Int = 0;
pub const UPGRADE_STATUS_IN_PRODUCTION: Int = 1;
pub const UPGRADE_STATUS_COMPLETE: Int = 2;

/// One `Upgrade` of `m_upgradeList`.
#[derive(Debug, Clone)]
pub struct PlayerUpgrade {
    /// The template, by `UpgradeTemplate::getUpgradeName`.
    pub name: String,
    pub mask: UpgradeMaskType,
    /// `UpgradeStatusType`.
    pub status: Int,
}

impl Player {
    /// The list and mask part of `addUpgrade`: a new entry goes to the head, an
    /// existing one only changes status. `onUpgradeCompleted` is the caller's.
    pub fn add_upgrade(&mut self, name: &str, mask: &UpgradeMaskType, status: Int) {
        let at = match self.upgrades.iter().position(|u| u.name == name) {
            Some(at) => at,
            None => {
                self.upgrades.insert(
                    0,
                    PlayerUpgrade {
                        name: name.to_owned(),
                        mask: mask.clone(),
                        status: UPGRADE_STATUS_INVALID,
                    },
                );
                0
            }
        };
        self.upgrades[at].status = status;
        let none = UpgradeMaskType::default();
        if status == UPGRADE_STATUS_IN_PRODUCTION {
            self.upgrades_in_progress.clear_and_set(&none, mask);
        } else if status == UPGRADE_STATUS_COMPLETE {
            self.upgrades_in_progress.clear_and_set(mask, &none);
            self.upgrades_completed.clear_and_set(&none, mask);
        }
    }

    /// `removeUpgrade`: nothing happens to the masks without a list entry.
    /// `onUpgradeRemoved` is not ported.
    pub fn remove_upgrade(&mut self, name: &str) {
        let Some(at) = self.upgrades.iter().position(|u| u.name == name) else {
            return;
        };
        let mask = self.upgrades.remove(at).mask;
        let none = UpgradeMaskType::default();
        self.upgrades_in_progress.clear_and_set(&mask, &none);
        self.upgrades_completed.clear_and_set(&mask, &none);
    }

    /// The upgrade entries of `Player::xfer`, `count` having been transferred.
    /// Loading adds each to the head, so it reverses the saved order.
    pub(super) fn xfer_upgrades(
        &mut self,
        xfer: &mut dyn Xfer,
        count: UnsignedShort,
        masks: &[(String, UpgradeMaskType)],
    ) -> Result<(), XferError> {
        if xfer.mode() != XferMode::Load {
            for upgrade in &mut self.upgrades {
                let mut name = upgrade.name.clone();
                xfer.xfer_ascii_string(&mut name)?;
                xfer_upgrade(xfer, &mut upgrade.status)?;
            }
            return Ok(());
        }
        for _ in 0..count {
            let mut name = String::new();
            xfer.xfer_ascii_string(&mut name)?;
            let (_, mask) = masks
                .iter()
                .find(|(known, _)| *known == name)
                .ok_or(XferError::InvalidData)?;
            self.add_upgrade(&name, mask, UPGRADE_STATUS_INVALID);
            // A repeated name keeps its place in the list.
            let upgrade = self.upgrades.iter_mut().find(|u| u.name == name);
            xfer_upgrade(xfer, &mut upgrade.ok_or(XferError::InvalidData)?.status)?;
        }
        Ok(())
    }
}

/// `Upgrade::xfer`.
fn xfer_upgrade(xfer: &mut dyn Xfer, status: &mut Int) -> Result<(), XferError> {
    let mut version: XferVersion = 1;
    xfer.xfer_version(&mut version, 1)?;
    xfer.xfer_int(status)
}

/// What a [`PlayerWorld`](crate::player::seams::PlayerWorld) needs to answer
/// `xfer_player_part` from the sim's own state.
pub struct PlayerParts<'a> {
    /// `SimWorld::ai_players`.
    pub ai_players: &'a mut BTreeMap<Int, AiPlayer>,
    /// `SimWorld::tunnel_trackers`. A player without an entry has used no tunnel.
    pub tunnel_trackers: &'a mut BTreeMap<Int, TunnelTrackerState>,
    pub thing_factory: Option<&'a ThingFactory>,
    pub teams: Option<&'a TeamFactory>,
}

impl PlayerParts<'_> {
    /// The snapshot, and for a load its `loadPostProcess`.
    pub fn xfer(
        &mut self,
        player: PlayerIndex,
        part: PlayerPart,
        xfer: &mut dyn Xfer,
    ) -> Result<(), XferError> {
        match part {
            PlayerPart::Ai => self
                .ai_players
                .get_mut(&player)
                .ok_or(XferError::InvalidData)?
                .xfer_with(xfer, self.thing_factory, self.teams),
            PlayerPart::TunnelTracker => {
                if xfer.mode() == XferMode::Load {
                    let tracker = self.tunnel_trackers.entry(player).or_default();
                    tracker.xfer(xfer)?;
                    // `TunnelTracker::loadPostProcess` puts the objects back
                    // into the tunnel: not ported.
                    if !tracker.contain_list.is_empty() {
                        return Err(XferError::InvalidData);
                    }
                    return Ok(());
                }
                match self.tunnel_trackers.get_mut(&player) {
                    Some(tracker) => tracker.xfer(xfer),
                    None => TunnelTrackerState::new().xfer(xfer),
                }
            }
        }
    }
}
