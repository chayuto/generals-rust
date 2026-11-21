//! `PKT-120`: Per-player mission kill/loss counters
//! (`GeneralsMD/Code/GameEngine/Source/Common/RTS/MissionStats.cpp`).

use super::MAX_PLAYER_COUNT;
use zh_common::types::{Int, XferVersion};
use zh_common::xfer::{Snapshot, Xfer, XferError};

/// Per-player kill/loss statistics matching C++ `MissionStats`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MissionStats {
    pub units_killed: [Int; MAX_PLAYER_COUNT],
    pub units_lost: Int,
    pub buildings_killed: [Int; MAX_PLAYER_COUNT],
    pub buildings_lost: Int,
}

impl MissionStats {
    pub fn new() -> Self {
        let mut out = Self::default();
        out.init();
        out
    }

    pub fn init(&mut self) {
        self.units_killed = [0; MAX_PLAYER_COUNT];
        self.buildings_killed = [0; MAX_PLAYER_COUNT];
        self.units_lost = 0;
        self.buildings_lost = 0;
    }
}

impl Snapshot for MissionStats {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;

        xfer_i32_array(xfer, &mut self.units_killed)?;
        xfer.xfer_int(&mut self.units_lost)?;
        xfer_i32_array(xfer, &mut self.buildings_killed)?;
        xfer.xfer_int(&mut self.buildings_lost)?;

        Ok(())
    }
}

fn xfer_i32_array<const N: usize>(xfer: &mut dyn Xfer, arr: &mut [Int; N]) -> Result<(), XferError> {
    let mut buf = vec![0u8; N * 4];
    for (i, &val) in arr.iter().enumerate() {
        buf[i * 4..i * 4 + 4].copy_from_slice(&val.to_le_bytes());
    }
    xfer.xfer_user(&mut buf)?;
    for (i, slot) in arr.iter_mut().enumerate() {
        *slot = i32::from_le_bytes([
            buf[i * 4],
            buf[i * 4 + 1],
            buf[i * 4 + 2],
            buf[i * 4 + 3],
        ]);
    }
    Ok(())
}
