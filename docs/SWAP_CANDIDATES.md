# Which ported modules can run in the C++ engine

Written by `python3 scripts/swap_candidates.py`. Do not edit by hand.

`OPEN_DECISIONS.md` item 10. **Data** methods return a reference into
Rust-owned simulation state, which a callback cannot answer. A module with none
can go in through the adapter as the traits stand. **Methods** is how many
`SimView`/`SimApi` methods it can reach, each needing a C++ callback. The list
comes from source text and errs long; it does not follow dispatch to other
modules (**other**).

- Classes analysed: 97; with no data method: 16.
- Instances analysed: 14039; in classes with no data method: 975.
- Classes not found in the Rust source: AIUpdateInterface, ChinookAIUpdate, DozerAIUpdate, HackInternetAIUpdate, SupplyTruckAIUpdate, TransportAIUpdate, WorkerAIUpdate.

## How many classes each data method blocks

| Method | Classes | Instances | Only blocker for |
|---|---|---|---|
| `object` | 79 | 12605 | 23 |
| `rules` | 56 | 10188 | 1 |
| `object_mut` | 37 | 7434 | 1 |
| `player_battle_plans` | 20 | 4844 | 0 |
| `weapon_store_state` | 20 | 4844 | 0 |
| `cave_tracker` | 12 | 1952 | 0 |
| `cave_tracker_mut` | 12 | 1952 | 0 |
| `tunnel_tracker` | 12 | 1952 | 0 |
| `tunnel_tracker_mut` | 12 | 1952 | 0 |
| `player_battle_plans_mut` | 1 | 1 | 0 |

## No data method

| Module | Instances | Replays | Methods | Services | Other | Data methods |
|---|---|---|---|---|---|---|
| `SupplyWarehouseCreate` | 224 | 6 | 1 | - | 0 | - |
| `ArmorUpgrade` | 201 | 6 | 1 | - | 0 | - |
| `DefaultProductionExitUpdate` | 138 | 6 | 15 | - | 0 | - |
| `RadarUpdate` | 77 | 6 | 6 | - | 0 | - |
| `ModelConditionUpgrade` | 56 | 5 | 1 | - | 0 | - |
| `WeaponBonusUpgrade` | 51 | 5 | 1 | - | 0 | - |
| `WeaponSetUpgrade` | 42 | 5 | 1 | - | 0 | - |
| `StealthUpgrade` | 41 | 4 | 3 | - | 0 | - |
| `PowerPlantUpdate` | 35 | 6 | 6 | - | 0 | - |
| `LocomotorSetUpgrade` | 34 | 3 | 1 | - | 0 | - |
| `ExperienceScalarUpgrade` | 32 | 4 | 1 | - | 0 | - |
| `LockWeaponCreate` | 18 | 4 | 1 | - | 0 | - |
| `SupplyCenterCreate` | 18 | 6 | 1 | - | 0 | - |
| `MaxHealthUpgrade` | 4 | 1 | 2 | - | 0 | - |
| `SpecialPowerCreate` | 3 | 2 | 1 | - | 0 | - |
| `LifetimeUpdate` | 1 | 1 | 9 | - | 0 | - |

## Blocked

| Module | Instances | Replays | Methods | Services | Other | Data methods |
|---|---|---|---|---|---|---|
| `SlowDeathBehavior` | 1429 | 6 | 64 | client, terrain | 1 | object, object_mut, player_battle_plans, rules, weapon_store_state |
| `DestroyDie` | 1016 | 6 | 134 | client, terrain | 4 | cave_tracker, cave_tracker_mut, object, object_mut, player_battle_plans, rules, tunnel_tracker, tunnel_tracker_mut, weapon_store_state |
| `FXListDie` | 842 | 6 | 7 | - | 1 | object, rules |
| `OCLSpecialPower` | 774 | 6 | 71 | client, terrain | 1 | object, object_mut, player_battle_plans, rules, weapon_store_state |
| `ActiveBody` | 754 | 6 | 20 | - | 0 | object, object_mut, rules |
| `FlammableUpdate` | 705 | 6 | 12 | - | 0 | object |
| `TransitionDamageFX` | 635 | 6 | 5 | - | 0 | object |
| `StealthUpdate` | 610 | 6 | 39 | - | 0 | object, rules |
| `AutoHealBehavior` | 449 | 6 | 16 | - | 1 | object, rules |
| `CreateObjectDie` | 404 | 6 | 134 | client, terrain | 4 | cave_tracker, cave_tracker_mut, object, object_mut, player_battle_plans, rules, tunnel_tracker, tunnel_tracker_mut, weapon_store_state |
| `PhysicsBehavior` | 379 | 6 | 65 | client, terrain | 0 | object, object_mut, player_battle_plans, rules, weapon_store_state |
| `ProductionUpdate` | 347 | 6 | 33 | - | 0 | object, object_mut, rules |
| `InactiveBody` | 346 | 6 | 2 | - | 0 | object_mut |
| `StructureBody` | 327 | 6 | 20 | - | 0 | object, object_mut, rules |
| `ImmortalBody` | 224 | 6 | 20 | - | 0 | object, object_mut, rules |
| `SupplyWarehouseDockUpdate` | 224 | 6 | 16 | - | 1 | object, rules |
| `CommandSetUpgrade` | 203 | 6 | 5 | - | 0 | object, rules |
| `KeepObjectDie` | 188 | 6 | 134 | client, terrain | 4 | cave_tracker, cave_tracker_mut, object, object_mut, player_battle_plans, rules, tunnel_tracker, tunnel_tracker_mut, weapon_store_state |
| `ToppleUpdate` | 186 | 4 | 21 | - | 0 | object, rules |
| `TechBuildingBehavior` | 178 | 6 | 11 | - | 0 | object |
| `SpecialAbility` | 160 | 6 | 71 | client, terrain | 1 | object, object_mut, player_battle_plans, rules, weapon_store_state |
| `AutoDepositUpdate` | 156 | 6 | 14 | - | 0 | object |
| `PoisonedBehavior` | 142 | 6 | 6 | - | 0 | object |
| `SquishCollide` | 141 | 6 | 10 | - | 0 | object, object_mut, rules |
| `GenerateMinefieldBehavior` | 137 | 6 | 17 | terrain | 0 | object, object_mut, rules |
| `SpecialAbilityUpdate` | 123 | 6 | 70 | client | 0 | object, object_mut, rules |
| `StealthDetectorUpdate` | 119 | 6 | 17 | - | 0 | object |
| `UnpauseSpecialPowerUpgrade` | 113 | 6 | 2 | - | 0 | rules |
| `CommandButtonHuntUpdate` | 108 | 6 | 31 | - | 0 | object, rules |
| `PreorderCreate` | 106 | 6 | 4 | - | 0 | object |
| `CreateCrateDie` | 105 | 6 | 134 | client, terrain | 4 | cave_tracker, cave_tracker_mut, object, object_mut, player_battle_plans, rules, tunnel_tracker, tunnel_tracker_mut, weapon_store_state |
| `FireWeaponWhenDeadBehavior` | 96 | 6 | 47 | client, terrain | 1 | object, object_mut, player_battle_plans, rules, weapon_store_state |
| `BaseRegenerateUpdate` | 92 | 5 | 8 | - | 0 | object, rules |
| `CashBountyPower` | 87 | 4 | 71 | client, terrain | 1 | object, object_mut, player_battle_plans, rules, weapon_store_state |
| `RadarUpgrade` | 80 | 6 | 4 | - | 0 | object |
| `GrantUpgradeCreate` | 79 | 6 | 2 | - | 0 | object |
| `GarrisonContain` | 78 | 6 | 88 | terrain | 8 | cave_tracker, cave_tracker_mut, object, rules, tunnel_tracker, tunnel_tracker_mut |
| `VeterancyGainCreate` | 69 | 3 | 3 | - | 0 | object |
| `SpectreGunshipDeploymentUpdate` | 64 | 4 | 23 | terrain | 1 | object, object_mut, rules |
| `FireSpreadUpdate` | 54 | 2 | 8 | - | 0 | object |
| `FireWeaponCollide` | 54 | 2 | 106 | client, terrain | 3 | cave_tracker, cave_tracker_mut, object, object_mut, player_battle_plans, rules, tunnel_tracker, tunnel_tracker_mut, weapon_store_state |
| `HighlanderBody` | 54 | 2 | 20 | - | 0 | object, object_mut, rules |
| `MinefieldBehavior` | 53 | 5 | 26 | terrain | 0 | object, rules |
| `HordeUpdate` | 50 | 3 | 20 | client | 0 | object, rules |
| `RebuildHoleExposeDie` | 49 | 4 | 134 | client, terrain | 4 | cave_tracker, cave_tracker_mut, object, object_mut, player_battle_plans, rules, tunnel_tracker, tunnel_tracker_mut, weapon_store_state |
| `ObjectCreationUpgrade` | 42 | 3 | 2 | - | 0 | object |
| `CashHackSpecialPower` | 41 | 6 | 71 | client, terrain | 1 | object, object_mut, player_battle_plans, rules, weapon_store_state |
| `GrantScienceUpgrade` | 36 | 4 | 3 | - | 0 | object, rules |
| `TransportContain` | 29 | 4 | 94 | terrain | 8 | cave_tracker, cave_tracker_mut, object, object_mut, rules, tunnel_tracker, tunnel_tracker_mut |
| `OverchargeBehavior` | 25 | 6 | 11 | - | 0 | object |
| `RepairDockUpdate` | 23 | 6 | 11 | - | 1 | object |
| `SpawnBehavior` | 23 | 6 | 38 | - | 0 | object, object_mut |
| `AutoFindHealingUpdate` | 20 | 4 | 11 | - | 0 | object |
| `HealContain` | 20 | 6 | 82 | terrain | 8 | cave_tracker, cave_tracker_mut, object, rules, tunnel_tracker, tunnel_tracker_mut |
| `HelicopterSlowDeathBehavior` | 18 | 4 | 76 | client, terrain | 1 | object, object_mut, player_battle_plans, rules, weapon_store_state |
| `SupplyCenterDockUpdate` | 18 | 6 | 19 | terrain | 1 | object, rules |
| `SupplyCenterProductionExitUpdate` | 18 | 6 | 16 | terrain | 0 | object |
| `DumbProjectileBehavior` | 14 | 1 | 66 | client, terrain | 0 | object, object_mut, player_battle_plans, rules, weapon_store_state |
| `SpyVisionUpdate` | 13 | 3 | 9 | - | 0 | object, rules |
| `EjectPilotDie` | 12 | 3 | 4 | - | 0 | object |
| `FireWeaponWhenDamagedBehavior` | 12 | 2 | 49 | client, terrain | 1 | object, object_mut, player_battle_plans, rules, weapon_store_state |
| `MoneyCrateCollide` | 12 | 2 | 5 | - | 1 | object, rules |
| `SlavedUpdate` | 12 | 3 | 36 | client, terrain | 0 | object |
| `StructureToppleUpdate` | 12 | 2 | 58 | client, terrain | 0 | object, object_mut, player_battle_plans, rules, weapon_store_state |
| `TensileFormationUpdate` | 12 | 2 | 20 | terrain | 0 | object |
| `QueueProductionExitUpdate` | 11 | 6 | 23 | terrain | 0 | object |
| `PowerPlantUpgrade` | 10 | 4 | 6 | - | 0 | object, rules |
| `AssistedTargetingUpdate` | 6 | 4 | 17 | client | 0 | object, object_mut, rules |
| `InternetHackContain` | 6 | 3 | 94 | terrain | 8 | cave_tracker, cave_tracker_mut, object, object_mut, rules, tunnel_tracker, tunnel_tracker_mut |
| `CostModifierUpgrade` | 4 | 2 | 3 | - | 0 | object |
| `HiveStructureBody` | 4 | 3 | 23 | - | 0 | object, object_mut, rules |
| `ParkingPlaceBehavior` | 4 | 3 | 26 | - | 0 | object, object_mut, rules |
| `SpawnPointProductionExitUpdate` | 4 | 3 | 16 | - | 0 | object |
| `MissileLauncherBuildingUpdate` | 3 | 2 | 11 | - | 0 | object, rules |
| `PointDefenseLaserUpdate` | 3 | 1 | 47 | client, terrain | 0 | object, object_mut, player_battle_plans, rules, weapon_store_state |
| `CrushDie` | 2 | 2 | 134 | client, terrain | 4 | cave_tracker, cave_tracker_mut, object, object_mut, player_battle_plans, rules, tunnel_tracker, tunnel_tracker_mut, weapon_store_state |
| `PropagandaTowerBehavior` | 2 | 2 | 21 | - | 0 | object, rules |
| `BattlePlanUpdate` | 1 | 1 | 34 | - | 0 | object, player_battle_plans_mut, rules |
| `OCLUpdate` | 1 | 1 | 11 | terrain | 0 | object |
| `SpyVisionSpecialPower` | 1 | 1 | 71 | client, terrain | 1 | object, object_mut, player_battle_plans, rules, weapon_store_state |
| `TunnelContain` | 1 | 1 | 86 | terrain | 8 | cave_tracker, cave_tracker_mut, object, object_mut, rules, tunnel_tracker, tunnel_tracker_mut |
