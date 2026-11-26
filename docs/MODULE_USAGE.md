# Behavior modules the replays create

Written by `python3 scripts/module_census.py`. Do not edit by hand.

Counts are module instances over the 6 local replays. They show what is
created, not how often it runs, and cover behavior modules only (not AI, pathfinding
or weapon code outside modules). The replays do not match their recordings
(`BASELINE.md`), so they are a sample of play, not the original games.

- Classes created: 104; in every replay: 50.
- Classes with a ported packet: 71 of 104; in every replay: 39 of 50.
- Instances with a ported packet: 12515 of 14355.

## Not ported

| Module | Packet | Replays | Instances | C++ bytes in packet |
|---|---|---|---|---|
| `ProductionUpdate` | `PKT-354` | 6 | 347 | 60927 |
| `AIUpdateInterface` | `PKT-301` | 6 | 178 | 221431 |
| `AutoDepositUpdate` | `PKT-305` | 6 | 156 | 15532 |
| `DefaultProductionExitUpdate` | `PKT-315` | 6 | 138 | 12787 |
| `GenerateMinefieldBehavior` | `PKT-272` | 6 | 137 | 26449 |
| `SpecialAbilityUpdate` | `PKT-368` | 6 | 123 | 85607 |
| `CommandButtonHuntUpdate` | `PKT-314` | 6 | 108 | 19021 |
| `DozerAIUpdate` | `PKT-321` | 6 | 45 | 105594 |
| `SpawnBehavior` | `PKT-296` | 6 | 23 | 46341 |
| `SupplyCenterProductionExitUpdate` | `PKT-379` | 6 | 18 | 13279 |
| `QueueProductionExitUpdate` | `PKT-357` | 6 | 11 | 16553 |
| `MinefieldBehavior` | `PKT-279` | 5 | 53 | 29795 |
| `ToppleUpdate` | `PKT-383` | 4 | 186 | 22132 |
| `SpectreGunshipDeploymentUpdate` | `PKT-370` | 4 | 64 | 18180 |
| `SupplyTruckAIUpdate` | `PKT-380` | 4 | 26 | 40352 |
| `AutoFindHealingUpdate` | `PKT-306` | 4 | 20 | 10536 |
| `ChinookAIUpdate` | `PKT-312` | 4 | 18 | 61770 |
| `HelicopterSlowDeathBehavior` | `PKT-334` | 4 | 18 | 26512 |
| `AssistedTargetingUpdate` | `PKT-304` | 4 | 6 | 10759 |
| `HordeUpdate` | `PKT-336` | 3 | 50 | 21101 |
| `WorkerAIUpdate` | `PKT-389` | 3 | 30 | 69258 |
| `SlavedUpdate` | `PKT-365` | 3 | 12 | 34735 |
| `TransportAIUpdate` | `PKT-384` | 3 | 11 | 11159 |
| `ParkingPlaceBehavior` | `PKT-286` | 3 | 4 | 41782 |
| `SpawnPointProductionExitUpdate` | `PKT-367` | 3 | 4 | 13341 |
| `StructureToppleUpdate` | `PKT-377` | 2 | 12 | 31223 |
| `TensileFormationUpdate` | `PKT-382` | 2 | 12 | 16588 |
| `MissileLauncherBuildingUpdate` | `PKT-341` | 2 | 3 | 19292 |
| `DumbProjectileBehavior` | `PKT-267` | 1 | 14 | 36155 |
| `HackInternetAIUpdate` | `PKT-332` | 1 | 8 | 29279 |
| `PointDefenseLaserUpdate` | `PKT-350` | 1 | 3 | 16970 |
| `BattlePlanUpdate` | `PKT-309` | 1 | 1 | 43241 |
| `OCLUpdate` | `PKT-346` | 1 | 1 | 14839 |
