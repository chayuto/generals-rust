//! `PKT-244`: Base `SpecialPowerModule`, `SpecialPowerModuleData`, and `SpecialPowerBehaviorImpl`
//! (`SpecialPowerModule.h` / `SpecialPowerModule.cpp`).

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext, ModuleSidecar, ModuleSlot, ObjectSlot};
use crate::modules::module::{BehaviorModule, ModuleData, ModuleSnapshotHeader, SpecialPowerFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::rules::science::{ScienceType, SCIENCE_INVALID};
use crate::rules::special_power::SpecialPowerTemplate;
use crate::rules::special_power_types::SpecialPowerType;
use crate::modules::module::SpecialPowerCaller;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, NameKeyType, Real, UnsignedInt, XferVersion, FALSE, Int, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// Shared INI configuration parsed for `SpecialPowerModuleData`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpecialPowerModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub special_power_name: String,
    pub special_power_id: UnsignedInt,
    pub initiate_sound: String,
    pub update_module_starts_attack: Bool,
    pub starts_paused: Bool,
    pub starts_ready: Bool,
    pub scripted_special_power_only: Bool,
}

impl SpecialPowerModuleData {
    #[inline]
    pub const fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            special_power_name: String::new(),
            special_power_id: 0,
            initiate_sound: String::new(),
            update_module_starts_attack: FALSE,
            starts_paused: FALSE,
            starts_ready: FALSE,
            scripted_special_power_only: FALSE,
        }
    }

    /// Parses one `SpecialPowerModuleData` INI field, returning `None` if `field` belongs to a subclass.
    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        if field.eq_ignore_ascii_case("SpecialPowerTemplate") {
            let res = (|| {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                if tok.eq_ignore_ascii_case("None") {
                    self.special_power_name.clear();
                    self.special_power_id = 0;
                } else {
                    self.special_power_name = tok.to_owned();
                    self.special_power_id = parse_ctx
                        .special_power_store
                        .find_special_power_template(tok)
                        .map_or(0, |t| t.id());
                }
                Ok(())
            })();
            Some(res)
        } else if field.eq_ignore_ascii_case("UpdateModuleStartsAttack") {
            Some(reader.parse_bool(&mut self.update_module_starts_attack))
        } else if field.eq_ignore_ascii_case("StartsPaused") {
            Some(reader.parse_bool(&mut self.starts_paused))
        } else if field.eq_ignore_ascii_case("StartsReady") {
            Some(reader.parse_bool(&mut self.starts_ready))
        } else if field.eq_ignore_ascii_case("InitiateSound") {
            let res = (|| {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                if tok.eq_ignore_ascii_case("NoSound") {
                    self.initiate_sound.clear();
                } else {
                    self.initiate_sound = tok.to_owned();
                }
                Ok(())
            })();
            Some(res)
        } else if field.eq_ignore_ascii_case("ScriptedSpecialPowerOnly") {
            Some(reader.parse_bool(&mut self.scripted_special_power_only))
        } else {
            None
        }
    }
}

impl ModuleData for SpecialPowerModuleData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask::SPECIAL_POWER
    }
}

/// Parses `SpecialPowerModuleData` base fields and delegates remaining fields to `parse_extra`.
pub fn parse_special_power_data_with(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
    mut parse_extra: impl FnMut(&str, &mut IniReader<'_>, &mut ModuleParseContext<'_>) -> Result<(), IniError>,
) -> Result<SpecialPowerModuleData, IniError> {
    let mut base = SpecialPowerModuleData::new(tag_key, module_name_key);
    ini.read_block_fields(|field, reader| {
        if let Some(res) = base.parse_field(field, reader, parse_ctx) {
            res
        } else {
            parse_extra(field, reader, parse_ctx)
        }
    })?;
    Ok(base)
}

/// Standard parser for `SpecialPowerModule` subclasses with no subclass INI fields.
pub fn parse_special_power_module_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let data = parse_special_power_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |_field, _reader, _ctx| Err(IniError::UnknownToken),
    )?;
    Ok(Box::new(data))
}

/// Reads the container occupancy count from `owner`'s `Contain` module sidecar (`getContainCount`).
pub(crate) fn owner_contain_count(sim: &dyn SimView, owner: ObjectSlot) -> Option<UnsignedInt> {
    let cs = sim.object(owner).and_then(|o| o.contain_slot)?;
    match sim.module_sidecar(cs) {
        ModuleSidecar::Contain { count, .. } => Some(count),
        _ => Some(0),
    }
}

/// Base runtime state matching C++ `SpecialPowerModule` (`SpecialPowerModule.h` / `SpecialPowerModule.cpp`).
#[derive(Debug, Clone)]
pub struct SpecialPowerModule {
    pub module_name_key: NameKeyType,
    pub data: SpecialPowerModuleData,
    pub available_on_frame: UnsignedInt,
    pub paused_count: Int,
    pub paused_on_frame: UnsignedInt,
    pub paused_percent: Real,
}

impl SpecialPowerModule {
    #[inline]
    pub fn new(data: SpecialPowerModuleData) -> Self {
        Self {
            module_name_key: data.module_name_key,
            data,
            available_on_frame: 0,
            paused_count: 0,
            paused_on_frame: 0,
            paused_percent: 0.0,
        }
    }

    #[inline]
    pub const fn with_name_key(module_name_key: NameKeyType) -> Self {
        Self {
            module_name_key,
            data: SpecialPowerModuleData::new(NameKeyType::INVALID, module_name_key),
            available_on_frame: 0,
            paused_count: 0,
            paused_on_frame: 0,
            paused_percent: 0.0,
        }
    }

    pub fn resolve_template<'a>(&self, sim: &'a dyn SimView) -> Option<&'a SpecialPowerTemplate> {
        if self.data.special_power_id != 0 {
            sim.rules()
                .special_power_store
                .find_special_power_template_by_id(self.data.special_power_id)
        } else if !self.data.special_power_name.is_empty() {
            sim.rules()
                .special_power_store
                .find_special_power_template(&self.data.special_power_name)
        } else {
            None
        }
    }

    pub fn resolve_template_id(&self, sim: &dyn SimView) -> UnsignedInt {
        self.resolve_template(sim)
            .map_or(self.data.special_power_id, |t| t.id())
    }

    #[inline]
    pub fn is_owner_disabled(sim: &dyn SimView, ctx: ModuleContext) -> bool {
        sim.object(ctx.owner_slot)
            .is_some_and(|o| o.is_disabled() != FALSE)
    }

    pub fn sync_sidecar(&self) -> ModuleSidecar {
        ModuleSidecar::SpecialPower {
            template_id: self.data.special_power_id,
            available_on_frame: self.available_on_frame,
            paused_count: self.paused_count,
            paused_on_frame: self.paused_on_frame,
            paused_percent: self.paused_percent,
            starts_ready: self.data.starts_ready,
        }
    }

    pub fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let under_const = sim.object(ctx.owner_slot).is_some_and(|o| {
            o.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE
        });
        let (tid, shared_n_sync, has_public_timer) = self
            .resolve_template(sim)
            .map_or((self.data.special_power_id, false, false), |t| {
                (
                    t.id(),
                    t.is_shared_n_sync() != FALSE,
                    t.has_public_timer() != FALSE,
                )
            });

        if !under_const && !shared_n_sync {
            self.start_power_recharge(sim, ctx);
            if self.starts_ready(sim, ctx) != FALSE {
                self.available_on_frame = sim.frame();
            }
        }

        if self.data.starts_paused != FALSE {
            self.pause_countdown(sim, ctx, TRUE);
        }

        if self.paused_count == 0
            && shared_n_sync
            && has_public_timer
            && sim.object_is_kind_of(ctx.owner_slot, KindOfType::Structure) != FALSE
        {
            let ready = self.get_ready_frame(sim, ctx);
            sim.update_public_timer(ctx.owner_slot, tid, ready, TRUE);
        }
    }

    pub fn on_delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if let Some((tid, true)) = self
            .resolve_template(sim)
            .map(|t| (t.id(), t.has_public_timer() != FALSE))
        {
            sim.update_public_timer(ctx.owner_slot, tid, 0, FALSE);
        }
    }

    pub fn on_disabled_edge(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, disabled: Bool) {
        self.pause_countdown(sim, ctx, disabled);
    }

    pub fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if let Some((tid, true, true)) = self.resolve_template(sim).map(|t| {
            (
                t.id(),
                t.is_shared_n_sync() != FALSE,
                t.has_public_timer() != FALSE,
            )
        }) {
            if self.paused_count == 0
                && sim.object_is_kind_of(ctx.owner_slot, KindOfType::Structure) != FALSE
            {
                let ready = self.get_ready_frame(sim, ctx);
                sim.update_public_timer(ctx.owner_slot, tid, ready, TRUE);
            }
        }
    }

    pub fn apply_deferred_op(&mut self, op: DeferredSelfOp, sim: &mut dyn SimApi, ctx: ModuleContext) {
        match op {
            DeferredSelfOp::SpecialPowerPause { pause } => self.pause_countdown(sim, ctx, pause),
            DeferredSelfOp::SpecialPowerSetReadyFrame(frame) => {
                self.set_ready_frame(sim, ctx, frame);
            }
            _ => {}
        }
    }

    #[inline]
    pub fn crc(&self, _xfer: &mut dyn Xfer) {}

    /// Serializes `SpecialPowerModule::xfer`.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        ModuleSnapshotHeader::xfer_special_power_module_header(
            xfer,
            &mut self.available_on_frame,
            &mut self.paused_count,
            &mut self.paused_on_frame,
            &mut self.paused_percent,
        )
    }

    /// Serializes a concrete subclass with no extra persisted fields (`version(1)` + `SpecialPowerModule::xfer`).
    pub fn xfer_subclass(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.xfer(xfer)
    }

    pub fn initiate_intent_to_do_special_power(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: Option<ObjectSlot>,
        target_pos: Option<&Coord3D>,
        command_options: UnsignedInt,
    ) -> bool {
        let updates: Vec<ModuleSlot> = sim.object(ctx.owner_slot).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|&&(_, mask)| mask.contains(FacetMask::SPECIAL_POWER_UPDATE))
                .map(|&(ms, _)| ms)
                .collect()
        });
        let template_id = self.resolve_template_id(&*sim);
        let mut taken = false;
        let mut unported = false;
        for update in updates {
            let caller = SpecialPowerCaller { ctx, module: self };
            match sim.initiate_special_power_update(
                update,
                caller,
                template_id,
                target,
                target_pos,
                command_options,
            ) {
                Some(valid) if valid != FALSE => {
                    taken = true;
                    break;
                }
                Some(_) => {}
                None => unported = true,
            }
        }
        // Update modules without the facet are stood in for by a recorded order.
        if !taken && unported {
            sim.trigger_special_ability(ctx.owner_slot, target, target_pos);
            taken = true;
        }

        if self.available_on_frame == u32::MAX {
            return false;
        }

        taken
    }

    pub fn about_to_do_special_power(
        &self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _location: Option<&Coord3D>,
    ) {
        let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) else {
            return;
        };
        let power_name = self.get_power_name(sim, ctx);
        sim.notify_special_power_triggered(team_id, &power_name, ctx.owner_id);
    }

    pub fn create_view_object(&self, sim: &mut dyn SimApi, ctx: ModuleContext, location: &Coord3D) {
        let Some((vision_range, vision_duration)) = self
            .resolve_template(sim)
            .map(|t| (t.view_object_range(), t.view_object_duration()))
        else {
            return;
        };
        if vision_range == 0.0 || vision_duration == 0 {
            return;
        }
        let view_name = sim.rules().global_data.special_power_view_object_name.clone();
        if view_name.is_empty() {
            return;
        }
        let Some(tmpl_id) = sim.rules().thing_factory.find_template_id(&view_name) else {
            return;
        };
        let Some(owner_team) = sim.object(ctx.owner_slot).map(|o| o.team_id) else {
            return;
        };
        let default_team = sim.player_default_team(owner_team);
        let view_obj = sim.spawn_object(tmpl_id, default_team, *location, 0.0);
        sim.set_shroud_clearing_range(view_obj, vision_range);
        sim.set_deletion_lifetime(view_obj, vision_duration, vision_duration);
    }

    pub fn trigger_special_power(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        location: Option<&Coord3D>,
    ) {
        self.about_to_do_special_power(sim, ctx, location);
        if let Some(loc) = location {
            self.create_view_object(sim, ctx, loc);
        }
        self.start_power_recharge(sim, ctx);
    }
}

impl BehaviorModule for SpecialPowerModule {
    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        SpecialPowerModule::on_object_created(self, sim, ctx);
    }

    fn on_delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        SpecialPowerModule::on_delete(self, sim, ctx);
    }

    fn on_disabled_edge(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, disabled: Bool) {
        SpecialPowerModule::on_disabled_edge(self, sim, ctx, disabled);
    }

    fn apply_deferred_op(&mut self, op: DeferredSelfOp, sim: &mut dyn SimApi, ctx: ModuleContext) {
        SpecialPowerModule::apply_deferred_op(self, op, sim, ctx);
    }

    fn sync_sidecar(&self) -> ModuleSidecar {
        SpecialPowerModule::sync_sidecar(self)
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        SpecialPowerModule::crc(self, xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        SpecialPowerModule::xfer(self, xfer)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        SpecialPowerModule::load_post_process(self, sim, ctx);
    }

    fn as_special_power(&self) -> Option<&dyn SpecialPowerFacet> {
        Some(self)
    }

    fn as_special_power_mut(&mut self) -> Option<&mut dyn SpecialPowerFacet> {
        Some(self)
    }
}

impl SpecialPowerFacet for SpecialPowerModule {
    fn is_module_for_power(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        template_id: UnsignedInt,
    ) -> Bool {
        Bool::from(self.resolve_template_id(sim) == template_id)
    }

    fn is_ready(&self, sim: &dyn SimView, ctx: ModuleContext) -> Bool {
        if sim.special_power_uses_delay() == FALSE {
            return TRUE;
        }
        if let Some(tmpl) = self.resolve_template(sim) {
            if tmpl.is_shared_n_sync() != FALSE {
                if let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
                    return Bool::from(
                        sim.frame() >= sim.shared_special_power_ready_frame(team_id, tmpl.id()),
                    );
                }
            }
        }
        Bool::from(self.paused_count == 0 && sim.frame() >= self.available_on_frame)
    }

    fn get_percent_ready(&self, sim: &dyn SimView, ctx: ModuleContext) -> Real {
        if self.paused_count > 0 && self.paused_percent == 1.0 {
            return 0.99999;
        }
        if sim.special_power_uses_delay() == FALSE || self.is_ready(sim, ctx) != FALSE {
            return 1.0;
        }
        if self.paused_count > 0 {
            return self.paused_percent;
        }
        let Some(tmpl) = self.resolve_template(sim) else {
            return 0.0;
        };
        let ready_frame = if tmpl.is_shared_n_sync() != FALSE {
            sim.object(ctx.owner_slot).map_or(self.available_on_frame, |o| {
                sim.shared_special_power_ready_frame(o.team_id, tmpl.id())
            })
        } else {
            self.available_on_frame
        };
        1.0 - (ready_frame.wrapping_sub(sim.frame()) as Real) / (tmpl.reload_time() as Real)
    }

    fn get_ready_frame(&self, sim: &dyn SimView, ctx: ModuleContext) -> UnsignedInt {
        if let Some(tmpl) = self.resolve_template(sim) {
            if tmpl.is_shared_n_sync() != FALSE {
                if let Some(obj) = sim.object(ctx.owner_slot) {
                    return sim.shared_special_power_ready_frame(obj.team_id, tmpl.id());
                }
            }
        }
        if self.paused_count > 0 || Self::is_owner_disabled(sim, ctx) {
            let paused_frames = sim.frame().wrapping_sub(self.paused_on_frame);
            self.available_on_frame.wrapping_add(paused_frames)
        } else {
            self.available_on_frame
        }
    }

    fn get_raw_ready_frame(&self) -> UnsignedInt {
        self.available_on_frame
    }

    fn get_power_name(&self, sim: &dyn SimView, _ctx: ModuleContext) -> String {
        self.resolve_template(sim)
            .map_or_else(|| self.data.special_power_name.clone(), |t| t.name().to_owned())
    }

    fn get_special_power_id(&self, sim: &dyn SimView, _ctx: ModuleContext) -> UnsignedInt {
        self.resolve_template_id(sim)
    }

    fn has_special_power(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        sp_type: SpecialPowerType,
    ) -> Bool {
        Bool::from(
            self.resolve_template(sim)
                .is_some_and(|t| t.special_power_type() == sp_type),
        )
    }

    fn get_required_science(&self, sim: &dyn SimView, _ctx: ModuleContext) -> ScienceType {
        self.resolve_template(sim)
            .map_or(SCIENCE_INVALID, |t| t.required_science())
    }

    fn set_ready_frame(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext, frame: UnsignedInt) {
        self.available_on_frame = frame;
        self.paused_on_frame = sim.frame();
    }

    fn is_paused(&self) -> Bool {
        Bool::from(self.paused_count > 0)
    }

    fn pause_countdown(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, pause: Bool) {
        if pause != FALSE {
            if self.paused_count == 0 {
                self.paused_on_frame = sim.frame();
                self.paused_percent = self.get_percent_ready(sim, ctx);
            }
            self.paused_count += 1;
        } else if self.paused_count > 0 {
            self.paused_count -= 1;
            if self.paused_count == 0 {
                let delta = sim.frame().wrapping_sub(self.paused_on_frame);
                self.available_on_frame = self.available_on_frame.wrapping_add(delta);
            }
        }
    }

    fn starts_ready(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Bool {
        #[cfg(feature = "retail-crc")]
        {
            FALSE
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            self.data.starts_ready
        }
    }

    fn is_script_only(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Bool {
        self.data.scripted_special_power_only
    }

    fn on_special_power_creation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.start_power_recharge(sim, ctx);

        let Some((tid, shared_n_sync, has_public_timer)) = self.resolve_template(sim).map(|t| {
            (
                t.id(),
                t.is_shared_n_sync() != FALSE,
                t.has_public_timer() != FALSE,
            )
        }) else {
            return;
        };

        if shared_n_sync {
            if let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
                let now = sim.frame();
                sim.express_shared_special_power_ready_frame(team_id, tid, now);
                self.available_on_frame = sim.shared_special_power_ready_frame(team_id, tid);
            }
        }

        if self.data.starts_paused != FALSE {
            self.pause_countdown(sim, ctx, TRUE);
        }

        if has_public_timer && sim.object_is_kind_of(ctx.owner_slot, KindOfType::Structure) != FALSE
        {
            let ready = self.get_ready_frame(sim, ctx);
            sim.update_public_timer(ctx.owner_slot, tid, ready, TRUE);
        }
    }

    fn start_power_recharge(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if sim.special_power_uses_delay() == FALSE {
            return;
        }
        let Some((tid, reload_time, shared)) = self
            .resolve_template(sim)
            .map(|t| (t.id(), t.reload_time(), t.is_shared_n_sync() != FALSE))
        else {
            return;
        };
        let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) else {
            return;
        };
        if shared {
            let _ = sim.reset_shared_special_power_ready_frame(team_id, tid);
        } else {
            self.available_on_frame = sim.frame().wrapping_add(reload_time);
        }
    }

    fn mark_special_power_triggered(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        loc: Option<&Coord3D>,
    ) {
        self.trigger_special_power(sim, ctx, loc);
    }

    fn do_special_power(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        command_options: UnsignedInt,
    ) {
        if self.paused_count > 0 || Self::is_owner_disabled(sim, ctx) {
            return;
        }
        let _ = self.initiate_intent_to_do_special_power(sim, ctx, None, None, command_options);
        if self.data.update_module_starts_attack == FALSE {
            self.trigger_special_power(sim, ctx, None);
        }
    }

    fn do_special_power_at_object(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: ObjectSlot,
        command_options: UnsignedInt,
    ) {
        if self.paused_count > 0 || Self::is_owner_disabled(sim, ctx) {
            return;
        }
        let target_pos = sim.object(target).map(|o| *o.thing.get_position());
        let _ = self.initiate_intent_to_do_special_power(
            sim,
            ctx,
            Some(target),
            None,
            command_options,
        );
        if self.data.update_module_starts_attack == FALSE {
            self.trigger_special_power(sim, ctx, target_pos.as_ref());
        }
    }

    fn do_special_power_at_location(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        loc: &Coord3D,
        _angle: Real,
        command_options: UnsignedInt,
    ) {
        if self.paused_count > 0 || Self::is_owner_disabled(sim, ctx) {
            return;
        }
        let _ =
            self.initiate_intent_to_do_special_power(sim, ctx, None, Some(loc), command_options);
        if self.available_on_frame == u32::MAX {
            return;
        }
        if self.data.update_module_starts_attack == FALSE {
            self.trigger_special_power(sim, ctx, Some(loc));
        }
    }

    fn do_special_power_using_waypoints(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _pts: &[Coord3D],
        command_options: UnsignedInt,
    ) {
        if self.paused_count > 0 || Self::is_owner_disabled(sim, ctx) {
            return;
        }
        let _ = self.initiate_intent_to_do_special_power(sim, ctx, None, None, command_options);
        if self.data.update_module_starts_attack == FALSE {
            self.trigger_special_power(sim, ctx, None);
        }
    }
}

/// Subclass hook trait implemented by concrete `SpecialPowerModule` subtypes.
pub trait SpecialPowerBehaviorImpl {
    fn base(&self) -> &SpecialPowerModule;
    fn base_mut(&mut self) -> &mut SpecialPowerModule;

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.base_mut().on_object_created(sim, ctx);
    }

    fn on_special_power_creation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        SpecialPowerFacet::on_special_power_creation(self.base_mut(), sim, ctx);
    }

    fn get_reference_thing_template_name(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
    ) -> Option<String> {
        None
    }

    fn do_special_power(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        command_options: UnsignedInt,
    ) {
        SpecialPowerFacet::do_special_power(self.base_mut(), sim, ctx, command_options);
    }

    fn do_special_power_at_object(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: ObjectSlot,
        command_options: UnsignedInt,
    ) {
        SpecialPowerFacet::do_special_power_at_object(
            self.base_mut(),
            sim,
            ctx,
            target,
            command_options,
        );
    }

    fn do_special_power_at_location(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        loc: &Coord3D,
        angle: Real,
        command_options: UnsignedInt,
    ) {
        SpecialPowerFacet::do_special_power_at_location(
            self.base_mut(),
            sim,
            ctx,
            loc,
            angle,
            command_options,
        );
    }

    fn do_special_power_using_waypoints(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        pts: &[Coord3D],
        command_options: UnsignedInt,
    ) {
        SpecialPowerFacet::do_special_power_using_waypoints(
            self.base_mut(),
            sim,
            ctx,
            pts,
            command_options,
        );
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), zh_common::xfer::XferError> {
        self.base_mut().xfer_subclass(xfer)
    }
}

/// Implements `BehaviorModule` and `SpecialPowerFacet` by forwarding shared timer/lifecycle
/// logic to `self.base()` and delegating overridable hooks through `SpecialPowerBehaviorImpl`.
macro_rules! impl_special_power_behavior {
    ($ty:ty) => {
        impl $crate::modules::module::BehaviorModule for $ty {
            fn module_name_key(&self) -> zh_common::types::NameKeyType {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .module_name_key
            }

            fn on_object_created(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
            ) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::on_object_created(
                    self, sim, ctx,
                );
            }

            fn on_delete(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
            ) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base_mut(self)
                    .on_delete(sim, ctx);
            }

            fn on_disabled_edge(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
                disabled: zh_common::types::Bool,
            ) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base_mut(self)
                    .on_disabled_edge(sim, ctx, disabled);
            }

            fn apply_deferred_op(
                &mut self,
                op: $crate::arena::DeferredSelfOp,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
            ) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base_mut(self)
                    .apply_deferred_op(op, sim, ctx);
            }

            fn sync_sidecar(&self) -> $crate::arena::ModuleSidecar {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .sync_sidecar()
            }

            fn crc(&self, xfer: &mut dyn zh_common::xfer::Xfer) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .crc(xfer);
            }

            fn xfer(
                &mut self,
                xfer: &mut dyn zh_common::xfer::Xfer,
                _sim: &dyn $crate::sim::SimView,
            ) -> Result<(), zh_common::xfer::XferError> {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::xfer(
                    self, xfer,
                )
            }

            fn load_post_process(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
            ) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base_mut(self)
                    .load_post_process(sim, ctx);
            }

            fn as_special_power(&self) -> Option<&dyn $crate::modules::module::SpecialPowerFacet> {
                Some(self)
            }

            fn as_special_power_mut(
                &mut self,
            ) -> Option<&mut dyn $crate::modules::module::SpecialPowerFacet> {
                Some(self)
            }

            fn as_any(&self) -> Option<&dyn std::any::Any> {
                Some(self)
            }

            fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
                Some(self)
            }
        }

        impl $crate::modules::module::SpecialPowerFacet for $ty {
            fn is_module_for_power(
                &self,
                sim: &dyn $crate::sim::SimView,
                ctx: $crate::arena::ModuleContext,
                template_id: zh_common::types::UnsignedInt,
            ) -> zh_common::types::Bool {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .is_module_for_power(sim, ctx, template_id)
            }

            fn is_ready(
                &self,
                sim: &dyn $crate::sim::SimView,
                ctx: $crate::arena::ModuleContext,
            ) -> zh_common::types::Bool {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .is_ready(sim, ctx)
            }

            fn get_percent_ready(
                &self,
                sim: &dyn $crate::sim::SimView,
                ctx: $crate::arena::ModuleContext,
            ) -> zh_common::types::Real {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .get_percent_ready(sim, ctx)
            }

            fn get_ready_frame(
                &self,
                sim: &dyn $crate::sim::SimView,
                ctx: $crate::arena::ModuleContext,
            ) -> zh_common::types::UnsignedInt {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .get_ready_frame(sim, ctx)
            }

            fn get_raw_ready_frame(&self) -> zh_common::types::UnsignedInt {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .get_raw_ready_frame()
            }

            fn get_power_name(
                &self,
                sim: &dyn $crate::sim::SimView,
                ctx: $crate::arena::ModuleContext,
            ) -> String {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .get_power_name(sim, ctx)
            }

            fn get_special_power_id(
                &self,
                sim: &dyn $crate::sim::SimView,
                ctx: $crate::arena::ModuleContext,
            ) -> zh_common::types::UnsignedInt {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .get_special_power_id(sim, ctx)
            }

            fn has_special_power(
                &self,
                sim: &dyn $crate::sim::SimView,
                ctx: $crate::arena::ModuleContext,
                sp_type: $crate::rules::special_power_types::SpecialPowerType,
            ) -> zh_common::types::Bool {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .has_special_power(sim, ctx, sp_type)
            }

            fn get_required_science(
                &self,
                sim: &dyn $crate::sim::SimView,
                ctx: $crate::arena::ModuleContext,
            ) -> $crate::rules::science::ScienceType {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .get_required_science(sim, ctx)
            }

            fn set_ready_frame(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
                frame: zh_common::types::UnsignedInt,
            ) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base_mut(self)
                    .set_ready_frame(sim, ctx, frame);
            }

            fn is_paused(&self) -> zh_common::types::Bool {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .is_paused()
            }

            fn pause_countdown(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
                pause: zh_common::types::Bool,
            ) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base_mut(self)
                    .pause_countdown(sim, ctx, pause);
            }

            fn starts_ready(
                &self,
                sim: &dyn $crate::sim::SimView,
                ctx: $crate::arena::ModuleContext,
            ) -> zh_common::types::Bool {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .starts_ready(sim, ctx)
            }

            fn is_script_only(
                &self,
                sim: &dyn $crate::sim::SimView,
                ctx: $crate::arena::ModuleContext,
            ) -> zh_common::types::Bool {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base(self)
                    .is_script_only(sim, ctx)
            }

            fn get_reference_thing_template_name(
                &self,
                sim: &dyn $crate::sim::SimView,
                ctx: $crate::arena::ModuleContext,
            ) -> Option<String> {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::get_reference_thing_template_name(
                    self, sim, ctx,
                )
            }

            fn on_special_power_creation(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
            ) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::on_special_power_creation(
                    self, sim, ctx,
                );
            }

            fn start_power_recharge(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
            ) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base_mut(self)
                    .start_power_recharge(sim, ctx);
            }

            fn mark_special_power_triggered(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
                loc: Option<&zh_common::math::Coord3D>,
            ) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::base_mut(self)
                    .mark_special_power_triggered(sim, ctx, loc);
            }

            fn do_special_power(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
                command_options: zh_common::types::UnsignedInt,
            ) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::do_special_power(
                    self,
                    sim,
                    ctx,
                    command_options,
                );
            }

            fn do_special_power_at_object(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
                target: $crate::arena::ObjectSlot,
                command_options: zh_common::types::UnsignedInt,
            ) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::do_special_power_at_object(
                    self,
                    sim,
                    ctx,
                    target,
                    command_options,
                );
            }

            fn do_special_power_at_location(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
                loc: &zh_common::math::Coord3D,
                angle: zh_common::types::Real,
                command_options: zh_common::types::UnsignedInt,
            ) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::do_special_power_at_location(
                    self,
                    sim,
                    ctx,
                    loc,
                    angle,
                    command_options,
                );
            }

            fn do_special_power_using_waypoints(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
                pts: &[zh_common::math::Coord3D],
                command_options: zh_common::types::UnsignedInt,
            ) {
                $crate::modules::special_power::special_power_module::SpecialPowerBehaviorImpl::do_special_power_using_waypoints(
                    self,
                    sim,
                    ctx,
                    pts,
                    command_options,
                );
            }
        }
    };
}

pub(crate) use impl_special_power_behavior;
