//! `PKT-118`: Player power production, consumption, sabotage & brownout tracker
//! (`GeneralsMD/Code/GameEngine/Source/Common/RTS/Energy.cpp`).

use zh_common::types::{Bool, Int, Real, UnsignedInt, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Snapshot, Xfer, XferError};

/// Callback invoked whenever power production or consumption changes on an owned [`Energy`].
pub trait EnergyOwnerCallbacks {
    fn on_power_brown_out_change(&mut self, _now_brownout: Bool) {}
}

/// No-op implementation of [`EnergyOwnerCallbacks`].
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopEnergyCallbacks;

impl EnergyOwnerCallbacks for NoopEnergyCallbacks {}

/// Subset of object/template energy properties read by [`Energy`] influence/bonus methods.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EnergyObjectInfo {
    pub energy_production: Int,
    pub energy_bonus: Int,
    pub is_disabled: Bool,
}

/// Player electrical power pool matching C++ `Energy`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Energy {
    energy_production: Int,
    energy_consumption: Int,
    power_sabotaged_till_frame: UnsignedInt,
    owner_player_index: Option<Int>,
}

impl Energy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(&mut self, owner_player_index: Option<Int>) {
        self.energy_production = 0;
        self.energy_consumption = 0;
        self.power_sabotaged_till_frame = 0;
        self.owner_player_index = owner_player_index;
    }

    #[inline]
    pub fn get_production(&self, current_frame: UnsignedInt) -> Int {
        if current_frame < self.power_sabotaged_till_frame {
            0
        } else {
            self.energy_production
        }
    }

    #[inline]
    pub fn get_consumption(&self) -> Int {
        self.energy_consumption
    }

    #[inline]
    pub fn has_sufficient_power(&self, current_frame: UnsignedInt) -> Bool {
        if current_frame < self.power_sabotaged_till_frame {
            FALSE
        } else if self.energy_production >= self.energy_consumption {
            TRUE
        } else {
            FALSE
        }
    }

    /// Returns the production-to-consumption ratio (`0.0` if sabotaged, `production as f32` if consumption is `0`).
    #[inline]
    pub fn get_energy_supply_ratio(&self, current_frame: UnsignedInt) -> Real {
        if current_frame < self.power_sabotaged_till_frame {
            return 0.0_f32;
        }
        if self.energy_consumption == 0 {
            return self.energy_production as Real;
        }
        (self.energy_production as Real) / (self.energy_consumption as Real)
    }

    pub fn adjust_power(
        &mut self,
        power_delta: Int,
        adding: Bool,
        current_frame: UnsignedInt,
        cb: &mut dyn EnergyOwnerCallbacks,
    ) {
        if power_delta == 0 {
            return;
        }
        if power_delta > 0 {
            if adding != FALSE {
                self.add_production(power_delta, current_frame, cb);
            } else {
                self.add_production(power_delta.wrapping_neg(), current_frame, cb);
            }
        } else if adding != FALSE {
            self.add_consumption(power_delta.wrapping_neg(), current_frame, cb);
        } else {
            self.add_consumption(power_delta, current_frame, cb);
        }
    }

    pub fn object_entering_influence(
        &mut self,
        obj: Option<EnergyObjectInfo>,
        current_frame: UnsignedInt,
        cb: &mut dyn EnergyOwnerCallbacks,
    ) {
        let Some(info) = obj else {
            return;
        };
        if info.energy_production < 0 {
            self.add_consumption(info.energy_production.wrapping_neg(), current_frame, cb);
        } else if info.energy_production > 0 {
            self.add_production(info.energy_production, current_frame, cb);
        }
    }

    pub fn object_leaving_influence(
        &mut self,
        obj: Option<EnergyObjectInfo>,
        current_frame: UnsignedInt,
        cb: &mut dyn EnergyOwnerCallbacks,
    ) {
        let Some(info) = obj else {
            return;
        };
        if info.energy_production < 0 {
            self.add_consumption(info.energy_production, current_frame, cb);
        } else if info.energy_production > 0 {
            self.add_production(info.energy_production.wrapping_neg(), current_frame, cb);
        }
    }

    pub fn add_power_bonus(
        &mut self,
        obj: Option<EnergyObjectInfo>,
        current_frame: UnsignedInt,
        cb: &mut dyn EnergyOwnerCallbacks,
    ) {
        let Some(info) = obj else {
            return;
        };
        self.add_production(info.energy_bonus, current_frame, cb);
    }

    pub fn remove_power_bonus(
        &mut self,
        obj: Option<EnergyObjectInfo>,
        current_frame: UnsignedInt,
        cb: &mut dyn EnergyOwnerCallbacks,
    ) {
        let Some(info) = obj else {
            return;
        };
        #[cfg(not(feature = "retail-crc"))]
        if info.is_disabled != FALSE {
            return;
        }
        self.add_production(info.energy_bonus.wrapping_neg(), current_frame, cb);
    }

    #[inline]
    pub fn set_power_sabotaged_till_frame(&mut self, frame: UnsignedInt) {
        self.power_sabotaged_till_frame = frame;
    }

    #[inline]
    pub fn get_power_sabotaged_till_frame(&self) -> UnsignedInt {
        self.power_sabotaged_till_frame
    }

    #[inline]
    pub fn owner_player_index(&self) -> Option<Int> {
        self.owner_player_index
    }

    fn add_production(
        &mut self,
        amt: Int,
        current_frame: UnsignedInt,
        cb: &mut dyn EnergyOwnerCallbacks,
    ) {
        self.energy_production = self.energy_production.wrapping_add(amt);
        if self.owner_player_index.is_none() {
            return;
        }
        let brownout = if self.has_sufficient_power(current_frame) != FALSE {
            FALSE
        } else {
            TRUE
        };
        cb.on_power_brown_out_change(brownout);
    }

    fn add_consumption(
        &mut self,
        amt: Int,
        current_frame: UnsignedInt,
        cb: &mut dyn EnergyOwnerCallbacks,
    ) {
        self.energy_consumption = self.energy_consumption.wrapping_add(amt);
        if self.owner_player_index.is_none() {
            return;
        }
        let brownout = if self.has_sufficient_power(current_frame) != FALSE {
            FALSE
        } else {
            TRUE
        };
        cb.on_power_brown_out_change(brownout);
    }
}

impl Snapshot for Energy {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 3;
        xfer.xfer_version(&mut version, 3)?;

        if version < 2 {
            xfer.xfer_int(&mut self.energy_production)?;
            xfer.xfer_int(&mut self.energy_consumption)?;
        }

        let mut owning_player_index: Int = self.owner_player_index.unwrap_or(-1);
        xfer.xfer_int(&mut owning_player_index)?;
        self.owner_player_index = if owning_player_index >= 0 {
            Some(owning_player_index)
        } else {
            None
        };

        if version >= 3 {
            xfer.xfer_unsigned_int(&mut self.power_sabotaged_till_frame)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_power_bonus_removal_respects_retail_crc_flag() {
        let mut energy = Energy::new();
        energy.init(Some(0));
        let mut cb = NoopEnergyCallbacks;
        let obj = EnergyObjectInfo {
            energy_production: 10,
            energy_bonus: 5,
            is_disabled: TRUE,
        };
        energy.add_power_bonus(Some(obj), 10, &mut cb);
        energy.remove_power_bonus(Some(obj), 10, &mut cb);
        if cfg!(feature = "retail-crc") {
            assert_eq!(energy.get_production(10), 0);
        } else {
            assert_eq!(energy.get_production(10), 5);
        }
    }
}
