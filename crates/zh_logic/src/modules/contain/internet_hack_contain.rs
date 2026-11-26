//! `PKT-277`: `InternetHackContain` (`InternetHackContain.h`, `.cpp`): a `TransportContain`
//! that orders its passengers to hack the internet.
//!
//! Packet: `packets/phase5/PKT-277_InternetHackContain.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_b.rs`.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::xfer_module_version;
use crate::modules::contain::open_contain::{
    impl_contain_module, ContainBehavior, OpenContainCore, OpenContainModuleData,
};
use crate::modules::contain::transport_contain::{
    transport_exit_busy, transport_is_valid_container_for, transport_kill_riders_not_free_to_exit,
    transport_on_capture, transport_on_containing, transport_on_removing,
    transport_passenger_allowed_to_fire, transport_reserve_door, transport_update, transport_xfer,
    TransportBehavior, TransportContainModuleData, TransportContainState,
};
use crate::modules::module::{BehaviorModule, CommandSourceType, ExitDoorType, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, Int, NameKeyType, ObjectId, TeamId, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

/// `InternetHackContainModuleData` adds no fields to its transport base.
#[derive(Debug, Clone, PartialEq)]
pub struct InternetHackContainModuleData {
    pub transport: TransportContainModuleData,
}

impl ModuleData for InternetHackContainModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.transport.open.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.transport.open.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0)
    }
}

/// Runtime instance of `InternetHackContain` (`PKT-277`).
pub struct InternetHackContain {
    pub data: InternetHackContainModuleData,
    pub state: TransportContainState,
}

impl InternetHackContain {
    pub fn new(data: InternetHackContainModuleData) -> Self {
        let state = TransportContainState::new(data.transport.open.passengers_allowed_to_fire);
        Self { data, state }
    }
}

impl TransportBehavior for InternetHackContain {
    fn transport_data(&self) -> &TransportContainModuleData {
        &self.data.transport
    }
    fn transport(&self) -> &TransportContainState {
        &self.state
    }
    fn transport_mut(&mut self) -> &mut TransportContainState {
        &mut self.state
    }
}

impl ContainBehavior for InternetHackContain {
    fn open_data(&self) -> &OpenContainModuleData {
        &self.data.transport.open
    }
    fn open_core(&self) -> &OpenContainCore {
        &self.state.core
    }
    fn open_core_mut(&mut self) -> &mut OpenContainCore {
        &mut self.state.core
    }
    fn contain_max_value(&self) -> Int {
        self.data.transport.slot_capacity
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
        if sim.has_ai(rider) != FALSE {
            sim.ai_hack_internet(rider, CommandSourceType::FromAi);
        }
    }
    fn removing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        transport_on_removing(self, sim, ctx, rider);
    }
    fn kill_riders_not_free_to_exit(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        transport_kill_riders_not_free_to_exit(self, sim, ctx);
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
    fn reserve_door(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        specific_obj: Option<ObjectSlot>,
    ) -> ExitDoorType {
        transport_reserve_door(sim, ctx, specific_obj)
    }
}

impl_contain_module!(InternetHackContain);

pub fn parse_internet_hack_contain_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut transport = TransportContainModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        transport
            .parse_field(field, r, parse_ctx)
            .unwrap_or(Err(IniError::UnknownToken))
    })?;
    Ok(Box::new(InternetHackContainModuleData { transport }))
}

pub fn create_internet_hack_contain(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<InternetHackContainModuleData>()
        .expect("InternetHackContain data type mismatch")
        .clone();
    Box::new(InternetHackContain::new(typed))
}
