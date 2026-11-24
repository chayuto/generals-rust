//! `WP-5.0`: Phase 5 `BehaviorModule` registry, traits, facet interfaces, and `ModuleFactory`
//! (`PKT-163`, `PKT-164`) plus `WP-5.1` and `WP-5.2` module category submodules.

pub mod body;
pub mod collide;
pub mod create;
pub mod damage;
pub mod destroy;
pub mod die;
pub mod helper;
pub mod module;
pub mod module_factory;
pub mod special_power;
pub mod upgrade;

pub use module::{
    AiUpdateFacet, BehaviorModule, BodyFacet, CollideFacet, ContainFacet, CreateFacet,
    DamageFacet, DestroyFacet, DieFacet, HelperFacet, MaxHealthChangeType, ModuleData,
    ModuleSnapshotHeader, ModuleType, PhysicsFacet, SlowDeathFacet, SpecialPowerFacet,
    StealthFacet, StubModuleData, UpdateFacet, UpgradeFacet, INVALID_BODY_TIMESTAMP,
    MAX_HEALTH_CHANGE_NAMES,
};
pub use module_factory::{
    ModuleCreateFn, ModuleDataParseFn, ModuleDataStore, ModuleDescriptor, ModuleParseContext,
    ModuleRegistry, BUILTIN_ZH_MODULES, W3D_DRAW_STUB_NAMES,
};
