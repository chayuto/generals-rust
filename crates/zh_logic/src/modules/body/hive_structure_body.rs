//! `PKT-173`: `HiveStructureBody` (`HiveStructureBody.h` / `HiveStructureBody.cpp`).

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext, ModuleSidecar};
use crate::combat::damage::DamageInfo;
use crate::delegate_body_facet_to_inner;
use crate::modules::body::active_body::ActiveBodyModuleData;
use crate::modules::body::structure_body::StructureBody;
use crate::modules::module::{BehaviorModule, BodyFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{get_damage_type_flag, parse_damage_type_flags, DamageTypeFlags};
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{NameKeyType, UnsignedInt, XferVersion, FALSE};
use zh_common::xfer::{Xfer, XferError};

/// Template configuration parsed for `HiveStructureBody`.
#[derive(Debug, Clone, PartialEq)]
pub struct HiveStructureBodyModuleData {
    pub base: ActiveBodyModuleData,
    pub damage_types_to_propagate_to_slaves: DamageTypeFlags,
    pub damage_types_to_swallow: DamageTypeFlags,
}

impl HiveStructureBodyModuleData {
    pub const fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            base: ActiveBodyModuleData::new(tag_key, module_name_key),
            damage_types_to_propagate_to_slaves: DamageTypeFlags::new(),
            damage_types_to_swallow: DamageTypeFlags::new(),
        }
    }
}

impl ModuleData for HiveStructureBodyModuleData {
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
        FacetMask::BODY
    }

    fn subdual_damage_heal_rate(&self) -> UnsignedInt {
        self.base.subdual_damage_heal_rate
    }

    fn subdual_damage_heal_amount(&self) -> zh_common::types::Real {
        self.base.subdual_damage_heal_amount
    }
}

pub fn parse_hive_structure_body_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = HiveStructureBodyModuleData::new(tag_key, module_name_key);
    ini.read_block_fields(|field, reader| {
        if data.base.parse_field(field, reader)? {
            Ok(())
        } else if field.eq_ignore_ascii_case("PropagateDamageTypesToSlavesWhenExisting") {
            data.damage_types_to_propagate_to_slaves = parse_damage_type_flags(reader)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("SwallowDamageTypesIfSlavesNotExisting") {
            data.damage_types_to_swallow = parse_damage_type_flags(reader)?;
            Ok(())
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

/// Structure body capable of redirecting selected damage types to spawned slaves or riders
/// (`HiveStructureBody.h` / `HiveStructureBody.cpp`).
#[derive(Debug, Clone)]
pub struct HiveStructureBody {
    pub base: StructureBody,
    pub damage_types_to_propagate_to_slaves: DamageTypeFlags,
    pub damage_types_to_swallow: DamageTypeFlags,
}

impl HiveStructureBody {
    pub fn from_module_data(
        module_name_key: NameKeyType,
        data: &HiveStructureBodyModuleData,
    ) -> Self {
        Self {
            base: StructureBody::from_active_data(module_name_key, &data.base),
            damage_types_to_propagate_to_slaves: data.damage_types_to_propagate_to_slaves.clone(),
            damage_types_to_swallow: data.damage_types_to_swallow.clone(),
        }
    }

    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let hbd = data
            .as_any()
            .downcast_ref::<HiveStructureBodyModuleData>()
            .expect("HiveStructureBodyModuleData");
        Box::new(Self::from_module_data(data.module_name_key(), hbd))
    }
}

impl BehaviorModule for HiveStructureBody {
    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key()
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.base.on_object_created(sim, ctx);
    }

    fn apply_deferred_op(&mut self, op: DeferredSelfOp, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.base.apply_deferred_op(op, sim, ctx);
    }

    fn sync_sidecar(&self) -> ModuleSidecar {
        self.base.sync_sidecar()
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.base.xfer(xfer, sim)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.base.load_post_process(sim, ctx);
    }

    fn as_body(&self) -> Option<&dyn BodyFacet> {
        Some(self)
    }

    fn as_body_mut(&mut self) -> Option<&mut dyn BodyFacet> {
        Some(self)
    }
}

impl BodyFacet for HiveStructureBody {
    fn attempt_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        let dt = dmg.input.damage_type.as_i32();
        if get_damage_type_flag(&self.damage_types_to_propagate_to_slaves, dt) != FALSE {
            let hive = ctx.owner_slot;
            if let Some(shooter_pos) = sim
                .find_object(dmg.input.source_id)
                .and_then(|s| sim.object(s))
                .map(|o| *o.thing.get_position())
            {
                // The spawns shelter the hive; without a spawn behavior its riders do.
                let heir = sim
                    .spawn_closest_slave(hive, &shooter_pos)
                    .or_else(|| sim.contain_closest_rider(hive, &shooter_pos));
                match heir {
                    Some(Some(slave)) => {
                        sim.attempt_damage(slave, dmg);
                        return;
                    }
                    Some(None) => {
                        if get_damage_type_flag(&self.damage_types_to_swallow, dt) != FALSE {
                            dmg.output.actual_damage_dealt = 0.0;
                            dmg.output.actual_damage_clipped = 0.0;
                            dmg.output.no_effect = zh_common::types::TRUE;
                            return;
                        }
                    }
                    None => {}
                }
            }
        }
        self.base.attempt_damage(sim, ctx, dmg);
    }

    fn set_constructor_object(&mut self, constructor_id: zh_common::types::ObjectId) {
        self.base.set_constructor_id(constructor_id);
    }

    delegate_body_facet_to_inner!(base);
}
