//! `PKT-212`: Base `DieMuxData`, `DieModuleData`, and `DieModule`
//! (`DieModule.h`, `DieModule.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::module::{ModuleData, ModuleSnapshotHeader};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{get_death_type_flag, DeathTypeFlags, DEATH_TYPE_FLAGS_ALL};
use crate::rules::object_status_types::{ObjectStatusMaskType, OBJECT_STATUS_NAMES};
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, VeterancyLevelFlags, VETERANCY_LEVEL_FLAGS_ALL};
use zh_common::types::{Bool, NameKeyType, VeterancyLevel, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// Checks whether `level` is enabled in `flags` (`getVeterancyLevelFlag` in `GameCommon.h`).
#[inline]
pub fn get_veterancy_level_flag(flags: VeterancyLevelFlags, level: VeterancyLevel) -> Bool {
    let bit = level.as_i32();
    Bool::from(bit >= 0 && bit < 32 && (flags & (1u32 << (bit as u32))) != 0)
}

/// Shared applicability filter matching C++ `DieMuxData` (`DieModule.h` / `DieModule.cpp`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DieMuxData {
    pub death_types: DeathTypeFlags,
    pub veterancy_levels: VeterancyLevelFlags,
    pub exempt_status: ObjectStatusMaskType,
    pub required_status: ObjectStatusMaskType,
}

impl Default for DieMuxData {
    fn default() -> Self {
        Self::new()
    }
}

impl DieMuxData {
    #[inline]
    pub fn new() -> Self {
        Self {
            death_types: DEATH_TYPE_FLAGS_ALL,
            veterancy_levels: VETERANCY_LEVEL_FLAGS_ALL,
            exempt_status: ObjectStatusMaskType::new(),
            required_status: ObjectStatusMaskType::new(),
        }
    }

    /// Parses one `DieMuxData` INI field, returning `None` if `field` belongs to a subclass.
    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
    ) -> Option<Result<(), IniError>> {
        if field.eq_ignore_ascii_case("DeathTypes") {
            Some(reader.parse_death_type_flags(&mut self.death_types))
        } else if field.eq_ignore_ascii_case("VeterancyLevels") {
            Some(reader.parse_veterancy_level_flags(&mut self.veterancy_levels))
        } else if field.eq_ignore_ascii_case("ExemptStatus") {
            Some(self.exempt_status.parse_from_ini(reader, &OBJECT_STATUS_NAMES))
        } else if field.eq_ignore_ascii_case("RequiredStatus") {
            Some(self.required_status.parse_from_ini(reader, &OBJECT_STATUS_NAMES))
        } else {
            None
        }
    }

    /// Evaluates C++ `DieMuxData::isDieApplicable`.
    pub fn is_die_applicable(
        &self,
        sim: &dyn SimView,
        owner: ObjectSlot,
        dmg: &DamageInfo,
    ) -> Bool {
        if get_death_type_flag(self.death_types, dmg.input.death_type.as_i32()) == FALSE {
            return FALSE;
        }
        let vet = sim.object_veterancy_level(owner);
        if get_veterancy_level_flag(self.veterancy_levels, vet) == FALSE {
            return FALSE;
        }
        let Some(obj) = sim.object(owner) else {
            return FALSE;
        };
        if obj.status.test_for_none(&self.exempt_status) == FALSE {
            return FALSE;
        }
        if obj.status.test_for_all(&self.required_status) == FALSE {
            return FALSE;
        }
        TRUE
    }
}

/// Shared template configuration for `DieModule` subclasses with no extra fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DieModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub die_mux: DieMuxData,
}

impl DieModuleData {
    #[inline]
    pub fn is_die_applicable(
        &self,
        sim: &dyn SimView,
        owner: ObjectSlot,
        dmg: &DamageInfo,
    ) -> Bool {
        self.die_mux.is_die_applicable(sim, owner, dmg)
    }
}

impl ModuleData for DieModuleData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask::DIE
    }
}

/// Parses `DieModuleData` base fields and delegates remaining fields to `parse_extra`.
pub fn parse_die_data_with(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
    mut parse_extra: impl FnMut(&str, &mut IniReader<'_>, &mut ModuleParseContext<'_>) -> Result<(), IniError>,
) -> Result<DieModuleData, IniError> {
    let mut base = DieModuleData {
        tag_key,
        module_name_key,
        die_mux: DieMuxData::new(),
    };
    ini.read_block_fields(|field, reader| {
        if let Some(res) = base.die_mux.parse_field(field, reader) {
            res
        } else {
            parse_extra(field, reader, parse_ctx)
        }
    })?;
    Ok(base)
}

/// Standard parser for `DieModule` subclasses with no subclass INI fields.
pub fn parse_die_module_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let data = parse_die_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |_field, _reader, _ctx| Err(IniError::UnknownToken),
    )?;
    Ok(Box::new(data))
}

/// Base runtime state and snapshot serialization matching C++ `DieModule` (`DieModule.cpp`).
#[derive(Debug, Clone)]
pub struct DieModule {
    pub module_name_key: NameKeyType,
    pub die_mux: DieMuxData,
}

impl DieModule {
    #[inline]
    pub fn new(module_name_key: NameKeyType, die_mux: DieMuxData) -> Self {
        Self {
            module_name_key,
            die_mux,
        }
    }

    #[inline]
    pub fn is_die_applicable(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Bool {
        self.die_mux.is_die_applicable(sim, ctx.owner_slot, dmg)
    }

    #[inline]
    pub fn crc(&self, _xfer: &mut dyn Xfer) {}

    /// Serializes `DieModule::xfer` (`version(1)` + `BehaviorModule::xfer`).
    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        ModuleSnapshotHeader::xfer_die_module_header(xfer)
    }

    /// Serializes a concrete subclass with no additional fields (`version(1)` + `DieModule::xfer`).
    pub fn xfer_subclass(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.xfer(xfer)
    }
}

/// Internal trait reducing boilerplate across simple concrete `DieModule` subclasses.
pub(crate) trait DieBehaviorImpl: Send {
    fn base(&self) -> &DieModule;
    fn base_mut(&mut self) -> &mut DieModule;
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo);
}

macro_rules! impl_die_behavior {
    ($ty:ty) => {
        impl BehaviorModule for $ty {
            fn module_name_key(&self) -> zh_common::types::NameKeyType {
                self.base().module_name_key
            }

            fn crc(&self, xfer: &mut dyn zh_common::xfer::Xfer) {
                self.base().crc(xfer);
            }

            fn xfer(
                &mut self,
                xfer: &mut dyn zh_common::xfer::Xfer,
                _sim: &dyn crate::sim::SimView,
            ) -> Result<(), zh_common::xfer::XferError> {
                self.base_mut().xfer_subclass(xfer)
            }

            fn as_die(&self) -> Option<&dyn DieFacet> {
                Some(self)
            }

            fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
                Some(self)
            }
        }

        impl DieFacet for $ty {
            fn on_die(
                &mut self,
                sim: &mut dyn crate::sim::SimApi,
                ctx: crate::arena::ModuleContext,
                dmg: &crate::combat::damage::DamageInfo,
            ) {
                DieBehaviorImpl::on_die(self, sim, ctx, dmg);
            }

            fn is_die_applicable(
                &self,
                sim: &dyn crate::sim::SimView,
                ctx: crate::arena::ModuleContext,
                dmg: &crate::combat::damage::DamageInfo,
            ) -> zh_common::types::Bool {
                self.base().is_die_applicable(sim, ctx, dmg)
            }
        }
    };
}

pub(crate) use impl_die_behavior;
