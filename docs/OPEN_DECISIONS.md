# Open Decisions and Known Port Gaps

Found while porting modules mechanically. Each needs an owner decision or a later spine change. Delete an entry once it is resolved.

## Decisions

1. **Where dock state lives.** The dock updates are unported. `setDockOpen` / `setDockCrippled` are recorded in `SimWorld::dock_closed` and `SimWorld::dock_crippled` as stand-ins (used by `PKT-292`, `PKT-297`). Options: a dock facet, or a side table.
2. **Default `XferSave` version.** The reference library is built with `RETAIL_COMPATIBLE_XFER_SAVE=1`, so `PoisonedBehavior` saves version 2. With `retail-xfer-save` off, the crate writes version 3. `golden_wp53_c1` strips the appended source id and compares it separately. Should default features match 0 or 1?
3. **AI random-number draws.** `AIUpdateInterface` is unported. Its draws are imitated in `sim/mod.rs` (`ai_construct`, `ai_init_state_machine`, `ai_idle`). The real port must take them over.
4. **Shared and redirected passenger lists.** `ContainFacet::get_contain_count`, `get_contain_max`, `get_contained_items`, `is_contained`, `is_garrisonable` and `is_kick_out_on_capture` take only `&self`. `TunnelContain` and `CaveContain` answer them from a tracker in `SimWorld`; `OverlordContain` forwards them to its rider's contain module. Neither is reachable without a view. Blocks `PKT-264`, `PKT-284`, `PKT-300`. The two trackers also lack `TunnelTracker`'s `isValidContainerFor`, `getContainMax`, `healObjects`, `updateNemesis` and cave-in handling.
5. **Containment override hooks.** `ContainBehavior` cannot express `addToContain`, `addToContainList`, `removeFromContain`, `isEnclosingContainerFor` or `isSpecialOverlordStyleContainer`, and the base functions in `open_contain.rs` call each other directly where C++ calls virtually. `HelixContain` (`PKT-275`) implements the facets itself. Should the trait gain these hooks? `OverlordContain` overrides the same set.
6. **`LandMineFacet::set_scoot_parms` has no view.** It takes `(&mut self, start, end)`, but C++ `setScootParms` reads terrain, gravity and module data, moves the object and wakes the module, so it needs `sim` and `ctx`. Blocks `PKT-279 MinefieldBehavior` and `PKT-272 GenerateMinefieldBehavior`, which calls it. `PKT-279` also needs a home for `AIUpdateInterface::isClearingMines`, calls into `AutoHealBehavior::stopHealing` and `BodyModuleInterface::internalChangeHealth`, `GeometryInfo::clipPointToFootprint`, and a way for its own `onDamage` and `onDie` to run when it damages itself.
7. **Minefield placement queries.** `PKT-272 GenerateMinefieldBehavior` needs `PartitionManager::iteratePotentialCollisions` (a geometry overlap query), `GeometryInfo` footprint operations (`expandFootprint`, `getFootprintArea`, `isPointInFootprint`, and `makeRandomOffsetWithinFootprint`, which draws from the PRNG) and `TerrainLogic::isCliffCell`. None are ported.
8. **Where a bridge's tower table lives.** `BridgeTowerBehavior` (`PKT-262`) asks the bridge module through `BridgeFacet::get_tower_id`, as C++ does. `link_bridge_tower` writes a second table, `SimWorld::bridge_links`, which nothing reads. The `BridgeBehavior` port should settle on one.
9. **Where barrel counts come from.** `Weapon::privateFireWeapon` reads `Drawable::getBarrelCount`, which comes from the model's bones, and the result is saved with the weapon. The port loads no models, so `ObjectRecord::weapon_barrel_count` stays 0 outside tests. Replay parity needs the counts from somewhere.

## Known gaps

- A module cannot be re-entered while it executes. `SupplyWarehouseCripplingBehavior` and `TechBuildingBehavior` apply their own self-callbacks inline instead.
- `BridgeScaffoldBehavior`: zero speed on a zero-length leg gives NaN positions. The reference then destroys the object from outside the module; Rust does not. `gen_wp53_c1.cpp` excludes those inputs, so no vector covers this.
- An unknown `PulseFX` name stays set in Rust but becomes null in C++.
- `capture_object` / `set_object_team` leave out the rest of `Object::onCapture`: `cancelAllTasks` on a dozer, clearing the `SCRIPT_UNSELLABLE` status and a skirmish AI selling a captured faction building.
- With `retail-crc`, `WeaponTemplate::get_delay_between_shots` returns the maximum when the minimum exceeds it; the retail PRNG helper draws a value there. The default build matches.
- `physics_apply_force` and `physics_apply_motive_force` are stand-ins: they add force over mass to velocity. C++ adds to acceleration, keeps only the lateral part while motive, and ignores NaN.
- `FloatUpdate` over dry ground: C++ writes an uninitialised `waterZ` to the object, so there is no value to match. The port writes 0.
- `ProneUpdate` skips the model condition for an object without a drawable, where C++ would crash.
- `PKT-307`, `PKT-343` and `PKT-364` are header-only manifest entries for modules ported under another packet (`PKT-308`, `PKT-281`, `PKT-233`). `PORT_MAP.md` still lists them as unported.
- `WeaponTemplate::fireWeaponTemplate` is unported. `fire_weapon_template` only records the shot, so range checks, projectiles, damage and the two logic PRNG draws for a weapon with `ScatterRadius` or infantry inaccuracy are missing. `golden_wp55_fire` replaces the C++ function the same way, so no vector covers it.
- `player_relationship` models player-to-player relationships only. C++ `Team::getRelationship` and `Player::getRelationship` first look up per-team and per-player overrides (`m_teamRelations`, `m_playerRelations`); (The port's implicit self-alliance matches the usual setup: `PlayerList::newGame` sets each player as its own ally.)
- `FiringTracker` is attached but nothing ported reports a shot to it (`Object::fireCurrentWeapon` is reached from the unported AI), so it only sleeps. Before it is wired, its sim host needs `reload_all_ammo` to use `load_ammo_now`/`reload_ammo` as `Object::reloadAllAmmo` does (it now only fills the clip), and its `xfer`/`load_post_process` need the live sleepy schedule. `golden_wp3_3b_combat_leaves` covers the tracker against a mock host only.
- `Weapon::onWeaponBonusChange` is unported. C++ retimes the weapons in a set when bonus flags change, which can draw from the logic PRNG.
- `set_weapon_set_flag` does not rebuild the weapon set, and spawn picks the set with empty flags. C++ `updateWeaponSet` uses the object's flags and runs again on every change.
- A weapon that sits in a weapon set has to be fired on a copy and written back. No production code does this yet.
- Weapon firing leaves out `DAMAGE_DEPLOY`'s assault order, `processRequestAssistance` and the victim effects of `DAMAGE_DISARM`.
- **`PKT-216 FireWeaponPower` is not bit-exact**, although `PORT_MAP.md` counts it. It fires at once, where C++ reloads all ammo and orders the unported AI to attack.
- `Weapon::transfer_next_shot_stats_from` copies the stored status; C++ copies `getStatus()`, which refreshes it first. It has no callers.
- `set_geometry_info` stores the geometry and re-registers with the partition grid, but does not tell the drawable as C++ `Object::setGeometryInfo` does.
- `ProjectileStreamUpdate::add_projectile` has no caller until weapon projectiles are ported.
- `RadiusDecalUpdate::create_radius_decal` has no caller, and takes a template name because decal templates are not ported.

## Waiting on unported code

- `PKT-266 CountermeasuresBehavior`: `PhysicsBehavior` (`getVelocityMagnitude`, `transferVelocityTo`, a real `applyMotiveForce`) and the `Vector2` normalise and rotate math. Its constructor also leaves `m_reloadFrame` unset.
- `PKT-267 DumbProjectileBehavior`: `Weapon::positionProjectileForLaunch`, `PartitionManager::estimateTerrainExtremesAlongLine`, `WeaponTemplate::shouldProjectileCollideWith` and `WeaponStore::handleProjectileDetonation`.
- `PKT-305 AutoDepositUpdate`: a hook for `Player::becomingTeamMember`, a per-player `ScoreKeeper`, construction percent on objects, and a home for the upgrade template that `getUpgradedSupplyBoost` caches across all instances.
- `PKT-325 EnemyNearUpdate`, `PKT-311 CheckpointUpdate`: `AI::findClosestEnemy` / `findClosestAlly`.
- `PKT-327 FireSpreadUpdate`, `PKT-330 FlammableUpdate`: they call each other (`tryToIgnite`, `wouldIgnite`, `startFireSpreading`), and no facet carries those calls.
