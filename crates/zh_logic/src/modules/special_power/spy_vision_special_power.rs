//! `PKT-245`: `SpyVisionSpecialPower` (`SpyVisionSpecialPower.h` / `SpyVisionSpecialPower.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData, SpecialPowerFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::special_power::special_power_module::{
    impl_special_power_behavior, owner_contain_count, parse_special_power_data_with,
    SpecialPowerBehaviorImpl, SpecialPowerModule, SpecialPowerModuleData,
};
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{NameKeyType, UnsignedInt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpyVisionSpecialPowerModuleData {
    pub base: SpecialPowerModuleData,
    pub base_duration_in_frames: UnsignedInt,
    pub bonus_duration_per_captured_in_frames: UnsignedInt,
    pub max_duration_in_frames: UnsignedInt,
}

impl ModuleData for SpyVisionSpecialPowerModuleData {
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

pub fn parse_spy_vision_special_power_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut base_duration_in_frames: UnsignedInt = 0;
    let mut bonus_duration_per_captured_in_frames: UnsignedInt = 0;
    let mut max_duration_in_frames: UnsignedInt = 0;

    let base = parse_special_power_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("BaseDuration") {
                reader.parse_duration_unsigned_int(&mut base_duration_in_frames)
            } else if field.eq_ignore_ascii_case("BonusDurationPerCaptured") {
                reader.parse_duration_unsigned_int(&mut bonus_duration_per_captured_in_frames)
            } else if field.eq_ignore_ascii_case("MaxDuration") {
                reader.parse_duration_unsigned_int(&mut max_duration_in_frames)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;

    Ok(Box::new(SpyVisionSpecialPowerModuleData {
        base,
        base_duration_in_frames,
        bonus_duration_per_captured_in_frames,
        max_duration_in_frames,
    }))
}

#[derive(Debug, Clone)]
pub struct SpyVisionSpecialPower {
    base: SpecialPowerModule,
    data: SpyVisionSpecialPowerModuleData,
}

impl SpyVisionSpecialPower {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<SpyVisionSpecialPowerModuleData>()
            .expect("SpyVisionSpecialPowerModuleData mismatch");
        Box::new(Self {
            base: SpecialPowerModule::new(typed.base.clone()),
            data: typed.clone(),
        })
    }

    /// Computes the spy-vision duration scaled by the owner's contain count (`SpyVisionSpecialPower::doSpecialPower`).
    pub fn compute_duration(&self, sim: &dyn SimView, owner: crate::arena::ObjectSlot) -> UnsignedInt {
        let mut duration = self.data.base_duration_in_frames;
        if let Some(count) = owner_contain_count(sim, owner) {
            duration = duration.saturating_add(
                count.saturating_mul(self.data.bonus_duration_per_captured_in_frames),
            );
            if duration > self.data.max_duration_in_frames {
                duration = self.data.max_duration_in_frames;
            }
        }
        duration
    }
}

impl SpecialPowerBehaviorImpl for SpyVisionSpecialPower {
    fn base(&self) -> &SpecialPowerModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut SpecialPowerModule {
        &mut self.base
    }

    fn do_special_power(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        command_options: UnsignedInt,
    ) {
        if SpecialPowerModule::is_owner_disabled(sim, ctx) {
            return;
        }
        SpecialPowerFacet::do_special_power(&mut self.base, sim, ctx, command_options);

        let duration = self.compute_duration(sim, ctx.owner_slot);
        sim.activate_spy_vision(ctx.owner_slot, duration);
    }
}

impl_special_power_behavior!(SpyVisionSpecialPower);
