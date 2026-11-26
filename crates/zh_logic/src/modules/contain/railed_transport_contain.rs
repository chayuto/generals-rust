//! `PKT-292`: `RailedTransportContain` (`RailedTransportContain.h`, `RailedTransportContain.cpp`).
//!
//! Packet: `packets/phase5/PKT-292_RailedTransportContain.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_c1.rs`.
//!
//! The dock update is not ported; its open flag is read and written through
//! `SimView::dock_is_open` / `SimApi::set_dock_open`.

use crate::arena::{ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::xfer_module_version;
use crate::modules::contain::open_contain::{
    impl_contain_module, kill_object, remove_from_contain, ContainBehavior, OpenContainCore,
    OpenContainModuleData,
};
use crate::modules::contain::transport_contain::{
    transport_exit_busy, transport_is_valid_container_for, transport_on_capture,
    transport_on_containing, transport_on_removing, transport_passenger_allowed_to_fire,
    transport_update, transport_xfer, TransportBehavior, TransportContainModuleData,
    TransportContainState,
};
use crate::modules::module::{BehaviorModule, ExitDoorType, ModuleData};
use crate::rules::combat_types::{DamageType, DeathType};
use crate::sim::{SimApi, SimView};
use zh_common::types::{Bool, Int, ObjectId, TeamId, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// Runtime instance of `RailedTransportContain` (`PKT-292`). It adds no fields or INI data.
pub struct RailedTransportContain {
    pub data: TransportContainModuleData,
    pub state: TransportContainState,
}

impl RailedTransportContain {
    pub fn new(data: TransportContainModuleData) -> Self {
        let state = TransportContainState::new(data.open.passengers_allowed_to_fire);
        Self { data, state }
    }
}

impl TransportBehavior for RailedTransportContain {
    fn transport_data(&self) -> &TransportContainModuleData {
        &self.data
    }
    fn transport(&self) -> &TransportContainState {
        &self.state
    }
    fn transport_mut(&mut self) -> &mut TransportContainState {
        &mut self.state
    }
}

impl ContainBehavior for RailedTransportContain {
    fn open_data(&self) -> &OpenContainModuleData {
        &self.data.open
    }
    fn open_core(&self) -> &OpenContainCore {
        &self.state.core
    }
    fn open_core_mut(&mut self) -> &mut OpenContainCore {
        &mut self.state.core
    }
    fn contain_max_value(&self) -> Int {
        self.data.slot_capacity
    }
    fn xfer_module(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        transport_xfer(&mut self.state, xfer, sim)
    }
    fn update_module(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        transport_update(self, sim, ctx)
    }
    fn is_valid_container(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        rider: ObjectSlot,
        check_capacity: Bool,
    ) -> Bool {
        transport_is_valid_container_for(self, sim, ctx, rider, check_capacity)
    }
    fn passenger_allowed_to_fire(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        id: ObjectId,
    ) -> Bool {
        transport_passenger_allowed_to_fire(&self.state.core, sim, ctx, id)
    }
    fn contained(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        transport_on_containing(self, sim, ctx, rider);
    }
    /// `RailedTransportContain::onRemoving`.
    fn removing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        transport_on_removing(self, sim, ctx, rider);
        // Emptied: open for docking again.
        if self.state.core.contain_list_size == 0 {
            sim.set_dock_open(ctx.owner_slot, TRUE);
        }
    }
    /// `TransportContain::killRidersWhoAreNotFreeToExit` with the railed
    /// `isSpecificRiderFreeToExit`, which only asks whether the dock is open.
    fn kill_riders_not_free_to_exit(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let destroy = self.data.destroy_riders_who_are_not_free_to_exit;
        #[cfg(not(feature = "retail-crc"))]
        let death = if self.data.open.is_burned_death_to_units != FALSE {
            DeathType::Burned
        } else {
            DeathType::Normal
        };
        #[cfg(feature = "retail-crc")]
        let death = DeathType::Normal;

        for id in self.state.core.contain_list.clone() {
            let Some(rider) = sim.find_object(id) else {
                continue;
            };
            if sim.dock_is_open(ctx.owner_slot) != FALSE {
                continue;
            }
            if destroy != FALSE {
                sim.destroy_object(rider);
                remove_from_contain(self, sim, ctx, rider, FALSE);
            } else {
                kill_object(sim, rider, DamageType::Unresistable, death);
            }
        }
    }
    fn capture(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        old_team: TeamId,
        new_team: TeamId,
    ) {
        transport_on_capture(self, sim, ctx, old_team, new_team);
    }
    fn exit_busy(&self, sim: &dyn SimView, ctx: ModuleContext) -> Bool {
        transport_exit_busy(self, sim, ctx)
    }
    /// `TransportContain::reserveDoorForExit` with the railed `isSpecificRiderFreeToExit`.
    fn reserve_door(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _specific_obj: Option<ObjectSlot>,
    ) -> ExitDoorType {
        if sim.dock_is_open(ctx.owner_slot) != FALSE {
            ExitDoorType::Door1
        } else {
            ExitDoorType::DoorNoneAvailable
        }
    }
    /// `RailedTransportContain::exitObjectViaDoor`. Inside the class the unqualified
    /// `getRailedTransportDockUpdateInterface()` names the inherited `BehaviorModule`
    /// virtual, which returns null here, so the function always returns early.
    fn exit_via_door(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _obj: ObjectSlot,
        _door: ExitDoorType,
    ) {
    }
}

impl_contain_module!(RailedTransportContain);

pub fn create_railed_transport_contain(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<TransportContainModuleData>()
        .expect("RailedTransportContain data type mismatch")
        .clone();
    Box::new(RailedTransportContain::new(typed))
}
