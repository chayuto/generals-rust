# Behavior modules the replays create

Written by `python3 scripts/module_census.py`. Do not edit by hand.

Counts are module instances over the 6 local replays. They show what is
created, not how often it runs, and cover behavior modules only (not AI, pathfinding
or weapon code outside modules). The replays do not match their recordings
(`BASELINE.md`), so they are a sample of play, not the original games.

- Classes created: 104; in every replay: 50.
- Classes with a ported packet: 97 of 104; in every replay: 48 of 50.
- Instances with a ported packet: 14039 of 14355.

## Not ported

| Module | Packet | Replays | Instances | C++ bytes in packet |
|---|---|---|---|---|
| `AIUpdateInterface` | `PKT-301` | 6 | 178 | 221431 |
| `DozerAIUpdate` | `PKT-321` | 6 | 45 | 105594 |
| `SupplyTruckAIUpdate` | `PKT-380` | 4 | 26 | 40352 |
| `ChinookAIUpdate` | `PKT-312` | 4 | 18 | 61770 |
| `WorkerAIUpdate` | `PKT-389` | 3 | 30 | 69258 |
| `TransportAIUpdate` | `PKT-384` | 3 | 11 | 11159 |
| `HackInternetAIUpdate` | `PKT-332` | 1 | 8 | 29279 |
