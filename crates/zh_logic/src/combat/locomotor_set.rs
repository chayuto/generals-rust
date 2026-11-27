//! `PKT-144`: Deterministic collection of locomotor instances.

use super::locomotor::{Locomotor, LocomotorStore};
use crate::rules::combat_types::LocomotorSurfaceTypeMask;
use zh_common::types::{Bool, Int, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

pub const LOCOMOTORSET_COUNT: usize = 8;

/// Locomotor set slot matching C++ `LocomotorSetType` (`AIUpdate.h:72-86`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum LocomotorSetType {
    Invalid = -1,
    #[default]
    Normal = 0,
    NormalUpgraded = 1,
    Freefall = 2,
    Wander = 3,
    Panic = 4,
    Taxiing = 5,
    Supersonic = 6,
    Sluggish = 7,
}

impl LocomotorSetType {
    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            -1 => Some(Self::Invalid),
            0 => Some(Self::Normal),
            1 => Some(Self::NormalUpgraded),
            2 => Some(Self::Freefall),
            3 => Some(Self::Wander),
            4 => Some(Self::Panic),
            5 => Some(Self::Taxiing),
            6 => Some(Self::Supersonic),
            7 => Some(Self::Sluggish),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// INI string names matching `TheLocomotorSetNames[]` (`AIUpdate.h:99-111`).
pub const LOCOMOTOR_SET_NAMES: [&str; LOCOMOTORSET_COUNT] = [
    "SET_NORMAL",
    "SET_NORMAL_UPGRADED",
    "SET_FREEFALL",
    "SET_WANDER",
    "SET_PANIC",
    "SET_TAXIING",
    "SET_SUPERSONIC",
    "SET_SLUGGISH",
];

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LocomotorSet {
    locomotors: Vec<Locomotor>,
    valid_surfaces: LocomotorSurfaceTypeMask,
    downhill_only: Bool,
}

impl LocomotorSet {
    pub fn new() -> Self { Self::default() }
    pub fn clear(&mut self) {
        self.locomotors.clear();
        self.valid_surfaces = 0;
        self.downhill_only = FALSE;
    }
    pub fn add_locomotor(&mut self, locomotor: Locomotor) {
        self.valid_surfaces |= locomotor.get_legal_surfaces();
        if locomotor.get_is_downhill_only() != FALSE {
            self.downhill_only = TRUE;
        }
        self.locomotors.push(locomotor);
    }
    pub fn find_locomotor(&self, surfaces: LocomotorSurfaceTypeMask) -> Option<&Locomotor> {
        self.locomotors.iter().find(|l| (l.get_legal_surfaces() & surfaces) != 0)
    }
    pub fn locomotors(&self) -> &[Locomotor] { &self.locomotors }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Locomotor> { self.locomotors.get_mut(index) }
    pub fn get_valid_surfaces(&self) -> LocomotorSurfaceTypeMask { self.valid_surfaces }
    pub fn is_downhill_only(&self) -> Bool { self.downhill_only }
}

impl Snapshot for LocomotorSet {
    fn crc(&self, _xfer: &mut dyn Xfer) {}
    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut count = self.locomotors.len().min(u16::MAX as usize) as u16;
        xfer.xfer_unsigned_short(&mut count)?;
        match xfer.mode() {
            XferMode::Save | XferMode::Crc => {
                for loco in &mut self.locomotors {
                    let mut name = loco.get_template_name().to_owned();
                    xfer.xfer_ascii_string(&mut name)?;
                    xfer.xfer_snapshot(loco)?;
                }
            }
            XferMode::Load => {
                if !self.locomotors.is_empty() { return Err(XferError::ListNotEmpty); }
                // Loading template lookup is supplied by the caller via the xfer_with_store helper.
                if count != 0 { return Err(XferError::InvalidParameters); }
            }
        }
        xfer.xfer_int(&mut self.valid_surfaces)?;
        xfer.xfer_bool(&mut self.downhill_only)?;
        Ok(())
    }
}

impl LocomotorSet {
    pub fn xfer_with_store(
        &mut self,
        xfer: &mut dyn Xfer,
        store: &LocomotorStore,
    ) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut count = self.locomotors.len().min(u16::MAX as usize) as u16;
        xfer.xfer_unsigned_short(&mut count)?;
        match xfer.mode() {
            XferMode::Save | XferMode::Crc => {
                for loco in &mut self.locomotors {
                    let mut name = loco.get_template_name().to_owned();
                    xfer.xfer_ascii_string(&mut name)?;
                    xfer.xfer_snapshot(loco)?;
                }
            }
            XferMode::Load => {
                if !self.locomotors.is_empty() { return Err(XferError::ListNotEmpty); }
                for _ in 0..count {
                    let mut name = String::new();
                    xfer.xfer_ascii_string(&mut name)?;
                    let template = store.find_by_name(&name).ok_or(XferError::InvalidData)?;
                    let mut loco = Locomotor::new(template, &mut zh_common::random_value::PrngState::new(), 0);
                    xfer.xfer_snapshot(&mut loco)?;
                    self.locomotors.push(loco);
                }
            }
        }
        xfer.xfer_int(&mut self.valid_surfaces)?;
        xfer.xfer_bool(&mut self.downhill_only)?;
        Ok(())
    }

    pub fn xfer_self_and_cur_loco_ptr(
        &mut self,
        xfer: &mut dyn Xfer,
        store: &LocomotorStore,
        cur_name: Option<&str>,
    ) -> Result<Option<&mut Locomotor>, XferError> {
        // `xferSelfAndCurLocoPtr` goes through `xferSnapshot`, which in CRC mode
        // calls the empty `LocomotorSet::crc`, and then writes nothing either.
        if xfer.mode() == XferMode::Crc {
            return Ok(None);
        }
        self.xfer_with_store(xfer, store)?;
        match xfer.mode() {
            XferMode::Save | XferMode::Crc => {
                let mut name = cur_name.unwrap_or("").to_owned();
                xfer.xfer_ascii_string(&mut name)?;
            }
            XferMode::Load => {
                let mut name = String::new();
                xfer.xfer_ascii_string(&mut name)?;
                if name.is_empty() { return Ok(None); }
                let idx = self.locomotors.iter().position(|l| l.get_template_name() == name);
                if let Some(idx) = idx { return Ok(self.locomotors.get_mut(idx)); }
                return Err(XferError::InvalidData);
            }
        }
        Ok(cur_name.and_then(|name| self.locomotors.iter_mut().find(|l| l.get_template_name() == name)))
    }
}
