//! `PKT-262`: `BridgeTowerBehavior` (`BridgeTowerBehavior.h`, `BridgeTowerBehavior.cpp`).
//!
//! Packet: `packets/phase5/PKT-262_BridgeTowerBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_c3.rs`.
//!
//! The bridge answers for its towers through [`BridgeFacet`](crate::modules::module::BridgeFacet),
//! which `BridgeBehavior` implements. `SimContext::add_landmark_bridge` makes the towers
//! and links them with `set_bridge` and `set_tower_type`.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{
    xfer_behavior_module_header, xfer_module_version, BehaviorModuleData,
};
use crate::modules::contain::open_contain::kill_object;
use crate::modules::module::{
    BehaviorModule, BridgeTowerFacet, BridgeTowerType, DamageFacet, DieFacet, ModuleData,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::kind_of::KindOfType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Int, NameKeyType, ObjectId, Real, FALSE};
use zh_common::xfer::{Xfer, XferError};

const TOWERS: [BridgeTowerType; BridgeTowerType::MAX_TOWERS] = [
    BridgeTowerType::FromLeft,
    BridgeTowerType::FromRight,
    BridgeTowerType::ToLeft,
    BridgeTowerType::ToRight,
];

/// INI template data for `BridgeTowerBehavior`, which has no fields of its own.
#[derive(Debug, Clone, PartialEq)]
pub struct BridgeTowerBehaviorModuleData {
    pub base: BehaviorModuleData,
}

impl ModuleData for BridgeTowerBehaviorModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.base.tag_key()
    }

    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key()
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask(0x006 | FacetMask::BRIDGE_TOWER.0)
    }
}

/// Runtime instance of `BridgeTowerBehavior` (`PKT-262`).
pub struct BridgeTowerBehavior {
    pub data: BridgeTowerBehaviorModuleData,
    pub bridge_id: ObjectId,
    /// Not set by the C++ constructor; the module pool hands out zeroed memory.
    pub tower_type: Int,
}

impl BridgeTowerBehavior {
    pub fn new(data: BridgeTowerBehaviorModuleData) -> Self {
        Self {
            data,
            bridge_id: ObjectId::INVALID,
            tower_type: BridgeTowerType::FromLeft.as_i32(),
        }
    }

    /// The part `onDamage` and `onHealing` share: the bridge, the fraction of this tower's
    /// health that `amount` stands for, and whether it is to be passed on.
    fn propagation(
        &self,
        sim: &dyn SimView,
        owner: ObjectSlot,
        dmg: &DamageInfo,
    ) -> Option<(ObjectSlot, Real)> {
        let bridge = sim.find_object(self.bridge_id)?;
        let percentage = dmg.input.amount / sim.object_max_health(owner);
        // No `BridgeBehaviorInterface` on the bridge.
        sim.bridge_tower_id(bridge, BridgeTowerType::FromLeft)?;
        // Damage from the bridge or another tower is not passed on again.
        if let Some(source) = sim.find_object(dmg.input.source_id) {
            if sim.object_is_kind_of(source, KindOfType::Bridge) != FALSE
                || sim.object_is_kind_of(source, KindOfType::BridgeTower) != FALSE
            {
                return None;
            }
        }
        Some((bridge, percentage))
    }

    /// The other tower standing at `tower_type`.
    fn other_tower(
        sim: &dyn SimView,
        owner: ObjectSlot,
        bridge: ObjectSlot,
        tower_type: BridgeTowerType,
    ) -> Option<ObjectSlot> {
        sim.bridge_tower_id(bridge, tower_type)
            .and_then(|id| sim.find_object(id))
            .filter(|&tower| tower != owner)
    }
}

impl DamageFacet for BridgeTowerBehavior {
    fn on_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        let owner = ctx.owner_slot;
        let Some((bridge, percentage)) = self.propagation(sim, owner, dmg) else {
            return;
        };
        let owner_id = sim.object(owner).map_or(ctx.owner_id, |o| o.id);
        let (damage_type, death_type) = (dmg.input.damage_type, dmg.input.death_type);
        let pass_on = |sim: &mut dyn SimApi, victim: ObjectSlot| {
            let mut passed = DamageInfo::default();
            passed.input.amount = percentage * sim.object_max_health(victim);
            passed.input.source_id = owner_id;
            passed.input.damage_type = damage_type;
            passed.input.death_type = death_type;
            sim.attempt_damage(victim, &mut passed);
        };

        for tower_type in TOWERS {
            if let Some(tower) = Self::other_tower(sim, owner, bridge, tower_type) {
                pass_on(sim, tower);
            }
        }
        pass_on(sim, bridge);
    }

    fn on_healing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        let owner = ctx.owner_slot;
        let Some((bridge, percentage)) = self.propagation(sim, owner, dmg) else {
            return;
        };
        let owner_id = sim.object(owner).map_or(ctx.owner_id, |o| o.id);
        // `Object::attemptHealing(amount, source)`.
        let pass_on = |sim: &mut dyn SimApi, patient: ObjectSlot| {
            let mut healing = DamageInfo::default();
            healing.input.damage_type = DamageType::Healing;
            healing.input.death_type = DeathType::NoneType;
            healing.input.source_id = owner_id;
            healing.input.amount = percentage * sim.object_max_health(patient);
            sim.attempt_healing(patient, &mut healing);
        };

        for tower_type in TOWERS {
            if let Some(tower) = Self::other_tower(sim, owner, bridge, tower_type) {
                pass_on(sim, tower);
            }
        }
        pass_on(sim, bridge);
    }
}

impl DieFacet for BridgeTowerBehavior {
    fn on_die(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext, _dmg: &DamageInfo) {
        // Killing the bridge takes the towers with it.
        if let Some(bridge) = sim.find_object(self.bridge_id) {
            kill_object(sim, bridge, DamageType::Unresistable, DeathType::Normal);
        }
    }
}

impl BridgeTowerFacet for BridgeTowerBehavior {
    fn set_bridge(&mut self, bridge_id: ObjectId) {
        self.bridge_id = bridge_id;
    }

    fn get_bridge_id(&self) -> ObjectId {
        self.bridge_id
    }

    fn set_tower_type(&mut self, tower_type: BridgeTowerType) {
        self.tower_type = tower_type.as_i32();
    }
}

impl BehaviorModule for BridgeTowerBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key()
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        xfer_behavior_module_header(xfer)?;
        xfer.xfer_object_id(&mut self.bridge_id)?;
        let mut tower_type = self.tower_type.to_le_bytes();
        xfer.xfer_user(&mut tower_type)?;
        self.tower_type = Int::from_le_bytes(tower_type);
        Ok(())
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn as_damage_mut(&mut self) -> Option<&mut dyn DamageFacet> {
        Some(self)
    }

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }

    fn as_bridge_tower(&self) -> Option<&dyn BridgeTowerFacet> {
        Some(self)
    }

    fn as_bridge_tower_mut(&mut self) -> Option<&mut dyn BridgeTowerFacet> {
        Some(self)
    }
}

pub fn parse_bridge_tower_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(BridgeTowerBehaviorModuleData {
        base: BehaviorModuleData::new(tag_key, module_name_key),
    }))
}

pub fn create_bridge_tower_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<BridgeTowerBehaviorModuleData>()
        .expect("BridgeTowerBehavior data type mismatch")
        .clone();
    Box::new(BridgeTowerBehavior::new(typed))
}
