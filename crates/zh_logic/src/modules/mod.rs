//! `WP-5.0`: Phase 5 `BehaviorModule` registry, traits, facet interfaces, and `ModuleFactory`
//! (`PKT-163`, `PKT-164`) plus `WP-5.1` and `WP-5.2` module category submodules.

pub mod behavior;
pub mod body;
pub mod collide;
pub mod contain;
pub mod create;
pub mod damage;
pub mod destroy;
pub mod die;
pub mod helper;
pub mod module;
pub mod module_data;
pub mod module_factory;
pub mod special_power;
pub mod update;
pub mod upgrade;

pub use module::{
    AiUpdateFacet, AttitudeType, BehaviorModule, BodyFacet, BridgeFacet, BridgeScaffoldFacet,
    BridgeTowerFacet, BridgeTowerType, CaveFacet, CollideFacet, CommandSourceType, ContainFacet,
    CountermeasuresFacet, CreateFacet, DamageFacet, DestroyFacet, DieFacet, EvacDisposition,
    ExitDoorType, ExitFacet, GenerateMinefieldFacet, HelperFacet, LandMineFacet, MaxHealthChangeType, ModuleData,
    ModuleSnapshotHeader, ModuleType, ObjectEnterExitType, OverchargeFacet, ParkingPlaceFacet,
    ParkingPlaceInfo, JetAiState, PhysicsFacet, ProjectileFacet, RebuildHoleFacet, RunwayReservationType,
    ScaffoldTargetMotion, SlavedUpdateFacet, SlowDeathFacet, SlowDeathPhaseType,
    SpawnBehaviorFacet, SpecialPowerFacet, StealthDetectorFacet, StealthFacet, StealthParams, StubModuleData,
    TransportPassengerFacet, UpdateFacet, UpgradeFacet, INVALID_BODY_TIMESTAMP,
    MAX_HEALTH_CHANGE_NAMES,
};
pub use module_factory::{
    ModuleCreateFn, ModuleDataParseFn, ModuleDataStore, ModuleDescriptor, ModuleParseContext,
    ModuleRegistry, BUILTIN_ZH_MODULES, W3D_DRAW_STUB_NAMES,
};
