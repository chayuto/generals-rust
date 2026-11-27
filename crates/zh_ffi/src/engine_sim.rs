//! The engine as seen by a ported module: `SimView`/`SimApi` answered by C++ callbacks.
//!
//! One `EngineSim` is bound to one engine object for the length of one call. The
//! trait impls are generated (`engine_sim_generated.rs`); only the methods below
//! reach the engine, and any other method aborts the game with its name.

use core::ffi::c_void;
use zh_common::types::{Bool, Int, Real, UnsignedInt, UpdateSleepTime};
use zh_logic::arena::{ModuleContext, ModuleDataId, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::{DamageInfo, DamageInfoInput};
use zh_logic::rules::combat_types::ArmorSetType;
use zh_logic::rules::kind_of::KindOfType;
use zh_logic::rules::model_state::ModelConditionFlagType;

/// Mirrors `ZhEngineSimCallbacks` in `zh_ffi.h`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ZhEngineSimCallbacks {
    pub ctx: *mut c_void,
    pub frame: extern "C" fn(*mut c_void) -> u32,
    pub rng_logic_int: extern "C" fn(*mut c_void, i32, i32) -> i32,
    pub set_wake_frame: extern "C" fn(*mut c_void, u32),
    pub hulk_max_lifetime_override: extern "C" fn(*mut c_void) -> i32,
    pub object_is_kind_of: extern "C" fn(*mut c_void, i32) -> u8,
    pub object_max_health: extern "C" fn(*mut c_void) -> f32,
    pub attempt_damage: extern "C" fn(*mut c_void, i32, i32, i32, f32, u8),
    pub object_has_drawable: extern "C" fn(*mut c_void) -> u8,
    pub set_model_condition_state: extern "C" fn(*mut c_void, i32),
    pub clear_model_condition_state: extern "C" fn(*mut c_void, i32),
    pub set_armor_set_flag: extern "C" fn(*mut c_void, i32, u8),
    pub register_supply_warehouse: extern "C" fn(*mut c_void),
}

/// Called by every method the adapter does not answer.
pub(crate) fn not_wired(method: &str) -> ! {
    eprintln!("zh_ffi: SimApi::{method} is not connected to the engine");
    std::process::abort();
}

/// The stand-in handles for "the one object and module this call is about".
const OWNER: ObjectSlot = ObjectSlot(0);
const MODULE: ModuleSlot = ModuleSlot(0);

pub struct EngineSim<'a> {
    callbacks: &'a ZhEngineSimCallbacks,
}

impl<'a> EngineSim<'a> {
    pub fn new(callbacks: &'a ZhEngineSimCallbacks) -> Self {
        Self { callbacks }
    }

    /// The context to hand the module together with this sim.
    pub fn context() -> ModuleContext {
        ModuleContext {
            mod_slot: MODULE,
            owner_slot: OWNER,
            owner_id: zh_common::types::ObjectId::INVALID,
            tag_key: Default::default(),
            template_data_id: ModuleDataId::INVALID,
        }
    }

    pub(crate) fn engine_frame(&self) -> UnsignedInt {
        (self.callbacks.frame)(self.callbacks.ctx)
    }

    pub(crate) fn engine_rng_logic_int(&mut self, lo: Int, hi: Int) -> Int {
        (self.callbacks.rng_logic_int)(self.callbacks.ctx, lo, hi)
    }

    pub(crate) fn engine_set_wake_frame(&mut self, slot: ModuleSlot, sleep: UpdateSleepTime) {
        assert_eq!(slot, MODULE, "only the calling module can be woken");
        (self.callbacks.set_wake_frame)(self.callbacks.ctx, sleep.0);
    }

    pub(crate) fn engine_hulk_max_lifetime_override(&self) -> Int {
        (self.callbacks.hulk_max_lifetime_override)(self.callbacks.ctx)
    }

    pub(crate) fn engine_object_is_kind_of(&self, owner: ObjectSlot, kindof: KindOfType) -> Bool {
        assert_eq!(owner, OWNER, "only the owning object can be queried");
        (self.callbacks.object_is_kind_of)(self.callbacks.ctx, kindof as i32)
    }

    pub(crate) fn engine_object_max_health(&self, owner: ObjectSlot) -> Real {
        assert_eq!(owner, OWNER, "only the owning object can be queried");
        (self.callbacks.object_max_health)(self.callbacks.ctx)
    }

    pub(crate) fn engine_object_has_drawable(&self, owner: ObjectSlot) -> Bool {
        assert_eq!(owner, OWNER, "only the owning object can be queried");
        (self.callbacks.object_has_drawable)(self.callbacks.ctx)
    }

    pub(crate) fn engine_set_model_condition_state(
        &mut self,
        owner: ObjectSlot,
        flag: ModelConditionFlagType,
    ) {
        assert_eq!(owner, OWNER, "only the owning object can be changed");
        (self.callbacks.set_model_condition_state)(self.callbacks.ctx, flag as i32);
    }

    pub(crate) fn engine_clear_model_condition_state(
        &mut self,
        owner: ObjectSlot,
        flag: ModelConditionFlagType,
    ) {
        assert_eq!(owner, OWNER, "only the owning object can be changed");
        (self.callbacks.clear_model_condition_state)(self.callbacks.ctx, flag as i32);
    }

    pub(crate) fn engine_set_armor_set_flag(
        &mut self,
        owner: ObjectSlot,
        flag: ArmorSetType,
        set: Bool,
    ) {
        assert_eq!(owner, OWNER, "only the owning object can be changed");
        (self.callbacks.set_armor_set_flag)(self.callbacks.ctx, flag as i32, set);
    }

    pub(crate) fn engine_register_supply_warehouse(&mut self, owner: ObjectSlot) {
        assert_eq!(owner, OWNER, "only the owning object can be registered");
        (self.callbacks.register_supply_warehouse)(self.callbacks.ctx);
    }

    /// Carries the fields `Object::kill` sets. A request that sets any other
    /// input field is refused, so nothing is dropped without notice.
    pub(crate) fn engine_attempt_damage(&mut self, target: ObjectSlot, dmg: &mut DamageInfo) {
        assert_eq!(target, OWNER, "only the owning object can be damaged");
        let input = &dmg.input;
        let carried = DamageInfoInput {
            source_id: input.source_id,
            damage_type: input.damage_type,
            death_type: input.death_type,
            amount: input.amount,
            kill: input.kill,
            ..DamageInfoInput::default()
        };
        assert!(*input == carried, "damage field the adapter does not carry");
        (self.callbacks.attempt_damage)(
            self.callbacks.ctx,
            input.damage_type as i32,
            input.death_type as i32,
            input.source_id.0,
            input.amount,
            input.kill,
        );
    }
}
