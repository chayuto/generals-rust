//! `PKT-167`: `BoneFXDamage` (`BoneFXDamage.h` / `BoneFXDamage.cpp`).

use crate::arena::ModuleContext;
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::BodyDamageType;
use crate::modules::damage::damage_module::{DamageModuleBase, DamageModuleData};
use crate::modules::module::{BehaviorModule, DamageFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::types::NameKeyType;
use zh_common::xfer::{Xfer, XferError};

pub fn parse_bone_fx_damage_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    ini.read_block_fields(|_field, _reader| Err(IniError::UnknownToken))?;
    Ok(Box::new(DamageModuleData {
        tag_key,
        module_name_key,
    }))
}

#[derive(Debug, Clone)]
pub struct BoneFXDamage {
    base: DamageModuleBase,
    has_bone_fx_update: bool,
}

impl BoneFXDamage {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        Box::new(Self {
            base: DamageModuleBase::new(data.module_name_key()),
            has_bone_fx_update: false,
        })
    }

    #[inline]
    pub const fn has_bone_fx_update(&self) -> bool {
        self.has_bone_fx_update
    }
}

impl BehaviorModule for BoneFXDamage {
    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let bone_fx_key = NameKeyType::INVALID;
        // Look up already-interned `"BoneFXUpdate"` cleanly if present without perturbing NameKeys.
        if let Some(desc) = sim
            .rules()
            .module_registry
            .find_descriptor("BoneFXUpdate", crate::modules::ModuleType::Behavior)
        {
            // ModuleRegistry entries are registered with decorated keys; match via name lookup on existing keys.
            let _ = desc;
        }
        // Check via module name key stored in BoneFXUpdate payloads when BoneFXUpdate is ported.
        if let Some(obj) = sim.object(ctx.owner_slot) {
            for &(ms, _) in &obj.behaviors {
                let _ = ms;
            }
        }
        if bone_fx_key != NameKeyType::INVALID {
            self.has_bone_fx_update = sim.has_module_by_name_key(ctx.owner_slot, bone_fx_key);
        }
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        self.base.xfer_subclass(xfer)
    }

    fn as_damage_mut(&mut self) -> Option<&mut dyn DamageFacet> {
        Some(self)
    }
}

impl DamageFacet for BoneFXDamage {
    fn on_damage(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _dmg: &mut DamageInfo) {}

    fn on_body_damage_state_change(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _dmg: &DamageInfo,
        old_state: BodyDamageType,
        new_state: BodyDamageType,
    ) {
        sim.notify_bone_fx_damage_state_change(ctx.owner_slot, old_state, new_state);
    }
}
