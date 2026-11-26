# Open Decisions and Known Port Gaps

Found while porting modules mechanically. Each needs an owner decision or a later spine change. Delete an entry once it is resolved.

## Decisions

1. **Where dock state lives.** The dock updates are unported. `setDockOpen` / `setDockCrippled` are recorded in `SimWorld::dock_closed` and `SimWorld::dock_crippled` as stand-ins (used by `PKT-292`, `PKT-297`). Options: a dock facet, or a side table.
2. **Default `XferSave` version.** The reference library is built with `RETAIL_COMPATIBLE_XFER_SAVE=1`, so `PoisonedBehavior` saves version 2. With `retail-xfer-save` off, the crate writes version 3. `golden_wp53_c1` strips the appended source id and compares it separately. Should default features match 0 or 1?
3. **AI random-number draws.** `AIUpdateInterface` is unported. Its draws are imitated in `sim/mod.rs` (`ai_construct`, `ai_init_state_machine`, `ai_idle`). The real port must take them over.

## Known gaps

- `set_object_team` does not call `on_capture` on any module. C++ `Object::setTeam` does, so capture-aware modules on a dying tech building miss the call.
- A module cannot be re-entered while it executes. `SupplyWarehouseCripplingBehavior` and `TechBuildingBehavior` apply their own self-callbacks inline instead.
- `BridgeScaffoldBehavior`: zero speed on a zero-length leg gives NaN positions. The reference then destroys the object from outside the module; Rust does not. `gen_wp53_c1.cpp` excludes those inputs, so no vector covers this.
- An unknown `PulseFX` name stays set in Rust but becomes null in C++.
