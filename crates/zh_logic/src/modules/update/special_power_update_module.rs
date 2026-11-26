//! `PKT-369`: `SpecialPowerUpdateModule`
//! (`SpecialPowerUpdateModule.h`, `SpecialPowerUpdateModule.cpp`).
//!
//! Packet: `packets/phase5/PKT-369_SpecialPowerUpdateModule.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp54_c.rs`.
//!
//! The reference class is an abstract base, so it is not registered as a module.
//! These are the parts its subclasses inherit. A subclass passes its own
//! `getExtraRequiredScience` value (`SCIENCE_INVALID` unless it overrides it)
//! and writes its own version byte before calling the base `xfer`.

use crate::arena::ObjectSlot;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::rules::science::{ScienceType, SCIENCE_INVALID};
use crate::sim::SimView;
use zh_common::types::{Bool, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// `doesSpecialPowerUpdatePassScienceTest`.
pub fn does_special_power_update_pass_science_test(
    sim: &dyn SimView,
    owner: ObjectSlot,
    extra_required_science: ScienceType,
) -> Bool {
    if extra_required_science == SCIENCE_INVALID {
        return TRUE;
    }
    sim.object(owner).map_or(FALSE, |obj| {
        sim.player_has_science(obj.team_id, extra_required_science)
    })
}

/// `SpecialPowerUpdateModule::xfer`.
pub fn xfer_special_power_update_module(
    xfer: &mut dyn Xfer,
    schedule: &mut UpdateScheduleTracker,
    sim: &dyn SimView,
) -> Result<(), XferError> {
    xfer_module_version(xfer, 1)?;
    schedule.xfer_update_header(xfer, sim)
}

/// `SpecialPowerUpdateModule::crc`.
pub fn crc_special_power_update_module(xfer: &mut dyn Xfer, schedule: &UpdateScheduleTracker) {
    schedule.crc(xfer);
}
