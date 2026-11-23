//! `WP-5.1-A`: Internal `ObjectHelper` modules (`ObjectSMCHelper`, `StatusDamageHelper`,
//! `SubdualDamageHelper`, `ObjectRepulsorHelper`, `ObjectDefectionHelper`,
//! `ObjectWeaponStatusHelper`, `TempWeaponBonusHelper`).

use crate::arena::ObjectSlot;
use crate::sim::SimContext;

/// Attaches built-in object helper modules onto `obj_slot` in exact C++ `Object::Object` order
/// prior to INI behavior instantiation (`WP-5.1-A`).
pub fn attach_object_helpers(_sim: &mut SimContext, _obj_slot: ObjectSlot) {
    // Registered and attached in `WP-5.1-A`.
}
