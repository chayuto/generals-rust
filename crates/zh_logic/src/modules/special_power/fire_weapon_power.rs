//! `PKT-216`: `FireWeaponPower` (`FireWeaponPower.h` / `FireWeaponPower.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::module::{BehaviorModule, ModuleData, SpecialPowerFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::special_power::special_power_module::{
    impl_special_power_behavior, parse_special_power_data_with, SpecialPowerBehaviorImpl,
    SpecialPowerModule, SpecialPowerModuleData,
};
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{NameKeyType, Real, UnsignedInt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireWeaponPowerModuleData {
    pub base: SpecialPowerModuleData,
    pub max_shots_to_fire: UnsignedInt,
}

impl ModuleData for FireWeaponPowerModuleData {
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

pub fn parse_fire_weapon_power_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut max_shots_to_fire: UnsignedInt = 1;
    let base = parse_special_power_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("MaxShotsToFire") {
                reader.parse_unsigned_int(&mut max_shots_to_fire)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(FireWeaponPowerModuleData {
        base,
        max_shots_to_fire,
    }))
}

#[derive(Debug, Clone)]
pub struct FireWeaponPower {
    base: SpecialPowerModule,
    pub max_shots_to_fire: UnsignedInt,
}

impl FireWeaponPower {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<FireWeaponPowerModuleData>()
            .expect("FireWeaponPowerModuleData mismatch");
        Box::new(Self {
            base: SpecialPowerModule::new(typed.base.clone()),
            max_shots_to_fire: typed.max_shots_to_fire,
        })
    }

    fn resolve_primary_weapon_name(sim: &dyn SimView, ctx: ModuleContext) -> String {
        let Some(obj) = sim.object(ctx.owner_slot) else {
            return String::new();
        };
        obj.thing
            .template_override()
            .get_final(&sim.rules().thing_factory)
            .and_then(|t| t.find_weapon_template_set(&obj.weapon_set_flags))
            .and_then(|s| s.templates[0].as_ref())
            .map(|w| w.name.clone())
            .unwrap_or_default()
    }
}

impl SpecialPowerBehaviorImpl for FireWeaponPower {
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
        let pos = sim
            .object(ctx.owner_slot)
            .map(|o| *o.thing.get_position())
            .unwrap_or_default();
        let wpn = Self::resolve_primary_weapon_name(sim, ctx);
        sim.fire_weapon_template(ctx.owner_slot, &wpn, None, &pos);
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
        let wpn = Self::resolve_primary_weapon_name(sim, ctx);
        sim.fire_weapon_template(ctx.owner_slot, &wpn, None, loc);
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
        SpecialPowerFacet::do_special_power_at_object(
            &mut self.base,
            sim,
            ctx,
            target,
            command_options,
        );
        let target_pos = sim
            .object(target)
            .map(|o| *o.thing.get_position())
            .unwrap_or_default();
        let wpn = Self::resolve_primary_weapon_name(sim, ctx);
        sim.fire_weapon_template(ctx.owner_slot, &wpn, Some(target), &target_pos);
    }
}

impl_special_power_behavior!(FireWeaponPower);
