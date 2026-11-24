//! `PKT-210`: `DemoralizeSpecialPower` (`DemoralizeSpecialPower.h` / `DemoralizeSpecialPower.cpp`).
//!
//! Compiled under `#ifdef ALLOW_DEMORALIZE` in C++ `ModuleFactory.cpp`, so it MUST NOT be
//! registered inside `register_special_power_modules`; tests register it separately via
//! [`super::register_demoralize_special_power`].

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::module::{BehaviorModule, ModuleData, SpecialPowerFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::special_power::special_power_module::{
    impl_special_power_behavior, owner_contain_count, parse_special_power_data_with,
    SpecialPowerBehaviorImpl, SpecialPowerModule, SpecialPowerModuleData,
};
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::Coord3D;
use zh_common::types::{NameKeyType, Real, Relationship, UnsignedInt, FALSE};

#[derive(Debug, Clone, PartialEq)]
pub struct DemoralizeSpecialPowerModuleData {
    pub base: SpecialPowerModuleData,
    pub base_range: Real,
    pub bonus_range_per_captured: Real,
    pub max_range: Real,
    pub base_duration_in_frames: UnsignedInt,
    pub bonus_duration_per_captured_in_frames: UnsignedInt,
    pub max_duration_in_frames: UnsignedInt,
    pub fx_list: Option<String>,
}

impl ModuleData for DemoralizeSpecialPowerModuleData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.base.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask::SPECIAL_POWER
    }
}

pub fn parse_demoralize_special_power_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut base_range: Real = 0.0;
    let mut bonus_range_per_captured: Real = 0.0;
    let mut max_range: Real = 0.0;
    let mut base_duration_in_frames: UnsignedInt = 0;
    let mut bonus_duration_per_captured_in_frames: UnsignedInt = 0;
    let mut max_duration_in_frames: UnsignedInt = 0;
    let mut fx_list: Option<String> = None;

    let base = parse_special_power_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, ctx| {
            if field.eq_ignore_ascii_case("BaseRange") {
                reader.parse_real(&mut base_range)
            } else if field.eq_ignore_ascii_case("BonusRangePerCaptured") {
                reader.parse_real(&mut bonus_range_per_captured)
            } else if field.eq_ignore_ascii_case("MaxRange") {
                reader.parse_real(&mut max_range)
            } else if field.eq_ignore_ascii_case("BaseDuration") {
                reader.parse_duration_unsigned_int(&mut base_duration_in_frames)
            } else if field.eq_ignore_ascii_case("BonusDurationPerCaptured") {
                reader.parse_duration_unsigned_int(&mut bonus_duration_per_captured_in_frames)
            } else if field.eq_ignore_ascii_case("MaxDuration") {
                reader.parse_duration_unsigned_int(&mut max_duration_in_frames)
            } else if field.eq_ignore_ascii_case("FXList") {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                if tok.eq_ignore_ascii_case("None") {
                    fx_list = None;
                } else {
                    let resolved = ctx
                        .fx_list_names
                        .iter()
                        .find(|n| n.eq_ignore_ascii_case(tok))
                        .cloned()
                        .unwrap_or_else(|| tok.to_owned());
                    fx_list = Some(resolved);
                }
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;

    Ok(Box::new(DemoralizeSpecialPowerModuleData {
        base,
        base_range,
        bonus_range_per_captured,
        max_range,
        base_duration_in_frames,
        bonus_duration_per_captured_in_frames,
        max_duration_in_frames,
        fx_list,
    }))
}

#[derive(Debug, Clone)]
pub struct DemoralizeSpecialPower {
    base: SpecialPowerModule,
    data: DemoralizeSpecialPowerModuleData,
}

impl DemoralizeSpecialPower {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<DemoralizeSpecialPowerModuleData>()
            .expect("DemoralizeSpecialPowerModuleData mismatch");
        Box::new(Self {
            base: SpecialPowerModule::new(typed.base.clone()),
            data: typed.clone(),
        })
    }
}

impl SpecialPowerBehaviorImpl for DemoralizeSpecialPower {
    fn base(&self) -> &SpecialPowerModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut SpecialPowerModule {
        &mut self.base
    }

    fn do_special_power_at_location(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        loc: &Coord3D,
        angle: Real,
        command_options: UnsignedInt,
    ) {
        if SpecialPowerModule::is_owner_disabled(sim, ctx) {
            return;
        }
        SpecialPowerFacet::do_special_power_at_location(
            &mut self.base,
            sim,
            ctx,
            loc,
            angle,
            command_options,
        );

        let mut duration = self.data.base_duration_in_frames;
        let mut range = self.data.base_range;
        if let Some(count) = owner_contain_count(sim, ctx.owner_slot) {
            duration = duration.saturating_add(
                count.saturating_mul(self.data.bonus_duration_per_captured_in_frames),
            );
            if duration > self.data.max_duration_in_frames {
                duration = self.data.max_duration_in_frames;
            }
            range += (count as Real) * self.data.bonus_range_per_captured;
            if range > self.data.max_range {
                range = self.data.max_range;
            }
        }

        let source_masked = sim.object(ctx.owner_slot).map_or(FALSE, |o| {
            o.status.test(ObjectStatusType::Masked.as_i32())
        });
        let candidates = sim.find_objects_in_range(loc, range);
        for target in candidates {
            let rel = sim.player_relationship(ctx.owner_slot, target);
            if rel != Relationship::Enemies && rel != Relationship::Neutral {
                continue;
            }
            if sim.object_is_kind_of(target, KindOfType::Infantry) == FALSE {
                continue;
            }
            let target_masked = sim.object(target).map_or(FALSE, |o| {
                o.status.test(ObjectStatusType::Masked.as_i32())
            });
            if target_masked != source_masked {
                continue;
            }
            if sim.object_has_ai(target) != FALSE {
                sim.set_demoralized(target, duration);
            }
        }

        if let Some(ref fx) = self.data.fx_list {
            sim.trigger_fx_list(fx, loc);
        }
    }

    fn do_special_power_at_object(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: ObjectSlot,
        command_options: UnsignedInt,
    ) {
        if SpecialPowerModule::is_owner_disabled(sim, ctx) {
            return;
        }
        if let Some(pos) = sim.object(target).map(|o| *o.thing.get_position()) {
            SpecialPowerFacet::do_special_power_at_location(
                self,
                sim,
                ctx,
                &pos,
                0.0,
                command_options,
            );
        }
    }
}

impl_special_power_behavior!(DemoralizeSpecialPower);
