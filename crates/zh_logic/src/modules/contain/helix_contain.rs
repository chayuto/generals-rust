//! `HelixContain` (`PKT-275`): a transport that also carries one portable structure which is
//! never entered into the contain list.
//!
//! Packet: `packets/phase5/PKT-275_HelixContain.md`. Golden suite:
//! `crates/zh_logic/tests/golden_wp53_c2.rs`.
//!
//! `impl_contain_module!` has no hook for `addToContain`, `addToContainList`,
//! `removeFromContain`, `isEnclosingContainerFor` or `isSpecialOverlordStyleContainer`, so
//! [`HelixContain`] implements the containment and exit facets itself and forwards everything
//! the C++ does not override to the macro-generated [`HelixBase`].

use crate::arena::{
    DeferredSelfOp, FacetMask, ModuleContext, ModuleSidecar, ObjectSlot, ThingTemplateId,
};
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::{BodyDamageType, WeaponBonusConditionType};
use crate::modules::behavior::behavior_module::xfer_module_version;
use crate::modules::contain::open_contain::{
    add_to_contain, exit_object_via_door, impl_contain_module, kill_object, on_collide, on_die,
    remove_from_contain, ContainBehavior, OpenContainCore, OpenContainModuleData,
};
use crate::modules::contain::transport_contain::{
    transport_exit_busy, transport_is_valid_container_for, transport_kill_riders_not_free_to_exit,
    transport_on_containing, transport_on_removing, transport_passenger_allowed_to_fire,
    transport_reserve_door, transport_update, transport_xfer, TransportBehavior,
    TransportContainModuleData, TransportContainState,
};
use crate::modules::module::{
    BehaviorModule, CollideFacet, CommandSourceType, ContainFacet, DamageFacet, DieFacet,
    EvacDisposition, ExitDoorType, ExitFacet, ModuleData, ObjectEnterExitType, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, TeamId, UnsignedInt, UnsignedShort, UpdateSleepTime,
    FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `HelixContainModuleData`.
#[derive(Debug, Clone)]
pub struct HelixContainModuleData {
    pub transport: TransportContainModuleData,
    pub payload_template_names: Vec<String>,
    pub draw_pips: Bool,
}

impl HelixContainModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            transport: TransportContainModuleData::new(tag_key, module_name_key),
            payload_template_names: Vec::new(),
            draw_pips: TRUE,
        }
    }

    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        if field.eq_ignore_ascii_case("PayloadTemplateName") {
            while let Some(tok) = reader.next_token_or_null(SEPS_DEFAULT) {
                self.payload_template_names.push(tok.to_owned());
            }
            Some(Ok(()))
        } else if field.eq_ignore_ascii_case("ShouldDrawPips") {
            Some(reader.parse_bool(&mut self.draw_pips))
        } else {
            self.transport.parse_field(field, reader, parse_ctx)
        }
    }
}

impl ModuleData for HelixContainModuleData {
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

/// `HelixContain`'s state plus every override that [`ContainBehavior`] can express.
pub struct HelixBase {
    pub data: HelixContainModuleData,
    pub state: TransportContainState,
    pub portable_structure_id: ObjectId,
    /// Slot of `portable_structure_id`; `isEnclosingContainerFor` receives no view to look it up.
    portable_slot: Option<ObjectSlot>,
    /// Set while `OpenContain::onCollide` runs for a portable, so that the base code does not
    /// enter it into the contain list before the override admits it.
    hold_portable: Bool,
}

impl HelixBase {
    fn is_portable_kind(sim: &dyn SimView, obj: ObjectSlot) -> bool {
        sim.object_is_kind_of(obj, KindOfType::PortableStructure) != FALSE
    }

    /// The portable-structure branch shared by `addToContain` and `addToContainList`.
    fn try_admit_portable(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        obj: ObjectSlot,
    ) -> bool {
        if !Self::is_portable_kind(sim, obj) || self.portable_structure_id != ObjectId::INVALID {
            return false;
        }
        let Some(id) = sim.object(obj).map(|o| o.id) else {
            return false;
        };
        self.portable_structure_id = id;
        self.portable_slot = Some(obj);
        // Fool the portable into thinking our object is its container.
        sim.set_contained_by(obj, Some(ctx.owner_slot));
        true
    }

    /// The portable-structure branch of `removeFromContain`.
    fn try_release_portable(&mut self, sim: &mut dyn SimApi, obj: ObjectSlot) -> bool {
        if !Self::is_portable_kind(sim, obj)
            || sim.object(obj).map(|o| o.id) != Some(self.portable_structure_id)
        {
            return false;
        }
        #[cfg(not(feature = "retail-crc"))]
        sim.set_contained_by(obj, None);
        self.portable_structure_id = ObjectId::INVALID;
        self.portable_slot = None;
        true
    }

    /// `HelixContain::addToContain`.
    fn admit(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, obj: ObjectSlot) {
        if !self.try_admit_portable(sim, ctx, obj) {
            add_to_contain(self, sim, ctx, obj);
        }
    }

    /// `HelixContain::removeFromContain`.
    fn release(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        obj: ObjectSlot,
        expose_stealthed: Bool,
    ) {
        if !self.try_release_portable(sim, obj) {
            remove_from_contain(self, sim, ctx, obj, expose_stealthed);
        }
    }

    fn portable(&self, sim: &dyn SimView) -> Option<ObjectSlot> {
        sim.find_object(self.portable_structure_id)
    }

    /// `HelixContain::createPayload`.
    fn create_payload(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.state.core.load_sounds_enabled = FALSE;
        for name in self.data.payload_template_names.clone() {
            let template = sim.rules().thing_factory.find_template_id(&name);
            let team = sim.object(ctx.owner_slot).map(|o| o.team_id);
            if let (Some(template), Some(team)) = (template, team) {
                let payload = sim.spawn_object(template, team, Coord3D::default(), 0.0);
                if self.is_valid_container(sim, ctx, payload, TRUE) != FALSE {
                    self.admit(sim, ctx, payload);
                }
            }
        }
        self.state.core.load_sounds_enabled = TRUE;
        self.state.payload_created = TRUE;
    }
}

impl TransportBehavior for HelixBase {
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

impl ContainBehavior for HelixBase {
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
    fn is_immune_to_clear(&self) -> Bool {
        TRUE
    }

    fn object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.state.core.schedule.bind(ctx);
        let sidecar = self.state.core.sidecar(self.contain_max_value(), TRUE, FALSE);
        sim.set_module_sidecar(ctx.mod_slot, sidecar);
        self.create_payload(sim, ctx);
    }

    fn delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if let Some(portable) = self.portable(sim) {
            sim.destroy_object(portable);
            // `Object::onDestroy` removes it from its container immediately.
            self.release(sim, ctx, portable, FALSE);
        }
        for id in self.state.core.contain_list.clone() {
            if let Some(rider) = sim.find_object(id) {
                sim.destroy_object(rider);
                remove_from_contain(self, sim, ctx, rider, FALSE);
            }
        }
    }

    /// `HelixContain::onCapture` does not extend the transport.
    fn capture(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        old_team: TeamId,
        new_team: TeamId,
    ) {
        if sim.player_index(old_team) == sim.player_index(new_team) {
            return;
        }
        if let Some(portable) = self.portable(sim) {
            let team = sim.player_default_team(new_team);
            sim.set_object_team(portable, team);
        }
    }

    fn xfer_module(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let version = xfer_module_version(xfer, 2)?;
        if version >= 2 {
            xfer.xfer_object_id(&mut self.portable_structure_id)?;
        }
        transport_xfer(&mut self.state, xfer, sim)
    }

    fn post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let data = self.data.transport.open.clone();
        let enclosing = self.is_enclosing();
        self.state
            .core
            .load_post_process(&data, enclosing, sim, ctx);
        self.portable_slot = sim.find_object(self.portable_structure_id);
    }

    fn update_module(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if let Some(portable) = self.portable(sim) {
            if let Some((pos, angle)) = sim
                .object(ctx.owner_slot)
                .map(|o| (*o.thing.get_position(), o.thing.get_orientation()))
            {
                sim.set_object_position(portable, &pos);
                sim.set_object_orientation(portable, angle);
            }
        }
        // `TransportContain::update` calls the virtual `createPayload`.
        if self.state.payload_created == FALSE {
            #[cfg(feature = "retail-crc")]
            self.create_payload(sim, ctx);
            #[cfg(not(feature = "retail-crc"))]
            if sim
                .object(ctx.owner_slot)
                .is_some_and(|o| !o.is_destroyed())
            {
                self.create_payload(sim, ctx);
            }
        }
        transport_update(self, sim, ctx)
    }

    fn is_valid_container(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        rider: ObjectSlot,
        check_capacity: Bool,
    ) -> Bool {
        if Self::is_portable_kind(sim, rider) {
            if self.hold_portable != FALSE {
                return FALSE;
            }
            if self.portable_structure_id == ObjectId::INVALID {
                return TRUE;
            }
        }
        transport_is_valid_container_for(self, sim, ctx, rider, check_capacity)
    }

    fn passenger_allowed_to_fire(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        id: ObjectId,
    ) -> Bool {
        // Nested containment voids firing.
        if sim
            .object(ctx.owner_slot)
            .is_some_and(|o| o.contained_by.is_some())
        {
            return FALSE;
        }
        if self.portable_structure_id != ObjectId::INVALID && self.portable_structure_id == id {
            return TRUE;
        }
        if let Some(rider) = sim.find_object(id) {
            if sim.object_is_kind_of(rider, KindOfType::Infantry) != FALSE {
                return transport_passenger_allowed_to_fire(&self.state.core, sim, ctx, id);
            }
        }
        FALSE
    }

    fn contained(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        transport_on_containing(self, sim, ctx, rider);
        sim.set_weapon_bonus_condition(rider, WeaponBonusConditionType::Garrisoned);
        sim.set_disabled(rider, DisabledType::Held);
        if Self::is_portable_kind(sim, rider)
            && sim
                .object(ctx.owner_slot)
                .is_some_and(|o| o.status.test(ObjectStatusType::Stealthed.as_i32()) != FALSE)
        {
            sim.receive_stealth_grant(rider, TRUE, 0);
        }
    }

    fn removing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        transport_on_removing(self, sim, ctx, rider);
        sim.clear_weapon_bonus_condition(rider, WeaponBonusConditionType::Garrisoned);
        sim.clear_disabled(rider, DisabledType::Held);
    }

    fn kill_riders_not_free_to_exit(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        transport_kill_riders_not_free_to_exit(self, sim, ctx);
    }

    /// `HelixContain::redeployOccupants`: every listed rider sits just above the hull.
    fn redeploy(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let Some(mut fire_pos) = sim.object(ctx.owner_slot).map(|o| *o.thing.get_position()) else {
            return;
        };
        fire_pos.z += 8.0;
        for id in self.state.core.contain_list.clone() {
            if let Some(rider) = sim.find_object(id) {
                sim.set_object_position(rider, &fire_pos);
            }
        }
    }

    fn collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
        _loc: Option<&Coord3D>,
        _normal: Option<&Coord3D>,
    ) {
        // `OpenContain::onCollide` ends in the virtual `addToContain`.
        let portable = other.filter(|&o| {
            Self::is_portable_kind(sim, o)
                && self.portable_structure_id == ObjectId::INVALID
                && sim.object(o).is_some_and(|r| r.ai_slot.is_some())
                && sim.object_ai_enter_target(o) == Some(ctx.owner_slot)
        });
        self.hold_portable = Bool::from(portable.is_some());
        on_collide(self, sim, ctx, other);
        self.hold_portable = FALSE;
        if let Some(portable) = portable {
            self.try_admit_portable(sim, ctx, portable);
        }
    }

    fn die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        if let Some(portable) = self.portable(sim) {
            kill_object(sim, portable, DamageType::Unresistable, DeathType::Normal);
        }
        on_die(self, sim, ctx, dmg);
    }

    fn body_damage_state_changed(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _old_state: BodyDamageType,
        new_state: BodyDamageType,
    ) {
        if new_state == BodyDamageType::Rubble {
            return;
        }
        if let Some(portable) = self.portable(sim) {
            sim.set_body_damage_state(portable, new_state);
        }
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
    fn exit_via_door(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        obj: ObjectSlot,
        _door: ExitDoorType,
    ) {
        // `exitObjectViaDoor` starts with the virtual `removeFromContain`.
        self.try_release_portable(sim, obj);
        exit_object_via_door(self, sim, ctx, obj);
    }
}

impl_contain_module!(HelixBase);

/// Runtime instance of `HelixContain` (`PKT-275`).
pub struct HelixContain(pub HelixBase);

impl HelixContain {
    pub fn new(data: HelixContainModuleData) -> Self {
        let state = TransportContainState::new(data.transport.open.passengers_allowed_to_fire);
        Self(HelixBase {
            data,
            state,
            portable_structure_id: ObjectId::INVALID,
            portable_slot: None,
            hold_portable: FALSE,
        })
    }
}

impl BehaviorModule for HelixContain {
    fn module_name_key(&self) -> NameKeyType {
        self.0.module_name_key()
    }
    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.0.on_construct(sim, ctx);
    }
    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.0.on_object_created(sim, ctx);
    }
    fn on_delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.0.on_delete(sim, ctx);
    }
    fn delete_child_objects(&self) -> Vec<ObjectId> {
        let mut ids = Vec::with_capacity(1 + self.0.state.core.contain_list.len());
        if self.0.portable_structure_id != ObjectId::INVALID {
            ids.push(self.0.portable_structure_id);
        }
        ids.extend_from_slice(&self.0.state.core.contain_list);
        ids
    }
    fn on_capture(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        old_team: TeamId,
        new_team: TeamId,
    ) {
        self.0.on_capture(sim, ctx, old_team, new_team);
    }
    fn apply_deferred_op(&mut self, op: DeferredSelfOp, sim: &mut dyn SimApi, ctx: ModuleContext) {
        match op {
            DeferredSelfOp::ContainAdd(rider) => self.0.admit(sim, ctx, rider),
            DeferredSelfOp::ContainRemove {
                obj,
                expose_stealthed,
            } => self.0.release(sim, ctx, obj, expose_stealthed),
            other => self.0.apply_deferred_op(other, sim, ctx),
        }
    }
    fn sync_sidecar(&self) -> ModuleSidecar {
        BehaviorModule::sync_sidecar(&self.0)
    }
    fn crc(&self, xfer: &mut dyn Xfer) {
        BehaviorModule::crc(&self.0, xfer);
    }
    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        BehaviorModule::xfer(&mut self.0, xfer, sim)
    }
    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        BehaviorModule::load_post_process(&mut self.0, sim, ctx);
    }
    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(&self.0)
    }
    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(&mut self.0)
    }
    fn as_contain(&self) -> Option<&dyn ContainFacet> {
        Some(self)
    }
    fn as_contain_mut(&mut self) -> Option<&mut dyn ContainFacet> {
        Some(self)
    }
    fn as_collide(&self) -> Option<&dyn CollideFacet> {
        Some(&self.0)
    }
    fn as_collide_mut(&mut self) -> Option<&mut dyn CollideFacet> {
        Some(&mut self.0)
    }
    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(&self.0)
    }
    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(&mut self.0)
    }
    fn as_damage_mut(&mut self) -> Option<&mut dyn DamageFacet> {
        Some(&mut self.0)
    }
    fn as_exit(&self) -> Option<&dyn ExitFacet> {
        Some(self)
    }
    fn as_exit_mut(&mut self) -> Option<&mut dyn ExitFacet> {
        Some(self)
    }
    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
}

impl ExitFacet for HelixContain {
    fn is_exit_busy(&self, sim: &dyn SimView, ctx: ModuleContext) -> Bool {
        self.0.is_exit_busy(sim, ctx)
    }
    fn reserve_door_for_exit(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        tmpl_id: ThingTemplateId,
        specific_obj: Option<ObjectSlot>,
    ) -> ExitDoorType {
        self.0
            .reserve_door_for_exit(sim, ctx, tmpl_id, specific_obj)
    }
    fn exit_object_via_door(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_obj: ObjectSlot,
        exit_door: ExitDoorType,
    ) {
        ExitFacet::exit_object_via_door(&mut self.0, sim, ctx, new_obj, exit_door);
    }
    fn exit_object_in_a_hurry(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_obj: ObjectSlot,
    ) {
        self.0.try_release_portable(sim, new_obj);
        self.0.exit_object_in_a_hurry(sim, ctx, new_obj);
    }
    fn set_rally_point(&mut self, pos: &Coord3D) {
        self.0.set_rally_point(pos);
    }
    fn get_rally_point(&self) -> Option<&Coord3D> {
        self.0.get_rally_point()
    }
    fn get_natural_rally_point(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        rally_point: &mut Coord3D,
        offset: Bool,
    ) -> Bool {
        self.0
            .get_natural_rally_point(sim, ctx, rally_point, offset)
    }
}

impl ContainFacet for HelixContain {
    fn contain_list_for_mass(&self) -> &[ObjectId] {
        self.0.contain_list_for_mass()
    }
    fn is_weapon_bonus_passed_to_passengers(&self) -> Bool {
        self.0.is_weapon_bonus_passed_to_passengers()
    }
    fn add_to_contain(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, obj: ObjectSlot) {
        self.0.admit(sim, ctx, obj);
    }
    fn add_to_contain_list(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, obj: ObjectSlot) {
        if !self.0.try_admit_portable(sim, ctx, obj) {
            self.0.add_to_contain_list(sim, ctx, obj);
        }
    }
    fn remove_from_contain(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        obj: ObjectSlot,
        expose_stealthed: Bool,
    ) {
        self.0.release(sim, ctx, obj, expose_stealthed);
    }
    fn is_enclosing_container_for(&self, other: ObjectSlot) -> Bool {
        if self.0.portable_structure_id != ObjectId::INVALID && self.0.portable_slot == Some(other)
        {
            return FALSE;
        }
        self.0.is_enclosing_container_for(other)
    }
    fn is_special_overlord_style_container(&self) -> Bool {
        TRUE
    }

    fn get_contain_count(&self, sim: &dyn SimView, ctx: ModuleContext) -> UnsignedInt {
        self.0.get_contain_count(sim, ctx)
    }
    fn get_contain_max(&self, sim: &dyn SimView, ctx: ModuleContext) -> Int {
        self.0.get_contain_max(sim, ctx)
    }
    fn is_garrisonable(&self, sim: &dyn SimView, ctx: ModuleContext) -> Bool {
        self.0.is_garrisonable(sim, ctx)
    }
    fn is_bustable(&self) -> Bool {
        self.0.is_bustable()
    }
    fn is_heal_contain(&self) -> Bool {
        self.0.is_heal_contain()
    }
    fn is_immune_to_clear_building_attacks(&self) -> Bool {
        self.0.is_immune_to_clear_building_attacks()
    }
    fn is_special_zero_slot_container(&self) -> Bool {
        self.0.is_special_zero_slot_container()
    }
    fn is_valid_container_for(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        obj: ObjectSlot,
        check_capacity: Bool,
    ) -> Bool {
        ContainFacet::is_valid_container_for(&self.0, sim, ctx, obj, check_capacity)
    }
    fn get_contained_items<'a>(
        &'a self,
        sim: &'a dyn SimView,
        ctx: ModuleContext,
    ) -> &'a [ObjectId] {
        self.0.get_contained_items(sim, ctx)
    }
    fn remove_all_contained(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        expose_stealthed: Bool,
    ) {
        self.0.remove_all_contained(sim, ctx, expose_stealthed);
    }
    fn kill_all_contained(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.0.kill_all_contained(sim, ctx);
    }
    fn harm_and_force_exit_all_contained(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) {
        self.0.harm_and_force_exit_all_contained(sim, ctx, dmg);
    }
    fn is_passenger_allowed_to_fire(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        id: ObjectId,
    ) -> Bool {
        self.0.is_passenger_allowed_to_fire(sim, ctx, id)
    }
    fn set_passenger_allowed_to_fire(&mut self, permission: Bool) {
        self.0.set_passenger_allowed_to_fire(permission);
    }
    fn get_stealth_units_contained(&self) -> UnsignedInt {
        self.0.get_stealth_units_contained()
    }
    fn get_hero_units_contained(&self, sim: &dyn SimView, ctx: ModuleContext) -> UnsignedInt {
        self.0.get_hero_units_contained(sim, ctx)
    }
    fn is_contained(&self, sim: &dyn SimView, ctx: ModuleContext, obj_id: ObjectId) -> Bool {
        self.0.is_contained(sim, ctx, obj_id)
    }
    fn on_selling(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.0.on_selling(sim, ctx);
    }
    fn process_damage_to_contained(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        percent_damage: Real,
    ) {
        self.0.process_damage_to_contained(sim, ctx, percent_damage);
    }
    fn get_closest_rider(&self, sim: &dyn SimView, pos: &Coord3D) -> Option<ObjectSlot> {
        self.0.get_closest_rider(sim, pos)
    }
    fn order_all_passengers_to_exit(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        cmd_source: CommandSourceType,
        instantly: Bool,
    ) {
        self.0
            .order_all_passengers_to_exit(sim, ctx, cmd_source, instantly);
    }
    fn order_all_passengers_to_idle(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.0.order_all_passengers_to_idle(sim, ctx);
    }
    fn order_all_passengers_to_hack_internet(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.0.order_all_passengers_to_hack_internet(sim, ctx);
    }
    fn mark_all_passengers_detected(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.0.mark_all_passengers_detected(sim, ctx);
    }
    fn get_player_entered_mask(&self) -> UnsignedShort {
        self.0.get_player_entered_mask()
    }
    fn set_player_entered_mask(&mut self, mask: UnsignedShort) {
        self.0.set_player_entered_mask(mask);
    }
    fn on_object_wants_to_enter_or_exit(&mut self, obj_id: ObjectId, wants: ObjectEnterExitType) {
        self.0.on_object_wants_to_enter_or_exit(obj_id, wants);
    }
    fn has_objects_wanting_to_enter_or_exit(&self) -> Bool {
        self.0.has_objects_wanting_to_enter_or_exit()
    }
    fn on_containing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, obj: ObjectSlot) {
        self.0.on_containing(sim, ctx, obj);
    }
    fn on_removing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, obj: ObjectSlot) {
        self.0.on_removing(sim, ctx, obj);
    }
    fn set_evac_disposition(&mut self, disp: EvacDisposition) {
        self.0.set_evac_disposition(disp);
    }
    fn set_override_destination(&mut self, pos: &Coord3D) {
        self.0.set_override_destination(pos);
    }
    fn contain_react_to_transform_change(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.0.contain_react_to_transform_change(sim, ctx);
    }
}

pub fn parse_helix_contain_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = HelixContainModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        data.parse_field(field, r, parse_ctx)
            .unwrap_or(Err(IniError::UnknownToken))
    })?;
    Ok(Box::new(data))
}

pub fn create_helix_contain(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<HelixContainModuleData>()
        .expect("HelixContain data type mismatch")
        .clone();
    Box::new(HelixContain::new(typed))
}
