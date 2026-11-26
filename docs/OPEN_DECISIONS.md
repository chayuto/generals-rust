# Open Decisions and Known Port Gaps

Found while porting modules mechanically. Each needs an owner decision or a later spine change. Delete an entry once it is resolved.

## Decisions

1. **Where dock state lives.** The dock updates are unported. `setDockOpen` / `setDockCrippled` are recorded in `SimWorld::dock_closed` and `SimWorld::dock_crippled` as stand-ins (used by `PKT-292`, `PKT-297`). Options: a dock facet, or a side table.
2. **Default `XferSave` version.** The reference library is built with `RETAIL_COMPATIBLE_XFER_SAVE=1`, so `PoisonedBehavior` saves version 2. With `retail-xfer-save` off, the crate writes version 3. `golden_wp53_c1` strips the appended source id and compares it separately. Should default features match 0 or 1?
3. **AI random-number draws.** `AIUpdateInterface` is unported. Its draws are imitated in `sim/mod.rs` (`ai_construct`, `ai_init_state_machine`, `ai_idle`). The real port must take them over.
4. **Shared and redirected passenger lists.** `ContainFacet::get_contain_count`, `get_contain_max`, `get_contained_items`, `is_contained`, `is_garrisonable` and `is_kick_out_on_capture` take only `&self`. `TunnelContain` and `CaveContain` answer them from a tracker in `SimWorld`; `OverlordContain` forwards them to its rider's contain module. Neither is reachable without a view. Blocks `PKT-264`, `PKT-284`, `PKT-300`. The two trackers also lack `TunnelTracker`'s `isValidContainerFor`, `getContainMax`, `healObjects`, `updateNemesis` and cave-in handling.
5. **Containment override hooks.** `ContainBehavior` cannot express `addToContain`, `addToContainList`, `removeFromContain`, `isEnclosingContainerFor` or `isSpecialOverlordStyleContainer`, and the base functions in `open_contain.rs` call each other directly where C++ calls virtually. `HelixContain` (`PKT-275`) implements the facets itself. Should the trait gain these hooks? `OverlordContain` overrides the same set.

## Known gaps

- `set_object_team` does not call `on_capture` on any module. C++ `Object::setTeam` does, so capture-aware modules on a dying tech building miss the call.
- A module cannot be re-entered while it executes. `SupplyWarehouseCripplingBehavior` and `TechBuildingBehavior` apply their own self-callbacks inline instead.
- `BridgeScaffoldBehavior`: zero speed on a zero-length leg gives NaN positions. The reference then destroys the object from outside the module; Rust does not. `gen_wp53_c1.cpp` excludes those inputs, so no vector covers this.
- An unknown `PulseFX` name stays set in Rust but becomes null in C++.
- `destroy_object` runs the contain removal at once but `on_delete` only in `process_destroy_list`. C++ `Object::onDestroy` runs both inside `destroyObject`, so a destroyed container's riders stay alive until the end of the frame here. `gen_wp53_c2.cpp` never destroys a container outright.
- A body damage state change does not set the `DAMAGED`, `REALLYDAMAGED` or `RUBBLE` model conditions (`Drawable::reactToBodyDamageStateChange` in C++). `OpenContain` saves these flags and redeploys its riders when they change, so a damaged container differs in `Xfer` bytes and rider positions. `golden_wp53_c2` sets the flags itself.
