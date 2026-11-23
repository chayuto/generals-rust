//! `WP-5.0`: Phase 5 `BehaviorModule` registry, traits, facet interfaces, and `ModuleFactory`
//! (`PKT-163`, `PKT-164`).

pub mod module;
pub mod module_factory;

pub use module::{
    AiUpdateFacet, BehaviorModule, BodyFacet, CollideFacet, ContainFacet, CreateFacet,
    DamageFacet, DestroyFacet, DieFacet, ModuleData, ModuleSnapshotHeader, ModuleType,
    PhysicsFacet, SpecialPowerFacet, StealthFacet, StubModuleData, UpdateFacet, UpgradeFacet,
};
pub use module_factory::{
    ModuleCreateFn, ModuleDataParseFn, ModuleDataStore, ModuleDescriptor, ModuleRegistry,
    BUILTIN_ZH_MODULES, W3D_DRAW_STUB_NAMES,
};
