//! `PKT-199`: Base `CollideModuleData`, `CollideModule`, and `CollideBehaviorImpl`
//! (`CollideModule.h` / `CollideModule.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::module::{ModuleData, ModuleSnapshotHeader};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, NameKeyType, XferVersion, FALSE};
use zh_common::xfer::{Xfer, XferError};

/// Shared template configuration for `CollideModule` subclasses with no extra fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollideModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
}

impl ModuleData for CollideModuleData {
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
        FacetMask::COLLIDE
    }
}

/// Parses `CollideModuleData` base fields and delegates remaining fields to `parse_extra`.
pub fn parse_collide_data_with(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
    mut parse_extra: impl FnMut(
        &str,
        &mut IniReader<'_>,
        &mut ModuleParseContext<'_>,
    ) -> Result<(), IniError>,
) -> Result<CollideModuleData, IniError> {
    ini.read_block_fields(|field, reader| parse_extra(field, reader, parse_ctx))?;
    Ok(CollideModuleData {
        tag_key,
        module_name_key,
    })
}

/// Standard parser for `CollideModule` subclasses with no subclass INI fields.
pub fn parse_collide_module_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let data = parse_collide_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |_field, _reader, _ctx| Err(IniError::UnknownToken),
    )?;
    Ok(Box::new(data))
}

/// Base runtime state and snapshot serialization matching C++ `CollideModule` (`CollideModule.cpp`).
#[derive(Debug, Clone)]
pub struct CollideModule {
    pub module_name_key: NameKeyType,
}

impl CollideModule {
    #[inline]
    pub const fn new(module_name_key: NameKeyType) -> Self {
        Self { module_name_key }
    }

    #[inline]
    pub fn crc(&self, _xfer: &mut dyn Xfer) {}

    /// Serializes `CollideModule::xfer` (`version(1)` + `BehaviorModule::xfer`).
    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        ModuleSnapshotHeader::xfer_collide_module_header(xfer)
    }

    /// Serializes a concrete subclass with no additional fields (`version(1)` + `CollideModule::xfer`).
    pub fn xfer_subclass(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.xfer(xfer)
    }
}

/// Trait reducing boilerplate across direct `CollideModule` subclasses (`FireWeaponCollide`, `SquishCollide`).
pub trait CollideBehaviorImpl: Send {
    fn collide_base(&self) -> &CollideModule;
    fn collide_base_mut(&mut self) -> &mut CollideModule;
    fn on_collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
        loc: Option<&Coord3D>,
        normal: Option<&Coord3D>,
    );

    fn would_like_to_collide_with(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        _other: ObjectSlot,
    ) -> Bool {
        FALSE
    }

    fn is_hijacked_vehicle_crate_collide(&self) -> Bool {
        FALSE
    }

    fn is_sabotage_building_crate_collide(&self) -> Bool {
        FALSE
    }

    fn is_car_bomb_crate_collide(&self) -> Bool {
        FALSE
    }

    fn is_railroad(&self) -> Bool {
        FALSE
    }

    fn is_salvage_crate_collide(&self) -> Bool {
        FALSE
    }
}

/// Implements [`BehaviorModule`] and [`CollideFacet`] by delegating to [`CollideBehaviorImpl`].
#[allow(unused_macros)]
macro_rules! impl_collide_behavior {
    ($ty:ty) => {
        impl $crate::modules::module::BehaviorModule for $ty {
            fn module_name_key(&self) -> zh_common::types::NameKeyType {
                $crate::modules::collide::collide_module::CollideBehaviorImpl::collide_base(self)
                    .module_name_key
            }

            fn crc(&self, xfer: &mut dyn zh_common::xfer::Xfer) {
                $crate::modules::collide::collide_module::CollideBehaviorImpl::collide_base(self)
                    .crc(xfer);
            }

            fn xfer(
                &mut self,
                xfer: &mut dyn zh_common::xfer::Xfer,
                _sim: &dyn $crate::sim::SimView,
            ) -> Result<(), zh_common::xfer::XferError> {
                $crate::modules::collide::collide_module::CollideBehaviorImpl::collide_base_mut(
                    self,
                )
                .xfer_subclass(xfer)
            }

            fn as_collide(&self) -> Option<&dyn $crate::modules::module::CollideFacet> {
                Some(self)
            }

            fn as_collide_mut(&mut self) -> Option<&mut dyn $crate::modules::module::CollideFacet> {
                Some(self)
            }
        }

        impl $crate::modules::module::CollideFacet for $ty {
            fn on_collide(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
                other: Option<$crate::arena::ObjectSlot>,
                loc: Option<&zh_common::math::Coord3D>,
                normal: Option<&zh_common::math::Coord3D>,
            ) {
                $crate::modules::collide::collide_module::CollideBehaviorImpl::on_collide(
                    self, sim, ctx, other, loc, normal,
                );
            }

            fn would_like_to_collide_with(
                &self,
                sim: &dyn $crate::sim::SimView,
                ctx: $crate::arena::ModuleContext,
                other: $crate::arena::ObjectSlot,
            ) -> zh_common::types::Bool {
                $crate::modules::collide::collide_module::CollideBehaviorImpl::would_like_to_collide_with(
                    self, sim, ctx, other,
                )
            }

            fn is_hijacked_vehicle_crate_collide(&self) -> zh_common::types::Bool {
                $crate::modules::collide::collide_module::CollideBehaviorImpl::is_hijacked_vehicle_crate_collide(self)
            }

            fn is_sabotage_building_crate_collide(&self) -> zh_common::types::Bool {
                $crate::modules::collide::collide_module::CollideBehaviorImpl::is_sabotage_building_crate_collide(self)
            }

            fn is_car_bomb_crate_collide(&self) -> zh_common::types::Bool {
                $crate::modules::collide::collide_module::CollideBehaviorImpl::is_car_bomb_crate_collide(self)
            }

            fn is_railroad(&self) -> zh_common::types::Bool {
                $crate::modules::collide::collide_module::CollideBehaviorImpl::is_railroad(self)
            }

            fn is_salvage_crate_collide(&self) -> zh_common::types::Bool {
                $crate::modules::collide::collide_module::CollideBehaviorImpl::is_salvage_crate_collide(self)
            }
        }
    };
}

#[allow(unused_imports)]
pub(crate) use impl_collide_behavior;
