# Translation Packet `PKT-164`: `ModuleFactory`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-164`
- **Work Package:** `WP-5.0` (Phase 5)
- **Complexity & Assigned Tier:** Class `C4` -> `tier:3-architect`
- **Environment Access:** `env:local-full`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/module_factory.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (212):**
  - `Common/AsciiString.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/AsciiString.h`
  - `Common/GameMemory.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameMemory.h`
  - `Common/Module.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Module.h`
  - `Common/ModuleFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ModuleFactory.h`
  - `Common/NameKeyGenerator.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/NameKeyGenerator.h`
  - `Common/STLTypedefs.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/STLTypedefs.h`
  - `Common/SubsystemInterface.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/SubsystemInterface.h`
  - `GameClient/Module/AnimatedParticleSysBoneClientUpdate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/Module/AnimatedParticleSysBoneClientUpdate.h`
  - `GameClient/Module/BeaconClientUpdate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/Module/BeaconClientUpdate.h`
  - `GameClient/Module/SwayClientUpdate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/Module/SwayClientUpdate.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/ActiveBody.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ActiveBody.h`
  - `GameLogic/Module/ActiveShroudUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ActiveShroudUpgrade.h`
  - `GameLogic/Module/AnimationSteeringUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AnimationSteeringUpdate.h`
  - `GameLogic/Module/ArmorUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ArmorUpgrade.h`
  - `GameLogic/Module/AssaultTransportAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AssaultTransportAIUpdate.h`
  - `GameLogic/Module/AssistedTargetingUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AssistedTargetingUpdate.h`
  - `GameLogic/Module/AutoDepositUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AutoDepositUpdate.h`
  - `GameLogic/Module/AutoFindHealingUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AutoFindHealingUpdate.h`
  - `GameLogic/Module/AutoHealBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AutoHealBehavior.h`
  - `GameLogic/Module/BaikonurLaunchPower.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BaikonurLaunchPower.h`
  - `GameLogic/Module/BaseRegenerateUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BaseRegenerateUpdate.h`
  - `GameLogic/Module/BattleBusSlowDeathBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BattleBusSlowDeathBehavior.h`
  - `GameLogic/Module/BattlePlanUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BattlePlanUpdate.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BoneFXDamage.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BoneFXDamage.h`
  - `GameLogic/Module/BoneFXUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BoneFXUpdate.h`
  - `GameLogic/Module/BridgeBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BridgeBehavior.h`
  - `GameLogic/Module/BridgeScaffoldBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BridgeScaffoldBehavior.h`
  - `GameLogic/Module/BridgeTowerBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BridgeTowerBehavior.h`
  - `GameLogic/Module/BunkerBusterBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BunkerBusterBehavior.h`
  - `GameLogic/Module/CashBountyPower.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CashBountyPower.h`
  - `GameLogic/Module/CashHackSpecialPower.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CashHackSpecialPower.h`
  - `GameLogic/Module/CaveContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CaveContain.h`
  - `GameLogic/Module/CheckpointUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CheckpointUpdate.h`
  - `GameLogic/Module/ChinookAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ChinookAIUpdate.h`
  - `GameLogic/Module/CleanupAreaPower.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CleanupAreaPower.h`
  - `GameLogic/Module/CleanupHazardUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CleanupHazardUpdate.h`
  - `GameLogic/Module/CommandButtonHuntUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CommandButtonHuntUpdate.h`
  - `GameLogic/Module/CommandSetUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CommandSetUpgrade.h`
  - `GameLogic/Module/ConvertToCarBombCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ConvertToCarBombCrateCollide.h`
  - `GameLogic/Module/ConvertToHijackedVehicleCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ConvertToHijackedVehicleCrateCollide.h`
  - `GameLogic/Module/CostModifierUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CostModifierUpgrade.h`
  - `GameLogic/Module/CountermeasuresBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CountermeasuresBehavior.h`
  - `GameLogic/Module/CreateCrateDie.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CreateCrateDie.h`
  - `GameLogic/Module/CreateObjectDie.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CreateObjectDie.h`
  - `GameLogic/Module/CrushDie.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CrushDie.h`
  - `GameLogic/Module/DamDie.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DamDie.h`
  - `GameLogic/Module/DefaultProductionExitUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DefaultProductionExitUpdate.h`
  - `GameLogic/Module/DefectorSpecialPower.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DefectorSpecialPower.h`
  - `GameLogic/Module/DeletionUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DeletionUpdate.h`
  - `GameLogic/Module/DeliverPayloadAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DeliverPayloadAIUpdate.h`
  - `GameLogic/Module/DemoTrapUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DemoTrapUpdate.h`
  - `GameLogic/Module/DemoralizeSpecialPower.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DemoralizeSpecialPower.h`
  - `GameLogic/Module/DeployStyleAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DeployStyleAIUpdate.h`
  - `GameLogic/Module/DestroyDie.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DestroyDie.h`
  - `GameLogic/Module/DozerAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DozerAIUpdate.h`
  - `GameLogic/Module/DumbProjectileBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DumbProjectileBehavior.h`
  - `GameLogic/Module/DynamicGeometryInfoUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DynamicGeometryInfoUpdate.h`
  - `GameLogic/Module/DynamicShroudClearingRangeUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DynamicShroudClearingRangeUpdate.h`
  - `GameLogic/Module/EMPUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/EMPUpdate.h`
  - `GameLogic/Module/EjectPilotDie.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/EjectPilotDie.h`
  - `GameLogic/Module/EnemyNearUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/EnemyNearUpdate.h`
  - `GameLogic/Module/ExperienceScalarUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ExperienceScalarUpgrade.h`
  - `GameLogic/Module/FXListDie.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FXListDie.h`
  - `GameLogic/Module/FireOCLAfterWeaponCooldownUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FireOCLAfterWeaponCooldownUpdate.h`
  - `GameLogic/Module/FireSpreadUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FireSpreadUpdate.h`
  - `GameLogic/Module/FireWeaponCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FireWeaponCollide.h`
  - `GameLogic/Module/FireWeaponPower.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FireWeaponPower.h`
  - `GameLogic/Module/FireWeaponUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FireWeaponUpdate.h`
  - `GameLogic/Module/FireWeaponWhenDamagedBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FireWeaponWhenDamagedBehavior.h`
  - `GameLogic/Module/FireWeaponWhenDeadBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FireWeaponWhenDeadBehavior.h`
  - `GameLogic/Module/FirestormDynamicGeometryInfoUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FirestormDynamicGeometryInfoUpdate.h`
  - `GameLogic/Module/FlammableUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FlammableUpdate.h`
  - `GameLogic/Module/FlightDeckBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FlightDeckBehavior.h`
  - `GameLogic/Module/FloatUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FloatUpdate.h`
  - `GameLogic/Module/GarrisonContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/GarrisonContain.h`
  - `GameLogic/Module/GenerateMinefieldBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/GenerateMinefieldBehavior.h`
  - `GameLogic/Module/GrantScienceUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/GrantScienceUpgrade.h`
  - `GameLogic/Module/GrantStealthBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/GrantStealthBehavior.h`
  - `GameLogic/Module/GrantUpgradeCreate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/GrantUpgradeCreate.h`
  - `GameLogic/Module/HackInternetAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HackInternetAIUpdate.h`
  - `GameLogic/Module/HealContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HealContain.h`
  - `GameLogic/Module/HealCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HealCrateCollide.h`
  - `GameLogic/Module/HeightDieUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HeightDieUpdate.h`
  - `GameLogic/Module/HelicopterSlowDeathUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HelicopterSlowDeathUpdate.h`
  - `GameLogic/Module/HelixContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HelixContain.h`
  - `GameLogic/Module/HighlanderBody.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HighlanderBody.h`
  - `GameLogic/Module/HijackerUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HijackerUpdate.h`
  - `GameLogic/Module/HiveStructureBody.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HiveStructureBody.h`
  - `GameLogic/Module/HordeUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HordeUpdate.h`
  - `GameLogic/Module/ImmortalBody.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ImmortalBody.h`
  - `GameLogic/Module/InactiveBody.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/InactiveBody.h`
  - `GameLogic/Module/InstantDeathBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/InstantDeathBehavior.h`
  - `GameLogic/Module/InternetHackContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/InternetHackContain.h`
  - `GameLogic/Module/JetAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/JetAIUpdate.h`
  - `GameLogic/Module/JetSlowDeathBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/JetSlowDeathBehavior.h`
  - `GameLogic/Module/KeepObjectDie.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/KeepObjectDie.h`
  - `GameLogic/Module/LaserUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/LaserUpdate.h`
  - `GameLogic/Module/LifetimeUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/LifetimeUpdate.h`
  - `GameLogic/Module/LockWeaponCreate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/LockWeaponCreate.h`
  - `GameLogic/Module/LocomotorSetUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/LocomotorSetUpgrade.h`
  - `GameLogic/Module/MaxHealthUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/MaxHealthUpgrade.h`
  - `GameLogic/Module/MinefieldBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/MinefieldBehavior.h`
  - `GameLogic/Module/MissileAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/MissileAIUpdate.h`
  - `GameLogic/Module/MissileLauncherBuildingUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/MissileLauncherBuildingUpdate.h`
  - `GameLogic/Module/MobMemberSlavedUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/MobMemberSlavedUpdate.h`
  - `GameLogic/Module/MobNexusContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/MobNexusContain.h`
  - `GameLogic/Module/ModelConditionUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ModelConditionUpgrade.h`
  - `GameLogic/Module/MoneyCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/MoneyCrateCollide.h`
  - `GameLogic/Module/NeutronBlastBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/NeutronBlastBehavior.h`
  - `GameLogic/Module/NeutronMissileSlowDeathUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/NeutronMissileSlowDeathUpdate.h`
  - `GameLogic/Module/NeutronMissileUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/NeutronMissileUpdate.h`
  - `GameLogic/Module/OCLSpecialPower.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OCLSpecialPower.h`
  - `GameLogic/Module/OCLUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OCLUpdate.h`
  - `GameLogic/Module/ObjectCreationUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ObjectCreationUpgrade.h`
  - `GameLogic/Module/OpenContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OpenContain.h`
  - `GameLogic/Module/OverchargeBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OverchargeBehavior.h`
  - `GameLogic/Module/OverlordContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OverlordContain.h`
  - `GameLogic/Module/POWTruckAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/POWTruckAIUpdate.h`
  - `GameLogic/Module/POWTruckBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/POWTruckBehavior.h`
  - `GameLogic/Module/ParachuteContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ParachuteContain.h`
  - `GameLogic/Module/ParkingPlaceBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ParkingPlaceBehavior.h`
  - `GameLogic/Module/ParticleUplinkCannonUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ParticleUplinkCannonUpdate.h`
  - `GameLogic/Module/PassengersFireUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PassengersFireUpgrade.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/PilotFindVehicleUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PilotFindVehicleUpdate.h`
  - `GameLogic/Module/PointDefenseLaserUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PointDefenseLaserUpdate.h`
  - `GameLogic/Module/PoisonedBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PoisonedBehavior.h`
  - `GameLogic/Module/PowerPlantUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PowerPlantUpdate.h`
  - `GameLogic/Module/PowerPlantUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PowerPlantUpgrade.h`
  - `GameLogic/Module/PreorderCreate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PreorderCreate.h`
  - `GameLogic/Module/PrisonBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PrisonBehavior.h`
  - `GameLogic/Module/PrisonDockUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PrisonDockUpdate.h`
  - `GameLogic/Module/ProductionUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ProductionUpdate.h`
  - `GameLogic/Module/ProjectileStreamUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ProjectileStreamUpdate.h`
  - `GameLogic/Module/ProneUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ProneUpdate.h`
  - `GameLogic/Module/PropagandaCenterBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PropagandaCenterBehavior.h`
  - `GameLogic/Module/PropagandaTowerBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PropagandaTowerBehavior.h`
  - `GameLogic/Module/QueueProductionExitUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/QueueProductionExitUpdate.h`
  - `GameLogic/Module/RadarUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RadarUpdate.h`
  - `GameLogic/Module/RadarUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RadarUpgrade.h`
  - `GameLogic/Module/RadiusDecalUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RadiusDecalUpdate.h`
  - `GameLogic/Module/RailedTransportAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RailedTransportAIUpdate.h`
  - `GameLogic/Module/RailedTransportContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RailedTransportContain.h`
  - `GameLogic/Module/RailedTransportDockUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RailedTransportDockUpdate.h`
  - `GameLogic/Module/RailroadGuideAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RailroadGuideAIUpdate.h`
  - `GameLogic/Module/RebuildHoleBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RebuildHoleBehavior.h`
  - `GameLogic/Module/RebuildHoleExposeDie.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RebuildHoleExposeDie.h`
  - `GameLogic/Module/RepairDockUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RepairDockUpdate.h`
  - `GameLogic/Module/ReplaceObjectUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ReplaceObjectUpgrade.h`
  - `GameLogic/Module/RiderChangeContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RiderChangeContain.h`
  - `GameLogic/Module/SabotageCommandCenterCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SabotageCommandCenterCrateCollide.h`
  - `GameLogic/Module/SabotageFakeBuildingCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SabotageFakeBuildingCrateCollide.h`
  - `GameLogic/Module/SabotageInternetCenterCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SabotageInternetCenterCrateCollide.h`
  - `GameLogic/Module/SabotageMilitaryFactoryCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SabotageMilitaryFactoryCrateCollide.h`
  - `GameLogic/Module/SabotagePowerPlantCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SabotagePowerPlantCrateCollide.h`
  - `GameLogic/Module/SabotageSuperweaponCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SabotageSuperweaponCrateCollide.h`
  - `GameLogic/Module/SabotageSupplyCenterCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SabotageSupplyCenterCrateCollide.h`
  - `GameLogic/Module/SabotageSupplyDropzoneCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SabotageSupplyDropzoneCrateCollide.h`
  - `GameLogic/Module/SalvageCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SalvageCrateCollide.h`
  - `GameLogic/Module/ShroudCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ShroudCrateCollide.h`
  - `GameLogic/Module/SlavedUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SlavedUpdate.h`
  - `GameLogic/Module/SlowDeathBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SlowDeathBehavior.h`
  - `GameLogic/Module/SmartBombTargetHomingUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SmartBombTargetHomingUpdate.h`
  - `GameLogic/Module/SpawnBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpawnBehavior.h`
  - `GameLogic/Module/SpawnPointProductionExitUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpawnPointProductionExitUpdate.h`
  - `GameLogic/Module/SpecialAbility.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialAbility.h`
  - `GameLogic/Module/SpecialAbilityUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialAbilityUpdate.h`
  - `GameLogic/Module/SpecialPowerCompletionDie.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialPowerCompletionDie.h`
  - `GameLogic/Module/SpecialPowerCreate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialPowerCreate.h`
  - `GameLogic/Module/SpectreGunshipDeploymentUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpectreGunshipDeploymentUpdate.h`
  - `GameLogic/Module/SpectreGunshipUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpectreGunshipUpdate.h`
  - `GameLogic/Module/SpyVisionSpecialPower.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpyVisionSpecialPower.h`
  - `GameLogic/Module/SpyVisionUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpyVisionUpdate.h`
  - `GameLogic/Module/SquishCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SquishCollide.h`
  - `GameLogic/Module/StatusBitsUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StatusBitsUpgrade.h`
  - `GameLogic/Module/StealthDetectorUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StealthDetectorUpdate.h`
  - `GameLogic/Module/StealthUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StealthUpdate.h`
  - `GameLogic/Module/StealthUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StealthUpgrade.h`
  - `GameLogic/Module/StickyBombUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StickyBombUpdate.h`
  - `GameLogic/Module/StructureBody.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StructureBody.h`
  - `GameLogic/Module/StructureCollapseUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StructureCollapseUpdate.h`
  - `GameLogic/Module/StructureToppleUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StructureToppleUpdate.h`
  - `GameLogic/Module/SubObjectsUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SubObjectsUpgrade.h`
  - `GameLogic/Module/SupplyCenterCreate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SupplyCenterCreate.h`
  - `GameLogic/Module/SupplyCenterDockUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SupplyCenterDockUpdate.h`
  - `GameLogic/Module/SupplyCenterProductionExitUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SupplyCenterProductionExitUpdate.h`
  - `GameLogic/Module/SupplyTruckAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SupplyTruckAIUpdate.h`
  - `GameLogic/Module/SupplyWarehouseCreate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SupplyWarehouseCreate.h`
  - `GameLogic/Module/SupplyWarehouseCripplingBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SupplyWarehouseCripplingBehavior.h`
  - `GameLogic/Module/SupplyWarehouseDockUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SupplyWarehouseDockUpdate.h`
  - `GameLogic/Module/TechBuildingBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/TechBuildingBehavior.h`
  - `GameLogic/Module/TensileFormationUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/TensileFormationUpdate.h`
  - `GameLogic/Module/ToppleUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ToppleUpdate.h`
  - `GameLogic/Module/TransitionDamageFX.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/TransitionDamageFX.h`
  - `GameLogic/Module/TransportAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/TransportAIUpdate.h`
  - `GameLogic/Module/TransportContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/TransportContain.h`
  - `GameLogic/Module/TunnelContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/TunnelContain.h`
  - `GameLogic/Module/UndeadBody.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UndeadBody.h`
  - `GameLogic/Module/UnitCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UnitCrateCollide.h`
  - `GameLogic/Module/UnpauseSpecialPowerUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UnpauseSpecialPowerUpgrade.h`
  - `GameLogic/Module/UpgradeDie.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpgradeDie.h`
  - `GameLogic/Module/VeterancyCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/VeterancyCrateCollide.h`
  - `GameLogic/Module/VeterancyGainCreate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/VeterancyGainCreate.h`
  - `GameLogic/Module/WanderAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/WanderAIUpdate.h`
  - `GameLogic/Module/WaveGuideUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/WaveGuideUpdate.h`
  - `GameLogic/Module/WeaponBonusUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/WeaponBonusUpdate.h`
  - `GameLogic/Module/WeaponBonusUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/WeaponBonusUpgrade.h`
  - `GameLogic/Module/WeaponSetUpgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/WeaponSetUpgrade.h`
  - `GameLogic/Module/WorkerAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/WorkerAIUpdate.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (1 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_common::core::dict`: `DICT_MAX_LEN`, `DictDataType`, `fn`, `DictValue`, `Dict`
- `zh_common::core::name_key_generator`: `SOCKET_COUNT`, `NAMEKEY_MAX`, `NameKeyGenerator`
- `zh_common::core::state_machine`: `StateId`, `MACHINE_DONE_STATE_ID`, `INVALID_STATE_ID`, `EXIT_MACHINE_WITH_SUCCESS`, `EXIT_MACHINE_WITH_FAILURE`, `STATE_CONTINUE_CODE`, `STATE_SUCCESS_CODE`, `STATE_FAILURE_CODE`, `STATE_SLEEP_FOREVER_FRAMES`, `MAX_TRANSITION_DEPTH`, `StateReturnType`, `fn`, `StateExitType`, `StateWorldView`, ... (+8)
- `zh_common::crc`: `Crc`, `fn`
- `zh_common::ini`: `INI_MAX_CHARS_PER_LINE`, `LOGICFRAMES_PER_SECOND`, `MSEC_PER_SECOND`, `LOGICFRAMES_PER_MSEC_REAL`, `MSEC_PER_LOGICFRAME_REAL`, `LOGICFRAMES_PER_SECONDS_REAL`, `SECONDS_PER_LOGICFRAME_REAL`, `RADS_PER_DEGREE`, `SEPS_DEFAULT`, `SEPS_PERCENT`, `SEPS_COLON`, `SEPS_QUOTE`, `END_TOKEN`, `Color`, ... (+22)
- `zh_common::math`: `fn`, `Coord2D`, `Coord3D`, `ICoord2D`, `ICoord3D`, `Region2D`, `Region3D`, `IRegion2D`, `IRegion3D`, `RealRange`, `RGBColor`, `RGBAColorReal`, `RGBAColorInt`, `Matrix3D`
- `zh_common::math::bez_fwd_iterator`: `BezFwdIterator`
- `zh_common::math::bezier_segment`: `USUAL_TOLERANCE`, `BEZ_BASIS_MATRIX`, `BezierMath`, `BezierSegment`
- `zh_common::math::discrete_circle`: `HorzLine`, `DiscreteCircle`
- `zh_common::math::gamemath`: `WWMATH_PI`, `WWMATH_TWO_PI`, `PI`, `TWO_PI`, `WWMATH_EPSILON`
- `zh_common::math::geometry`: `GeometryType`, `fn`, `GEOMETRY_NAMES`, `GeometryInfo`
- `zh_common::math::partition_solver`: `PairObjectIdAndUInt`, `PairObjectId`, `EntriesVec`, `SpacesVec`, `SolutionVec`, `SolutionType`, `PartitionSolver`
- `zh_common::random_value`: `INITIAL_SEED_WORDS`, `THE_MULT_FACTOR`, `DistributionType`, `PrngState`, `fn`
- `zh_common::types`: `Int`, `UnsignedInt`, `Short`, `UnsignedShort`, `Byte`, `UnsignedByte`, `Bool`, `Real`, `XferVersion`, `FALSE`, `TRUE`, `ObjectId`, `INVALID`, `TeamId`, ... (+11)
- `zh_common::xfer`: `XferMode`, `XferError`, `Xfer`, `Snapshot`, `XferCrc`, `fn`, `XferDeepCrc`, `XferSave`, `XferLoad`
- `zh_common::xfer::bit_flags`: `ARMOR_SET_BIT_NAMES`, `MODEL_CONDITION_BIT_NAMES`, `ArmorSetFlags`, `ModelConditionFlags`, `BitFlags`, `NUM_BITS`, `fn`
- `zh_logic::arena`: `ObjectSlot`, `INVALID`, `fn`, `ModuleSlot`, `ModuleDataId`, `ThingTemplateId`, `FacetMask`, `NONE`, `UPDATE`, `DIE`, `DAMAGE`, `CREATE`, `COLLIDE`, `BODY`, ... (+22)
- `zh_logic::object::experience_tracker`: `ExperienceParentAccess`, `ExperienceTrackerState`, `fn`
- `zh_logic::object::object_script_status_bits`: `ObjectScriptStatusBit`, `fn`, `OBJECT_STATUS_SCRIPT_DISABLED`, `OBJECT_STATUS_SCRIPT_UNPOWERED`, `OBJECT_STATUS_SCRIPT_UNSELLABLE`, `OBJECT_STATUS_SCRIPT_UNSTEALTHED`, `OBJECT_STATUS_SCRIPT_TARGETABLE`
- `zh_logic::object::object_types`: `ObjectTypes`
- `zh_logic::object::overridable`: `OverridableEntry`, `fn`
- `zh_logic::object::override`: `OverrideStore`, `Override`, `fn`
- `zh_logic::object::thing`: `INITIAL_THING_POS_X`, `INITIAL_THING_POS_Y`, `ThingTerrainAccess`, `ThingTransformReactor`, `NoopTransformReactor`, `Thing`, `fn`
- `zh_logic::object::thing_factory`: `ThingFactory`, `fn`
- `zh_logic::object::thing_template`: `LEVEL_COUNT`, `MAX_UPGRADE_CAMEO_UPGRADES`, `USE_EXP_VALUE_FOR_SKILL_VALUE`, `RADAR_PRIORITY_NAMES`, `MODULEINTERFACE_UPDATE`, `MODULEINTERFACE_DIE`, `MODULEINTERFACE_DAMAGE`, `MODULEINTERFACE_CREATE`, `MODULEINTERFACE_COLLIDE`, `MODULEINTERFACE_BODY`, `MODULEINTERFACE_CONTAIN`, `MODULEINTERFACE_UPGRADE`, `MODULEINTERFACE_SPECIAL_POWER`, `MODULEINTERFACE_DESTROY`, ... (+27)
- `zh_logic::object::weapon`: `NO_MAX_SHOTS_LIMIT`, `PATHFIND_CELL_SIZE_F`, `WEAPON_UNDERSIZE`, `WeaponReloadType`, `fn`, `WeaponPrefireType`, `WEAPON_RELOAD_NAMES`, `WEAPON_PREFIRE_NAMES`, `WEAPON_ANTI_AIRBORNE_VEHICLE`, `WEAPON_ANTI_GROUND`, `WEAPON_ANTI_PROJECTILE`, `WEAPON_ANTI_SMALL_MISSILE`, `WEAPON_ANTI_MINE`, `WEAPON_ANTI_AIRBORNE_INFANTRY`, ... (+22)
- `zh_logic::object::weapon_set`: `WEAPONSLOT_COUNT`, `WeaponSlotType`, `fn`, `WeaponSetConditionType`, `WeaponChoiceCriteria`, `WeaponLockType`, `CanAttackResult`, `WeaponTemplateSet`, `WEAPON_SET_TYPE_TO_MODEL_CONDITION_TYPE_MAP`, `WeaponSet`
- `zh_logic::rules`: `AcademyClassificationType`, `fn`, `ACADEMY_CLASSIFICATION_TYPE_NAMES`
- `zh_logic::rules::combat_types`: `ARMORSET_COUNT`, `ArmorSetFlags`, `ARMOR_SET_NAMES`, `ArmorSetType`, `fn`, `DAMAGE_NUM_TYPES`, `DamageTypeFlags`, `DAMAGE_TYPE_NAMES`, `DamageType`, `DEATH_NUM_TYPES`, `DeathTypeFlags`, `DEATH_TYPE_FLAGS_ALL`, `DEATH_TYPE_FLAGS_NONE`, `DEATH_TYPE_NAMES`, ... (+14)
- `zh_logic::rules::disabled_types`: `DISABLED_COUNT`, `DISABLED_ANY_SENTINEL`, `DisabledMaskType`, `DisabledType`, `fn`, `DISABLED_TYPE_NAMES`
- `zh_logic::rules::kind_of`: `KINDOF_COUNT`, `KindOfMaskType`, `KindOfType`, `fn`, `KINDOF_NAMES`
- `zh_logic::rules::model_state`: `MODELCONDITION_COUNT`, `ModelConditionFlags`, `MODEL_CONDITION_NAMES`, `ModelConditionFlagType`, `fn`
- `zh_logic::rules::multiplayer_settings`: `Color`, `PLAYERTEMPLATE_RANDOM`, `PLAYERTEMPLATE_OBSERVER`, `PLAYERTEMPLATE_MIN`, `GAME_COLOR_UNDEFINED`, `MultiplayerColorDefinition`, `MultiplayerColorList`, `MultiplayerStartingMoneyList`, `MultiplayerSettings`
- `zh_logic::rules::object_status_types`: `OBJECT_STATUS_COUNT`, `ObjectStatusMaskType`, `ObjectStatusType`, `fn`, `OBJECT_STATUS_NAMES`
- `zh_logic::rules::player_template`: `MAX_MP_STARTING_UNITS`, `VETERANCY_NAMES`, `ProductionChangeMap`, `ProductionVeterancyMap`, `PlayerTemplateImageLookup`, `PlayerTemplate`, `PlayerTemplateStore`
- `zh_logic::rules::production_prerequisite`: `UNIT_OR_WITH_PREV`, `MAX_PREREQ`, `ThingTemplateRef`, `ThingTemplateRegistry`, `PrereqPlayerView`, `PrereqTextLookup`, `ProductionPrerequisite`
- `zh_logic::rules::rank_info`: `RankInfoData`, `RankInfoStore`
- `zh_logic::rules::science`: `ScienceType`, `SCIENCE_INVALID`, `ScienceVec`, `SciencePlayerAccess`, `ScienceTextLookup`, `ScienceInfoData`, `ScienceStore`
- `zh_logic::rules::special_power`: `DEFAULT_DEFECTION_DETECTION_PROTECTION_TIME_LIMIT`, `SpecialPowerObjectAccess`, `SpecialPowerTemplateData`, `SpecialPowerTemplate`, `fn`, `SpecialPowerStore`
- `zh_logic::rules::special_power_types`: `SPECIALPOWER_COUNT`, `SpecialPowerMaskType`, `SpecialPowerType`, `fn`, `SPECIAL_POWER_NAMES`
- `zh_logic::rules::upgrade`: `UPGRADE_MAX_COUNT`, `UpgradeMaskType`, `UpgradeStatusType`, `fn`, `UpgradeType`, `UPGRADE_TYPE_NAMES`, `UpgradePlayerAccess`, `Upgrade`, `UpgradeTemplate`, `UpgradeCenter`
- `zh_logic::scheduler::sleepy_heap`: `UPDATE_SLEEP_INVALID`, `UPDATE_SLEEP_NONE`, `UPDATE_SLEEP_FOREVER`, `INVALID_LOGIC_INDEX`, `SleepyUpdatePhase`, `fn`, `UpdateModuleScheduleState`, `ScheduleStore`, `SleepyUpdateHeap`
- `zh_logic::combat::armor`: `ArmorTemplate`, `Armor`, `fn`, `ArmorStore`
- `zh_logic::combat::armor_set`: `ArmorTemplateSet`
- `zh_logic::combat::damage`: `PlayerMaskType`, `HUGE_DAMAGE_AMOUNT`, `DamageInfoInput`, `DamageInfoOutput`, `DamageInfo`
- `zh_logic::combat::locomotor`: `BIGNUM`, `LocomotorAppearance`, `fn`, `LOCOMOTOR_APPEARANCE_NAMES`, `LocomotorPriority`, `LOCOMOTOR_PRIORITY_NAMES`, `LocomotorBehaviorZ`, `LOCOMOTOR_BEHAVIOR_Z_NAMES`, `LocomotorTemplate`, `LocomotorStore`, `Locomotor`
- `zh_logic::combat::locomotor_set`: `LocomotorSet`
- `zh_logic::economy`: `MAX_PLAYER_COUNT`
- `zh_logic::economy::academy_stats`: `MAX_ADVICE_TIPS`, `FRAMES_BETWEEN_UPDATES`, `DEFAULT_SUPPLY_CENTER_COST`, `KINDOF_DOZER`, `KINDOF_FS_SUPPLY_CENTER`, `KINDOF_COMMANDCENTER`, `KINDOF_INFANTRY`, `KINDOF_VEHICLE`, `KINDOF_HARVESTER`, `KINDOF_HERO`, `KINDOF_FS_STRATEGY_CENTER`, `KINDOF_MONEY_HACKER`, `KINDOF_FS_BLACK_MARKET`, `KINDOF_FS_SUPPLY_DROPZONE`, ... (+9)
- `zh_logic::economy::energy`: `EnergyOwnerCallbacks`, `NoopEnergyCallbacks`, `EnergyObjectInfo`, `Energy`
- `zh_logic::economy::handicap`: `HANDICAP_TYPE_COUNT`, `THING_TYPE_COUNT`, `HandicapType`, `ThingType`, `Handicap`, `fn`
- `zh_logic::economy::mission_stats`: `MissionStats`
- `zh_logic::economy::money`: `INCOME_BUCKET_COUNT`, `LOGICFRAMES_PER_SECOND`, `MoneyCallbacks`, `NoopMoneyCallbacks`, `Money`
- `zh_logic::gamelogic::global_data`: `MAX_GLOBAL_LIGHTS`, `MAX_WATER_GRID_SETTINGS`, `SIMULATE_REPLAYS_SEQUENTIAL`, `DEFAULT_DISPLAY_WIDTH`, `DEFAULT_DISPLAY_HEIGHT`, `DEFAULT_JPEG_QUALITY`, `DEFAULT_DOUBLE_CLICK_TIME_MS`, `PLAYERTYPE_COUNT`, `DIFFICULTY_COUNT`, `LEVEL_COUNT`, `MULTISAMPLE_MODE_NONE`, `TEXTURE_FILTER_BILINEAR`, `TEXTURE_FILTER_ANISOTROPIC_2X`, `FIREWALL_TYPE_UNKNOWN`, ... (+30)
- `zh_logic::gamelogic::message_stream`: `TranslatorId`, `TRANSLATOR_ID_INVALID`, `MSG_COUNT_VALUE`, `GameMessageArgumentDataType`, `GameMessageArgumentValue`, `GameMessageArgument`, `GameMessageType`, `fn`, `MessageId`, `GameMessage`, `GameMessageList`, `CommandList`, `GameMessageDisposition`, `GameMessageTranslator`, ... (+1)

---

## Compact Worker Cheat Sheet (Embedded in All Translation Packets)

### 1. Primitive Type Mapping
| C++ Type | Rust Type | Size | Notes |
|---|---|---:|---|
| `Int` | `i32` | 4 | Two's-complement 32-bit signed |
| `UnsignedInt` | `u32` | 4 | 32-bit unsigned |
| `Short` | `i16` | 2 | 16-bit signed |
| `UnsignedShort` | `u16` | 2 | 16-bit unsigned |
| `Byte` / `char` | `i8` | 1 | Signed 8-bit (`char` is signed on x86/macOS) |
| `UnsignedByte` | `u8` | 1 | Unsigned 8-bit |
| `Bool` | `u8` (`Bool`) | 1 | `FALSE = 0`, `TRUE = 1` (1 byte in `Xfer`!) |
| `Real` | `f32` | 4 | IEEE-754 binary32, round-to-nearest-even, no FMA |
| `ObjectID` / `TeamID` / `DrawableID` | `ObjectId(pub i32)` etc. | 4 | `INVALID_ID = 0` |
| `NameKeyType` | `NameKeyType(pub i32)` | 4 | `NAMEKEY_INVALID = 0` |
| `UpdateSleepTime` | `UpdateSleepTime(pub u32)` | 4 | `NONE = 1`, `FOREVER = 0x3fff_ffff` |

### 2. Ten Non-Negotiable Determinism Rules
1. **Wrapping Integer Arithmetic:** Never use bare `+`, `-`, `*`, `+=`, `-=`, `*=` on `i32`/`u32` where C++ can overflow or wrap. Use `.wrapping_add()`, `.wrapping_sub()`, `.wrapping_mul()`, `.wrapping_neg()`.
2. **Unfused IEEE-754 `f32` Arithmetic:** Never call `f32::mul_add` (FMA). Keep exact C++ parenthesization and left-to-right operator grouping: `(a * b) * c + d` must not be rewritten as `a * (b * c) + d`.
3. **Deterministic Transcendentals via `GameMath`:** Never call standard `f32::sin`, `cos`, `tan`, `asin`, `acos`, `atan2`, `sqrt`, `floor`, `ceil`, `powf`, `exp`, or `ln`. Always call `zh_common::gamemath::{gm_sinf, gm_cosf, gm_tanf, gm_asinf, gm_acosf, gm_atan2f, gm_sqrtf, gm_floorf, gm_ceilf, gm_powf, gm_expf, gm_logf}`.
4. **Mixed `Int * Real` Promotion & Float-to-Int Truncation:**
   - In C++, `Int a; Real s; a *= s;` evaluates `a = static_cast<Int>(static_cast<Real>(a) * s)`.
   - In Rust, always write: `a = ((a as f32) * s) as i32;` (or `zh_common::math::cpp_mul_i32_f32(a, s)`).
   - For `REAL_TO_INT_FLOOR(x)` and `REAL_TO_INT_CEIL(x)`, call `zh_common::math::{real_to_int_floor, real_to_int_ceil}` (which route to `gm_floorf(x) as i32` / `gm_ceilf(x) as i32` in default deterministic mode, and `fast_float_floor`/`ceil` under `#[cfg(feature = "retail-crc")]`).
5. **No `HashMap` or `HashSet`:** Never import `std::collections::{HashMap, HashSet}`. Always use `std::collections::{BTreeMap, BTreeSet}` keyed by a deterministic integer ID (`ObjectId`, `NameKeyType`, `TeamId`, `u32`), never by raw pointer address.
6. **No `BinaryHeap` for Sleepy Updates:** Use `zh_logic::SleepyUpdateHeap` (strict `>` comparator, left-child tie-breaking, in-place root sifting, reverse-heap-index removal).
7. **Evaluation Order of Side-Effecting Arguments:** When a C++ function call has 2+ arguments that call `GameLogicRandomValue*` or mutate state, evaluate them into explicit `let arg0 = ...; let arg1 = ...;` bindings in left-to-right order before the call.
8. **Exact PRNG Call Parity:** `GameLogicRandomValue(lo, hi)` and `GameLogicRandomValueReal(lo, hi)` return `hi` **without advancing the seed** when `lo >= hi` (under default `RETAIL_COMPATIBLE_CRC == 0`). Never reorder, hoist, or short-circuit a branch that calls `GameLogicRandomValue*`.
9. **Exact `Xfer` & `XferCRC` Field Order and Widths:**
   - Serialize fields in the **exact** C++ `crc(Xfer*)` and `xfer(Xfer*)` order.
   - Never serialize `usize` or `isize`; cast lengths to `u16`, `u32`, or `i32` matching the C++ local variable type (`UnsignedShort`, `UnsignedInt`, `Int`).
   - Remember `Bool` is `1 byte` (`u8`), and enum values serialized via `xferUser(&e, sizeof(Enum))` are `4 bytes` (`i32`).
10. **Preserve `#if !RETAIL_COMPATIBLE_CRC` Guards:** Translate `#if RETAIL_COMPATIBLE_CRC` / `#if !RETAIL_COMPATIBLE_CRC` blocks using `#[cfg(feature = "retail-crc")]` and `#[cfg(not(feature = "retail-crc"))]`.

---

## 3. Reference C++ Source (`fbraz3/GeneralsX` at `84a7a20`)

### `GeneralsMD/Code/GameEngine/Include/Common/ModuleFactory.h`
```cpp
// FILE: ModuleFactory.h //////////////////////////////////////////////////////////////////////////
// Author: Colin Day, September 2001
// Desc:	 TheModuleFactory is where we actually instance modules for objects
//				 and drawables.  Those modules are things such as an UpdateModule
//			   or DamageModule or DrawModule etc.
//
//				 TheModuleFactory will contain a list of ModuleTemplates, when we
//				 request a new module, we will look for that template in our
//				 list and create it
//
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include <stdlib.h>

#include "Common/GameMemory.h"
#include "Common/SubsystemInterface.h"
#include "Common/AsciiString.h"
#include "Common/NameKeyGenerator.h"
#include "Common/Module.h"
#include "Common/STLTypedefs.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class Thing;
class Object;
class Drawable;
class INI;

// TYPE DEFINITIONS ///////////////////////////////////////////////////////////////////////////////
typedef Module *(*NewModuleProc)(Thing *thing, const ModuleData* moduleData);
typedef ModuleData* (*NewModuleDataProc)(INI* ini);

//-------------------------------------------------------------------------------------------------
/** We use TheModuleFactory to register classes that will be attached
	* to objects and drawables which will be executed or "called back" in the
	* correct situations ... such as Die, Damage, Update etc or just as
	* a place to store data specific to that type of thing */
//-------------------------------------------------------------------------------------------------
class ModuleFactory : public SubsystemInterface, public Snapshot
{

public:

	ModuleFactory();
	virtual ~ModuleFactory() override;

	virtual void init() override;
	virtual void reset() override { }					///< We don't reset during the lifetime of the app
	virtual void update() override { }					///< As of now, we don't have a need for an update

	Module *newModule( Thing *thing, const AsciiString& name, const ModuleData* data, ModuleType type );  ///< allocate a new module

	// module-data
	ModuleData* newModuleDataFromINI(INI* ini, const AsciiString& name, ModuleType type, const AsciiString& moduleTag);

	Int findModuleInterfaceMask(const AsciiString& name, ModuleType type);

	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

protected:


	class ModuleTemplate
	{
	public:
		ModuleTemplate() : m_createProc(nullptr), m_createDataProc(nullptr), m_whichInterfaces(0)
		{
		}

		NewModuleProc m_createProc;					///< creation method
		NewModuleDataProc m_createDataProc;	///< creation method
		Int m_whichInterfaces;
	};

	const ModuleTemplate* findModuleTemplate(const AsciiString& name, ModuleType type);

	/// adding a new module template to the factory, and assisting macro to make it easier
	void addModuleInternal( NewModuleProc proc, NewModuleDataProc dataproc, ModuleType type, const AsciiString& name, Int whichIntf );
	#define addModule( classname )											\
		addModuleInternal( classname::friend_newModuleInstance,  \
											 classname::friend_newModuleData,			\
											 classname::getModuleType(),		\
											 AsciiString( #classname ),			\
											 classname::getInterfaceMask())

	static NameKeyType makeDecoratedNameKey(const AsciiString& name, ModuleType type);

	typedef std::map< NameKeyType, ModuleTemplate, std::less<NameKeyType>/**/> ModuleTemplateMap;
	typedef std::vector<const ModuleData*> ModuleDataList;

	ModuleTemplateMap			m_moduleTemplateMap;
	ModuleDataList				m_moduleDataList;

};

// EXTERN /////////////////////////////////////////////////////////////////////////////////////////
extern ModuleFactory *TheModuleFactory;  ///< singleton definition
```

### `GeneralsMD/Code/GameEngine/Source/Common/Thing/ModuleFactory.cpp`
```cpp
// FILE: ModuleFactory.cpp ////////////////////////////////////////////////////////////////////////
// Author: Colin Day, September 2001
// Desc:	 TheModuleFactory is where we actually instance modules for objects
//				 and drawables.  Those modules are things such as an UpdateModule
//			   or DamageModule or DrawModule etc.
//
//				 TheModuleFactory will contain a list of ModuleTemplates, when we
//				 request a new module, we will look for that template in our
//				 list and create it
//
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/Module.h"
#include "Common/ModuleFactory.h"
#include "Common/NameKeyGenerator.h"

// behavior includes
#include "GameLogic/Module/AutoHealBehavior.h"
#include "GameLogic/Module/GrantStealthBehavior.h"
#include "GameLogic/Module/NeutronBlastBehavior.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BridgeBehavior.h"
#include "GameLogic/Module/BridgeScaffoldBehavior.h"
#include "GameLogic/Module/BridgeTowerBehavior.h"
#include "GameLogic/Module/CountermeasuresBehavior.h"
#include "GameLogic/Module/DumbProjectileBehavior.h"
#include "GameLogic/Module/InstantDeathBehavior.h"
#include "GameLogic/Module/SlowDeathBehavior.h"
#include "GameLogic/Module/HelicopterSlowDeathUpdate.h"
#include "GameLogic/Module/NeutronMissileSlowDeathUpdate.h"
#include "GameLogic/Module/CaveContain.h"
#include "GameLogic/Module/OpenContain.h"
#include "GameLogic/Module/OverchargeBehavior.h"
#include "GameLogic/Module/HealContain.h"
#include "GameLogic/Module/GarrisonContain.h"
#include "GameLogic/Module/InternetHackContain.h"
#include "GameLogic/Module/RailedTransportContain.h"
#include "GameLogic/Module/RiderChangeContain.h"
#include "GameLogic/Module/TransportContain.h"
#include "GameLogic/Module/MobNexusContain.h"
#include "GameLogic/Module/TunnelContain.h"
#include "GameLogic/Module/OverlordContain.h"
#include "GameLogic/Module/HelixContain.h"
#include "GameLogic/Module/ParachuteContain.h"
#ifdef ALLOW_SURRENDER
#include "GameLogic/Module/POWTruckBehavior.h"
#include "GameLogic/Module/PrisonBehavior.h"
#include "GameLogic/Module/PropagandaCenterBehavior.h"
#endif
#include "GameLogic/Module/PropagandaTowerBehavior.h"
#include "GameLogic/Module/BunkerBusterBehavior.h"
#include "GameLogic/Module/FireWeaponWhenDamagedBehavior.h"
#include "GameLogic/Module/FireWeaponWhenDeadBehavior.h"
#include "GameLogic/Module/GenerateMinefieldBehavior.h"
#include "GameLogic/Module/ParkingPlaceBehavior.h"
#include "GameLogic/Module/FlightDeckBehavior.h"
#include "GameLogic/Module/PoisonedBehavior.h"
#include "GameLogic/Module/RebuildHoleBehavior.h"
#include "GameLogic/Module/SupplyWarehouseCripplingBehavior.h"
#include "GameLogic/Module/TechBuildingBehavior.h"
#include "GameLogic/Module/MinefieldBehavior.h"
#include "GameLogic/Module/BattleBusSlowDeathBehavior.h"
#include "GameLogic/Module/JetSlowDeathBehavior.h"

// die includes
#include "GameLogic/Module/CreateCrateDie.h"
#include "GameLogic/Module/CreateObjectDie.h"
#include "GameLogic/Module/CrushDie.h"
#include "GameLogic/Module/DamDie.h"
#include "GameLogic/Module/DestroyDie.h"
#include "GameLogic/Module/EjectPilotDie.h"
#include "GameLogic/Module/FXListDie.h"
#include "GameLogic/Module/RebuildHoleExposeDie.h"
#include "GameLogic/Module/SpecialPowerCompletionDie.h"
#include "GameLogic/Module/UpgradeDie.h"
#include "GameLogic/Module/KeepObjectDie.h"

// logic update includes
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/AnimationSteeringUpdate.h"
#include "GameLogic/Module/AssistedTargetingUpdate.h"
#include "GameLogic/Module/BaseRegenerateUpdate.h"
#include "GameLogic/Module/BoneFXUpdate.h"
#include "GameLogic/Module/ChinookAIUpdate.h"
#include "GameLogic/Module/DefaultProductionExitUpdate.h"
#include "GameLogic/Module/DeletionUpdate.h"
#include "GameLogic/Module/SmartBombTargetHomingUpdate.h"
#include "GameLogic/Module/DeliverPayloadAIUpdate.h"
#include "GameLogic/Module/DozerAIUpdate.h"
#include "GameLogic/Module/DynamicGeometryInfoUpdate.h"
#include "GameLogic/Module/DynamicShroudClearingRangeUpdate.h"
#include "GameLogic/Module/EnemyNearUpdate.h"
#include "GameLogic/Module/FireSpreadUpdate.h"
#include "GameLogic/Module/FirestormDynamicGeometryInfoUpdate.h"
#include "GameLogic/Module/FireWeaponUpdate.h"
#include "GameLogic/Module/FlammableUpdate.h"
#include "GameLogic/Module/FloatUpdate.h"
#include "GameLogic/Module/TensileFormationUpdate.h"
#include "GameLogic/Module/HackInternetAIUpdate.h"
#include "GameLogic/Module/DeployStyleAIUpdate.h"
#include "GameLogic/Module/AssaultTransportAIUpdate.h"
#include "GameLogic/Module/HeightDieUpdate.h"
#include "GameLogic/Module/HordeUpdate.h"
#include "GameLogic/Module/JetAIUpdate.h"
#include "GameLogic/Module/LaserUpdate.h"
#include "GameLogic/Module/PointDefenseLaserUpdate.h"
#include "GameLogic/Module/CleanupHazardUpdate.h"
#include "GameLogic/Module/AutoFindHealingUpdate.h"
#include "GameLogic/Module/CommandButtonHuntUpdate.h"
#include "GameLogic/Module/PilotFindVehicleUpdate.h"
#include "GameLogic/Module/DemoTrapUpdate.h"
#include "GameLogic/Module/ParticleUplinkCannonUpdate.h"
#include "GameLogic/Module/SpectreGunshipUpdate.h"
#include "GameLogic/Module/SpectreGunshipDeploymentUpdate.h"
#include "GameLogic/Module/BaikonurLaunchPower.h"
#include "GameLogic/Module/BattlePlanUpdate.h"
#include "GameLogic/Module/LifetimeUpdate.h"
#include "GameLogic/Module/RadiusDecalUpdate.h"
#include "GameLogic/Module/AutoDepositUpdate.h"
#include "GameLogic/Module/MissileAIUpdate.h"
#include "GameLogic/Module/NeutronMissileUpdate.h"
#include "GameLogic/Module/OCLUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#ifdef ALLOW_SURRENDER
#include "GameLogic/Module/POWTruckAIUpdate.h"
#endif
#include "GameLogic/Module/ProductionUpdate.h"
#include "GameLogic/Module/ProjectileStreamUpdate.h"
#include "GameLogic/Module/ProneUpdate.h"
#include "GameLogic/Module/QueueProductionExitUpdate.h"
#include "GameLogic/Module/RadarUpdate.h"
#include "GameLogic/Module/RepairDockUpdate.h"
#ifdef ALLOW_SURRENDER
#include "GameLogic/Module/PrisonDockUpdate.h"
#endif
#include "GameLogic/Module/RailedTransportDockUpdate.h"
#include "GameLogic/Module/RailedTransportAIUpdate.h"
#include "GameLogic/Module/RailroadGuideAIUpdate.h"
#include "GameLogic/Module/SlavedUpdate.h"
#include "GameLogic/Module/MobMemberSlavedUpdate.h"
#include "GameLogic/Module/SpecialAbilityUpdate.h"
#include "GameLogic/Module/MissileLauncherBuildingUpdate.h"
#include "GameLogic/Module/StealthDetectorUpdate.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Module/SpawnPointProductionExitUpdate.h"
#include "GameLogic/Module/SpawnBehavior.h"
#include "GameLogic/Module/SpyVisionUpdate.h"
#include "GameLogic/Module/StickyBombUpdate.h"
#include "GameLogic/Module/FireOCLAfterWeaponCooldownUpdate.h"
#include "GameLogic/Module/HijackerUpdate.h"
#include "GameLogic/Module/StructureCollapseUpdate.h"
#include "GameLogic/Module/StructureToppleUpdate.h"
#include "GameLogic/Module/SupplyCenterDockUpdate.h"
#include "GameLogic/Module/SupplyCenterProductionExitUpdate.h"
#include "GameLogic/Module/SupplyTruckAIUpdate.h"
#include "GameLogic/Module/SupplyWarehouseDockUpdate.h"
#include "GameLogic/Module/ToppleUpdate.h"
#include "GameLogic/Module/TransportAIUpdate.h"
#include "GameLogic/Module/WanderAIUpdate.h"
#include "GameLogic/Module/WaveGuideUpdate.h"
#include "GameLogic/Module/WeaponBonusUpdate.h"
#include "GameLogic/Module/WorkerAIUpdate.h"
#include "GameLogic/Module/PowerPlantUpdate.h"
#include "GameLogic/Module/CheckpointUpdate.h"
#include "GameLogic/Module/EMPUpdate.h"

// upgrade includes
#include "GameLogic/Module/ActiveShroudUpgrade.h"
#include "GameLogic/Module/ArmorUpgrade.h"
#include "GameLogic/Module/CommandSetUpgrade.h"
#include "GameLogic/Module/GrantScienceUpgrade.h"
#include "GameLogic/Module/PassengersFireUpgrade.h"
#include "GameLogic/Module/LocomotorSetUpgrade.h"
#include "GameLogic/Module/ObjectCreationUpgrade.h"
#include "GameLogic/Module/RadarUpgrade.h"
#include "GameLogic/Module/PowerPlantUpgrade.h"
#include "GameLogic/Module/ReplaceObjectUpgrade.h"
#include "GameLogic/Module/ModelConditionUpgrade.h"
#include "GameLogic/Module/StatusBitsUpgrade.h"
#include "GameLogic/Module/SubObjectsUpgrade.h"
#include "GameLogic/Module/StealthUpgrade.h"
#include "GameLogic/Module/UnpauseSpecialPowerUpgrade.h"
#include "GameLogic/Module/WeaponBonusUpgrade.h"
#include "GameLogic/Module/WeaponSetUpgrade.h"
#include "GameLogic/Module/CostModifierUpgrade.h"
#include "GameLogic/Module/ExperienceScalarUpgrade.h"
#include "GameLogic/Module/MaxHealthUpgrade.h"

// create includes
#include "GameLogic/Module/LockWeaponCreate.h"
#include "GameLogic/Module/SupplyCenterCreate.h"
#include "GameLogic/Module/SupplyWarehouseCreate.h"
#include "GameLogic/Module/GrantUpgradeCreate.h"
#include "GameLogic/Module/PreorderCreate.h"
#include "GameLogic/Module/SpecialPowerCreate.h"
#include "GameLogic/Module/VeterancyGainCreate.h"

// damage includes
#include "GameLogic/Module/BoneFXDamage.h"
#include "GameLogic/Module/TransitionDamageFX.h"

// collide includes
#include "GameLogic/Module/FireWeaponCollide.h"
#include "GameLogic/Module/SquishCollide.h"

#include "GameLogic/Module/ConvertToCarBombCrateCollide.h"
#include "GameLogic/Module/ConvertToHijackedVehicleCrateCollide.h"
#include "GameLogic/Module/HealCrateCollide.h"
#include "GameLogic/Module/MoneyCrateCollide.h"
#include "GameLogic/Module/SabotageCommandCenterCrateCollide.h"
#include "GameLogic/Module/SabotageFakeBuildingCrateCollide.h"
#include "GameLogic/Module/SabotageInternetCenterCrateCollide.h"
#include "GameLogic/Module/SabotageMilitaryFactoryCrateCollide.h"
#include "GameLogic/Module/SabotagePowerPlantCrateCollide.h"
#include "GameLogic/Module/SabotageSuperweaponCrateCollide.h"
#include "GameLogic/Module/SabotageSupplyCenterCrateCollide.h"
#include "GameLogic/Module/SabotageSupplyDropzoneCrateCollide.h"
#include "GameLogic/Module/SalvageCrateCollide.h"
#include "GameLogic/Module/ShroudCrateCollide.h"
#include "GameLogic/Module/UnitCrateCollide.h"
#include "GameLogic/Module/VeterancyCrateCollide.h"

// body includes
#include "GameLogic/Module/InactiveBody.h"
#include "GameLogic/Module/ActiveBody.h"
#include "GameLogic/Module/HighlanderBody.h"
#include "GameLogic/Module/ImmortalBody.h"
#include "GameLogic/Module/StructureBody.h"
#include "GameLogic/Module/HiveStructureBody.h"
#include "GameLogic/Module/UndeadBody.h"

// contain includes
// (none)

// special power modules
#include "GameLogic/Module/CashHackSpecialPower.h"
#include "GameLogic/Module/DefectorSpecialPower.h"
#ifdef ALLOW_DEMORALIZE
#include "GameLogic/Module/DemoralizeSpecialPower.h"
#endif
#include "GameLogic/Module/OCLSpecialPower.h"
#include "GameLogic/Module/SpecialAbility.h"
#include "GameLogic/Module/SpyVisionSpecialPower.h"
#include "GameLogic/Module/CashBountyPower.h"
#include "GameLogic/Module/CleanupAreaPower.h"
#include "GameLogic/Module/FireWeaponPower.h"

// destroy includes
// (none)

// client update includes
#include "GameClient/Module/AnimatedParticleSysBoneClientUpdate.h"
#include "GameClient/Module/SwayClientUpdate.h"
#include "GameClient/Module/BeaconClientUpdate.h"

// PUBLIC DATA ////////////////////////////////////////////////////////////////////////////////////
ModuleFactory *TheModuleFactory = nullptr;  ///< the module factory singleton

// PUBLIC FUNCTIONS ///////////////////////////////////////////////////////////////////////////////


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ModuleFactory::ModuleFactory()
{
	m_moduleTemplateMap.clear();
	m_moduleDataList.clear();

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ModuleFactory::~ModuleFactory()
{
	m_moduleTemplateMap.clear();

	for (ModuleDataList::iterator i = m_moduleDataList.begin(); i != m_moduleDataList.end(); ++i)
	{
		const ModuleData* data = *i;
		delete data;
	}
	m_moduleDataList.clear();

}

//-------------------------------------------------------------------------------------------------
/** Initialize the module factory.  Any class that needs to be attached
	* to objects or drawables as modules needs to add a template
	* for that class here */
//-------------------------------------------------------------------------------------------------
void ModuleFactory::init()
{

	// behavior modules
	addModule( AutoHealBehavior );
	addModule( GrantStealthBehavior );
	addModule( NeutronBlastBehavior );
	addModule( BridgeBehavior );
	addModule( BridgeScaffoldBehavior );
	addModule( BridgeTowerBehavior );
	addModule( CountermeasuresBehavior );
	addModule( DumbProjectileBehavior );
	addModule( PhysicsBehavior );
	addModule( InstantDeathBehavior );
	addModule( SlowDeathBehavior );
	addModule( HelicopterSlowDeathBehavior );
	addModule( NeutronMissileSlowDeathBehavior );
	addModule( CaveContain );
	addModule( OpenContain );
	addModule( OverchargeBehavior );
	addModule( HealContain );
	addModule( GarrisonContain );
	addModule( InternetHackContain );
	addModule( TransportContain );
	addModule( RiderChangeContain );
	addModule( RailedTransportContain );
	addModule( MobNexusContain );
	addModule( TunnelContain );
	addModule( OverlordContain );
	addModule( HelixContain );
	addModule( ParachuteContain );
#ifdef ALLOW_SURRENDER
	addModule( POWTruckBehavior );
	addModule( PrisonBehavior );
	addModule( PropagandaCenterBehavior );
#endif
	addModule( PropagandaTowerBehavior );
	addModule( BunkerBusterBehavior );
	addModule( FireWeaponWhenDamagedBehavior );
	addModule( FireWeaponWhenDeadBehavior );
	addModule( GenerateMinefieldBehavior );
	addModule( ParkingPlaceBehavior );
	addModule( FlightDeckBehavior );
	addModule( PoisonedBehavior );
	addModule( RebuildHoleBehavior );
	addModule( SupplyWarehouseCripplingBehavior );
	addModule( TechBuildingBehavior );
	addModule( MinefieldBehavior );
	addModule( BattleBusSlowDeathBehavior );
	addModule( JetSlowDeathBehavior );
	addModule( RailroadBehavior );
	addModule( SpawnBehavior );

	// die modules
	addModule( DestroyDie );
	addModule( FXListDie );
	addModule( CrushDie );
	addModule( DamDie );
	addModule( CreateCrateDie );
	addModule( CreateObjectDie );
	addModule( EjectPilotDie );
	addModule( SpecialPowerCompletionDie );
	addModule( RebuildHoleExposeDie );
	addModule( UpgradeDie );
	addModule( KeepObjectDie );

	// update modules
	addModule( AssistedTargetingUpdate );
	addModule( AutoFindHealingUpdate );
	addModule( BaseRegenerateUpdate );
	addModule( StealthDetectorUpdate );
	addModule( StealthUpdate );
	addModule( DeletionUpdate );
	addModule( SmartBombTargetHomingUpdate );
	addModule( DynamicShroudClearingRangeUpdate );
	addModule( DeployStyleAIUpdate );
	addModule( AssaultTransportAIUpdate );
	addModule( HordeUpdate );
	addModule( ToppleUpdate );
	addModule( EnemyNearUpdate );
	addModule( LifetimeUpdate );
	addModule( RadiusDecalUpdate );
	addModule( EMPUpdate );
  addModule( LeafletDropBehavior );
	addModule( AutoDepositUpdate );
	addModule( WeaponBonusUpdate );
	addModule( MissileAIUpdate );
	addModule( NeutronMissileUpdate );
	addModule( FireSpreadUpdate );
	addModule( FireWeaponUpdate );
	addModule( FlammableUpdate );
	addModule( FloatUpdate );
	addModule( TensileFormationUpdate );
	addModule( HeightDieUpdate );
	addModule( ChinookAIUpdate );
	addModule( JetAIUpdate );
	addModule( AIUpdateInterface );
	addModule( SupplyTruckAIUpdate );
	addModule( DeliverPayloadAIUpdate );
	addModule( HackInternetAIUpdate );
	addModule( DynamicGeometryInfoUpdate );
	addModule( FirestormDynamicGeometryInfoUpdate );
	addModule( LaserUpdate );
	addModule( PointDefenseLaserUpdate );
	addModule( CleanupHazardUpdate );
	addModule( CommandButtonHuntUpdate );
	addModule( PilotFindVehicleUpdate );
	addModule( DemoTrapUpdate );
	addModule( ParticleUplinkCannonUpdate );
	addModule( SpectreGunshipUpdate );
	addModule( SpectreGunshipDeploymentUpdate );
	addModule( BaikonurLaunchPower );
	addModule( BattlePlanUpdate );
	addModule( ProjectileStreamUpdate );
	addModule( QueueProductionExitUpdate );
	addModule( RepairDockUpdate );
#ifdef ALLOW_SURRENDER
	addModule( PrisonDockUpdate );
#endif
	addModule( RailedTransportDockUpdate );
	addModule( DefaultProductionExitUpdate );
	addModule( SpawnPointProductionExitUpdate );
	addModule( SpyVisionUpdate );
	addModule( SlavedUpdate );
	addModule( MobMemberSlavedUpdate );
	addModule( OCLUpdate );
	addModule( SpecialAbilityUpdate );
	addModule( MissileLauncherBuildingUpdate );
	addModule( SupplyCenterProductionExitUpdate );
	addModule( SupplyCenterDockUpdate );
	addModule( SupplyWarehouseDockUpdate );
	addModule( DozerAIUpdate );
#ifdef ALLOW_SURRENDER
	addModule( POWTruckAIUpdate );
#endif
	addModule( RailedTransportAIUpdate );
	addModule( ProductionUpdate );
	addModule( ProneUpdate );
	addModule( StickyBombUpdate );
	addModule( FireOCLAfterWeaponCooldownUpdate );
	addModule( HijackerUpdate );
	addModule( StructureToppleUpdate );
	addModule( StructureCollapseUpdate );
	addModule( BoneFXUpdate );
	addModule( RadarUpdate );
	addModule( AnimationSteeringUpdate );
	addModule( TransportAIUpdate );
	addModule( WanderAIUpdate );
	addModule( WaveGuideUpdate );
	addModule( WorkerAIUpdate );
	addModule( PowerPlantUpdate );
	addModule( CheckpointUpdate );

	// upgrade modules
	addModule( CostModifierUpgrade );
	addModule( ActiveShroudUpgrade );
	addModule( ArmorUpgrade );
	addModule( CommandSetUpgrade );
	addModule( GrantScienceUpgrade );
	addModule( PassengersFireUpgrade );
	addModule( StatusBitsUpgrade );
	addModule( SubObjectsUpgrade );
	addModule( StealthUpgrade );
	addModule( RadarUpgrade );
	addModule( PowerPlantUpgrade );
	addModule( LocomotorSetUpgrade );
	addModule( ObjectCreationUpgrade );
	addModule( ReplaceObjectUpgrade );
	addModule( ModelConditionUpgrade );
	addModule( UnpauseSpecialPowerUpgrade );
	addModule( WeaponBonusUpgrade );
	addModule( WeaponSetUpgrade );
	addModule( WeaponBonusUpgrade );
	addModule( ExperienceScalarUpgrade );
	addModule( MaxHealthUpgrade );

	// create modules
	addModule( LockWeaponCreate );
	addModule( PreorderCreate );
	addModule( SupplyCenterCreate );
	addModule( SupplyWarehouseCreate );
	addModule( SpecialPowerCreate );
	addModule( GrantUpgradeCreate );
	addModule( VeterancyGainCreate );

	// damage modules
	addModule( BoneFXDamage );
	addModule( TransitionDamageFX );

	// collide modules
	addModule( FireWeaponCollide );
	addModule( SquishCollide );

	addModule( HealCrateCollide );
	addModule( MoneyCrateCollide );
	addModule( ShroudCrateCollide );
	addModule( UnitCrateCollide );
	addModule( VeterancyCrateCollide );
	addModule( ConvertToCarBombCrateCollide );
	addModule( ConvertToHijackedVehicleCrateCollide );
	addModule( SabotageCommandCenterCrateCollide );
	addModule( SabotageFakeBuildingCrateCollide );
	addModule( SabotageInternetCenterCrateCollide );
	addModule( SabotageMilitaryFactoryCrateCollide );
	addModule( SabotagePowerPlantCrateCollide );
	addModule( SabotageSuperweaponCrateCollide );
	addModule( SabotageSupplyCenterCrateCollide );
	addModule( SabotageSupplyDropzoneCrateCollide );
	addModule( SalvageCrateCollide );

	// body modules
	addModule( InactiveBody );
	addModule( ActiveBody );
	addModule( HighlanderBody );
	addModule( ImmortalBody );
	addModule( StructureBody );
	addModule( HiveStructureBody );
	addModule( UndeadBody );

	// contain modules
	// (none)

	// special power modules
	addModule( CashHackSpecialPower );
	addModule( DefectorSpecialPower );
#ifdef ALLOW_DEMORALIZE
	addModule( DemoralizeSpecialPower );
#endif
	addModule( OCLSpecialPower );
	addModule( FireWeaponPower );
	addModule( SpecialAbility );
	addModule( SpyVisionSpecialPower );
	addModule( CashBountyPower );
	addModule( CleanupAreaPower );

	// destroy modules
	// (none)

	// client update modules
	addModule( AnimatedParticleSysBoneClientUpdate );
	addModule( SwayClientUpdate );
	addModule( BeaconClientUpdate );

}

//-------------------------------------------------------------------------------------------------
Int ModuleFactory::findModuleInterfaceMask(const AsciiString& name, ModuleType type)
{
	if (name.isEmpty())
		return 0;

	const ModuleTemplate* moduleTemplate = findModuleTemplate(name, type);
	if (moduleTemplate)
	{
		return moduleTemplate->m_whichInterfaces;
	}

	return 0;
}

//-------------------------------------------------------------------------------------------------
ModuleData* ModuleFactory::newModuleDataFromINI(INI* ini, const AsciiString& name, ModuleType type,
																								const AsciiString& moduleTag)
{
	if (name.isEmpty())
		return nullptr;

	const ModuleTemplate* moduleTemplate = findModuleTemplate(name, type);
	if (moduleTemplate)
	{
		ModuleData* md = (*moduleTemplate->m_createDataProc)(ini);
		md->setModuleTagNameKey( NAMEKEY( moduleTag ) );
		m_moduleDataList.push_back(md);
		return md;
	}

	return nullptr;
}

// PRIVATE FUNCTIONS //////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
/*static*/ NameKeyType ModuleFactory::makeDecoratedNameKey(const AsciiString& name, ModuleType type)
{
	char tmp[256];
	tmp[0] = '0' + (int)type;
	strcpy(&tmp[1], name.str());
	return TheNameKeyGenerator->nameToKey(tmp);
}

//-------------------------------------------------------------------------------------------------
const ModuleFactory::ModuleTemplate* ModuleFactory::findModuleTemplate(const AsciiString& name, ModuleType type)
{
	NameKeyType namekey = makeDecoratedNameKey(name, type);

  ModuleTemplateMap::const_iterator it = m_moduleTemplateMap.find(namekey);
  if (it == m_moduleTemplateMap.end())
	{
		DEBUG_CRASH(( "Module name '%s' not found", name.str() ));
		return nullptr;
	}
	else
	{
		return &(*it).second;
	}
}

//-------------------------------------------------------------------------------------------------
/** Allocate a new acton class instance given the name */
//-------------------------------------------------------------------------------------------------
Module *ModuleFactory::newModule( Thing *thing, const AsciiString& name, const ModuleData* moduleData, ModuleType type )
{
	// sanity
	if( name.isEmpty() )
	{
		DEBUG_CRASH(("attempting to create module with empty name"));
		return nullptr;
	}
	const ModuleTemplate* mt = findModuleTemplate(name, type);
	if (mt)
	{
		Module* mod = (*mt->m_createProc)( thing, moduleData );

#ifdef DEBUG_CRASHING
		if (type == MODULETYPE_BEHAVIOR)
		{
			BehaviorModule* bm = (BehaviorModule*)mod;

			DEBUG_ASSERTCRASH(
				((mt->m_whichInterfaces & (MODULEINTERFACE_BODY)) != 0) == (bm->getBody() != nullptr),
				("getInterfaceMask bad for MODULE_BODY (%s)",name.str()));
			DEBUG_ASSERTCRASH(
				((mt->m_whichInterfaces & (MODULEINTERFACE_COLLIDE)) != 0) == (bm->getCollide() != nullptr),
				("getInterfaceMask bad for MODULE_COLLIDE (%s)",name.str()));
			DEBUG_ASSERTCRASH(
				((mt->m_whichInterfaces & (MODULEINTERFACE_CONTAIN)) != 0) == (bm->getContain() != nullptr),
				("getInterfaceMask bad for MODULE_CONTAIN (%s)",name.str()));
			DEBUG_ASSERTCRASH(
				((mt->m_whichInterfaces & (MODULEINTERFACE_CREATE)) != 0) == (bm->getCreate() != nullptr),
				("getInterfaceMask bad for MODULE_CREATE (%s)",name.str()));
			DEBUG_ASSERTCRASH(
				((mt->m_whichInterfaces & (MODULEINTERFACE_DAMAGE)) != 0) == (bm->getDamage() != nullptr),
				("getInterfaceMask bad for MODULE_DAMAGE (%s)",name.str()));
			DEBUG_ASSERTCRASH(
				((mt->m_whichInterfaces & (MODULEINTERFACE_DESTROY)) != 0) == (bm->getDestroy() != nullptr),
				("getInterfaceMask bad for MODULE_DESTROY (%s)",name.str()));
			DEBUG_ASSERTCRASH(
				((mt->m_whichInterfaces & (MODULEINTERFACE_DIE)) != 0) == (bm->getDie() != nullptr),
				("getInterfaceMask bad for MODULE_DIE (%s)",name.str()));
			DEBUG_ASSERTCRASH(
				((mt->m_whichInterfaces & (MODULEINTERFACE_SPECIAL_POWER)) != 0) == (bm->getSpecialPower() != nullptr),
				("getInterfaceMask bad for MODULE_SPECIAL_POWER (%s)",name.str()));
			DEBUG_ASSERTCRASH(
				((mt->m_whichInterfaces & (MODULEINTERFACE_UPDATE)) != 0) == (bm->getUpdate() != nullptr),
				("getInterfaceMask bad for MODULE_UPDATE (%s)",name.str()));
			DEBUG_ASSERTCRASH(
				((mt->m_whichInterfaces & (MODULEINTERFACE_UPGRADE)) != 0) == (bm->getUpgrade() != nullptr),
				("getInterfaceMask bad for MODULE_UPGRADE (%s)",name.str()));
		}
#endif

		return mod;
	}

	return nullptr;

}

//-------------------------------------------------------------------------------------------------
/** Add a module template to our list of templates */
//-------------------------------------------------------------------------------------------------
void ModuleFactory::addModuleInternal( NewModuleProc proc, NewModuleDataProc dataproc, ModuleType type, const AsciiString& name, Int whichIntf )
{
	NameKeyType namekey = makeDecoratedNameKey(name, type);
	ModuleTemplate& mtm = m_moduleTemplateMap[namekey];	// this creates it if it does not exist already
	mtm.m_createProc = proc;
	mtm.m_createDataProc = dataproc;
	mtm.m_whichInterfaces = whichIntf;
}

//-------------------------------------------------------------------------------------------------
void ModuleFactory::crc( Xfer *xfer )
{
	for (ModuleDataList::iterator mdIt = m_moduleDataList.begin(); mdIt != m_moduleDataList.end(); ++mdIt)
	{
		((ModuleData *)(*mdIt))->crc(xfer);
	}
}

//-------------------------------------------------------------------------------------------------
void ModuleFactory::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	for (ModuleDataList::iterator mdIt = m_moduleDataList.begin(); mdIt != m_moduleDataList.end(); ++mdIt)
	{
		((ModuleData *)(*mdIt))->xfer(xfer);
	}
}

//-------------------------------------------------------------------------------------------------
void ModuleFactory::loadPostProcess()
{
}
```
