//! `PKT-215`: `FireWeaponCollide` (`FireWeaponCollide.h` / `FireWeaponCollide.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::collide::collide_module::{
    parse_collide_data_with, CollideBehaviorImpl, CollideModule,
};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::object::weapon::{Weapon, WeaponTemplate};
use crate::object::weapon_fire;
use crate::object::weapon_set::WeaponSlotType;
use crate::rules::object_status_types::{ObjectStatusMaskType, OBJECT_STATUS_NAMES};
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, NameKeyType, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError, XferMode};

/// Template configuration parsed for `FireWeaponCollide`.
#[derive(Debug, Clone, PartialEq)]
pub struct FireWeaponCollideModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub collide_weapon_template: Option<WeaponTemplate>,
    pub required_status: ObjectStatusMaskType,
    pub forbidden_status: ObjectStatusMaskType,
    pub fire_once: Bool,
}

impl ModuleData for FireWeaponCollideModuleData {
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
        FacetMask::COLLIDE
    }
}

pub fn parse_fire_weapon_collide_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut collide_weapon_template: Option<WeaponTemplate> = None;
    let mut required_status = ObjectStatusMaskType::new();
    let mut forbidden_status = ObjectStatusMaskType::new();
    let mut fire_once = FALSE;

    let _base = parse_collide_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, ctx| {
            if field.eq_ignore_ascii_case("CollideWeapon") {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                if tok.eq_ignore_ascii_case("None") {
                    collide_weapon_template = None;
                } else {
                    let Some(wt) = ctx.weapon_store.find_weapon_template(tok) else {
                        return Err(IniError::InvalidData);
                    };
                    collide_weapon_template = Some(wt.clone());
                }
                Ok(())
            } else if field.eq_ignore_ascii_case("FireOnce") {
                reader.parse_bool(&mut fire_once)
            } else if field.eq_ignore_ascii_case("RequiredStatus") {
                required_status.parse_from_ini(reader, &OBJECT_STATUS_NAMES)
            } else if field.eq_ignore_ascii_case("ForbiddenStatus") {
                forbidden_status.parse_from_ini(reader, &OBJECT_STATUS_NAMES)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;

    Ok(Box::new(FireWeaponCollideModuleData {
        tag_key,
        module_name_key,
        collide_weapon_template,
        required_status,
        forbidden_status,
        fire_once,
    }))
}

#[derive(Debug, Clone)]
pub struct FireWeaponCollide {
    base: CollideModule,
    required_status: ObjectStatusMaskType,
    forbidden_status: ObjectStatusMaskType,
    fire_once: Bool,
    pub collide_weapon: Option<Weapon>,
    pub ever_fired: Bool,
}

impl FireWeaponCollide {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<FireWeaponCollideModuleData>()
            .expect("FireWeaponCollideModuleData mismatch");
        let collide_weapon = typed
            .collide_weapon_template
            .as_ref()
            .map(|tmpl| Weapon::new(tmpl.clone(), WeaponSlotType::Primary, 0));
        Box::new(Self {
            base: CollideModule::new(typed.module_name_key),
            required_status: typed.required_status.clone(),
            forbidden_status: typed.forbidden_status.clone(),
            fire_once: typed.fire_once,
            collide_weapon,
            ever_fired: FALSE,
        })
    }

    pub fn should_fire_weapon(&self, sim: &dyn SimView, ctx: ModuleContext) -> Bool {
        let Some(obj) = sim.object(ctx.owner_slot) else {
            return FALSE;
        };
        if obj.status.test_for_all(&self.required_status) == FALSE {
            return FALSE;
        }
        if obj.status.test_for_any(&self.forbidden_status) != FALSE {
            return FALSE;
        }
        if self.ever_fired != FALSE && self.fire_once != FALSE {
            return FALSE;
        }
        TRUE
    }
}

impl CollideBehaviorImpl for FireWeaponCollide {
    fn collide_base(&self) -> &CollideModule {
        &self.base
    }

    fn collide_base_mut(&mut self) -> &mut CollideModule {
        &mut self.base
    }

    fn on_collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
        _loc: Option<&Coord3D>,
        _normal: Option<&Coord3D>,
    ) {
        let Some(other_slot) = other else {
            return;
        };
        if self.should_fire_weapon(sim, ctx) == FALSE {
            return;
        }
        if let Some(ref mut weapon) = self.collide_weapon {
            weapon_fire::load_ammo_now(weapon, sim, ctx.owner_slot);
            weapon_fire::fire_weapon_at_object(weapon, sim, ctx.owner_slot, other_slot);
        }
    }
}

impl BehaviorModule for FireWeaponCollide {
    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext) {
        let now = sim.frame();
        if let Some(ref mut w) = self.collide_weapon {
            w.suspend_fx_frame = now.wrapping_add(w.template.suspend_fx_delay);
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.base.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.base.xfer(xfer)?;

        let mut weapon_present: Bool = Bool::from(self.collide_weapon.is_some());
        xfer.xfer_bool(&mut weapon_present)?;
        if weapon_present != FALSE {
            if xfer.mode() == XferMode::Load && self.collide_weapon.is_none() {
                self.collide_weapon = Some(Weapon::new(
                    WeaponTemplate::default(),
                    WeaponSlotType::Primary,
                    0,
                ));
            }
            if let Some(ref mut w) = self.collide_weapon {
                xfer.xfer_snapshot(w)?;
            }
        } else if xfer.mode() == XferMode::Load {
            self.collide_weapon = None;
        }

        xfer.xfer_bool(&mut self.ever_fired)
    }

    fn as_collide(&self) -> Option<&dyn crate::modules::module::CollideFacet> {
        Some(self)
    }

    fn as_collide_mut(&mut self) -> Option<&mut dyn crate::modules::module::CollideFacet> {
        Some(self)
    }
}

impl crate::modules::module::CollideFacet for FireWeaponCollide {
    fn on_collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
        loc: Option<&Coord3D>,
        normal: Option<&Coord3D>,
    ) {
        CollideBehaviorImpl::on_collide(self, sim, ctx, other, loc, normal);
    }
}
