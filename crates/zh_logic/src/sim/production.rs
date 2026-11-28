//! What `ProductionUpdate` (`PKT-354`) calls outside its own module: `BuildAssistant`,
//! the `Player` build and upgrade checks, money, and the creation of the finished unit.
//! The player is reached through one of its teams.

use super::{SimApi, SimContext, SimView};
use crate::player::player_snapshots::UPGRADE_STATUS_IN_PRODUCTION;
use crate::arena::{DeferredSelfOp, FacetMask, ModuleSlot, ObjectSlot, ThingTemplateId};
use crate::economy::academy_stats::{academy_kind_bits, AcademyObjectInfo, AcademyStats};
use crate::economy::handicap::{Handicap, HandicapType};
use crate::player::player::Player;
use crate::rules::player_template::ProductionChangeMap;
use crate::economy::score_keeper::{ScoreKeeper, ScoreObjectView, ScoreTemplateDesc};
use crate::modules::module::{CanMakeType, ExitDoorType, ProductionFacet};
use crate::object::object_script_status_bits::{
    OBJECT_STATUS_SCRIPT_DISABLED, OBJECT_STATUS_SCRIPT_UNPOWERED,
};
use crate::object::thing_template::{
    BuildableStatus, ThingTemplate, ThingTemplateGlobalTunables, ThingTemplatePlayerView,
};
use crate::rules::command_set::{CommandButton, CommandButtonType};
use crate::rules::kind_of::{KindOfMaskType, KindOfType};
use crate::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use crate::rules::production_prerequisite::{PrereqPlayerView, ThingTemplateRef};
use crate::rules::science::ScienceType;
use crate::rules::upgrade::{UpgradeMaskType, UpgradeType};
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, NameKeyType, Real, TeamId, UnsignedInt, UnsignedShort, FALSE, TRUE,
};

/// The `Player` as the build cost, time and prerequisite code sees it.
struct PlayerView<'a> {
    ctx: &'a SimContext,
    team: TeamId,
}

impl PrereqPlayerView for PlayerView<'_> {
    fn has_science(&self, st: ScienceType) -> bool {
        self.ctx.player_has_science(self.team, st) != FALSE
    }

    /// `Player::countObjectsByThingTemplate` with `ignoreDead = false` and the default
    /// `ignoreUnderConstruction = TRUE`, the only way production reaches it.
    fn count_objects_by_thing_template(
        &self,
        tmpls: &[Option<ThingTemplateRef>],
        counts: &mut [Int],
    ) {
        counts.fill(0);
        for slot in self.ctx.objects_of_player(self.team) {
            let Some(obj) = self.ctx.world.objects.get(slot) else {
                continue;
            };
            if obj
                .status
                .test(ObjectStatusType::UnderConstruction.as_i32())
                != FALSE
            {
                continue;
            }
            let Some(own) = self.ctx.object_template(slot) else {
                continue;
            };
            let found = tmpls.iter().position(|t| {
                t.and_then(|r| self.ctx.template_by_id(ThingTemplateId(r.0 as u32)))
                    .is_some_and(|t| own.is_equivalent_to(t) != FALSE)
            });
            if let Some(i) = found {
                counts[i] += 1;
            }
        }
    }
}

impl PlayerView<'_> {
    /// `getProductionCostChangePercent` / `getProductionTimeChangePercent`.
    fn change_percent(
        &self,
        template_name: &str,
        changes: impl Fn(&Player) -> &ProductionChangeMap,
    ) -> Real {
        let key = self.ctx.rules.name_keys.lookup_key(template_name);
        self.ctx
            .player_of_team(self.team)
            .zip(key)
            .and_then(|(p, key)| changes(p).get(&key).copied())
            .unwrap_or(0.0)
    }
}

impl ThingTemplatePlayerView for PlayerView<'_> {
    fn as_prereq_view(&self) -> Option<&dyn PrereqPlayerView> {
        Some(self)
    }

    fn get_production_cost_change_percent(&self, template_name: &str) -> Real {
        self.change_percent(template_name, |p| &p.production_cost_changes)
    }

    fn get_production_cost_change_based_on_kind_of(&self, kindof: &KindOfMaskType) -> Real {
        self.ctx.production_cost_change(self.team, kindof)
    }

    fn get_production_time_change_percent(&self, template_name: &str) -> Real {
        self.change_percent(template_name, |p| &p.production_time_changes)
    }

    fn get_handicap(&self, ht: HandicapType, tmpl: &ThingTemplate) -> Real {
        let structure = tmpl.is_kind_of(KindOfType::Structure) != FALSE;
        self.ctx.player_of_team(self.team).map_or_else(
            || Handicap::new().get_handicap(ht, structure),
            |p| p.handicap.get_handicap(ht, structure),
        )
    }

    fn get_energy_supply_ratio(&self) -> Real {
        // A player that never had a power module has no production and no consumption,
        // which is a ratio of zero.
        let energy = self
            .ctx
            .controlling_player(self.team)
            .and_then(|p| self.ctx.world.players.player(p));
        energy.map_or(0.0, |p| p.energy.get_energy_supply_ratio(self.ctx.world.frame))
    }

    fn count_objects_by_template_ref(&self, tmpl: ThingTemplateRef) -> Int {
        let mut count = [0];
        self.count_objects_by_thing_template(&[Some(tmpl)], &mut count);
        count[0]
    }
}

impl SimContext {
    fn template_by_id(&self, id: ThingTemplateId) -> Option<&ThingTemplate> {
        self.rules
            .thing_factory
            .find_by_template_id_final(id.0 as UnsignedShort)
    }

    fn object_template(&self, owner: ObjectSlot) -> Option<&ThingTemplate> {
        self.world
            .objects
            .get(owner)?
            .thing
            .template_override()
            .get_final(&self.rules.thing_factory)
    }

    /// `Player::iterateObjects` for the player controlling `team`.
    fn objects_of_player(&self, team: TeamId) -> Vec<ObjectSlot> {
        self.player_object_slots(self.player_index_for_team(team))
    }

    fn facet_slots(&self, owner: ObjectSlot, facet: FacetMask) -> Vec<ModuleSlot> {
        self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(facet))
                .map(|&(s, _)| s)
                .collect()
        })
    }

    fn production_slot(&self, owner: ObjectSlot) -> Option<ModuleSlot> {
        self.facet_slots(owner, FacetMask::PRODUCTION_UPDATE)
            .first()
            .copied()
    }

    /// `Object::getProductionUpdateInterface`; `None` while that module is executing.
    pub fn production_module(&self, owner: ObjectSlot) -> Option<&dyn ProductionFacet> {
        let ms = self.production_slot(owner)?;
        self.world.modules.payload(ms)?.as_production()
    }

    /// Runs `f` on the production interface of `owner`; `None` without one.
    pub fn with_production<R>(
        &mut self,
        owner: ObjectSlot,
        f: impl FnOnce(&mut dyn ProductionFacet, &mut dyn SimApi, crate::arena::ModuleContext) -> R,
    ) -> Option<R> {
        let ms = self.production_slot(owner)?;
        self.dispatch_module(ms, |m, sim, ctx| {
            m.as_production_mut().map(|p| f(p, sim, ctx))
        })
        .ok()
        .flatten()
    }

    /// Queues a unit for an AI player (`ProductionUpdateInterface::queueCreateUnit`
    /// with `requestUniqueUnitID`).
    pub(crate) fn ai_queue_create_unit(
        &mut self,
        factory: ObjectSlot,
        tmpl: ThingTemplateId,
    ) -> bool {
        self.with_production(factory, |production, sim, ctx| {
            let id = production.request_unique_unit_id();
            production.queue_create_unit(sim, ctx, tmpl, id) != FALSE
        })
        .unwrap_or(false)
    }

    fn can_build_more_of_type(
        &self,
        team: TeamId,
        tmpl: &ThingTemplate,
        builder: ObjectSlot,
        own: Option<&dyn ProductionFacet>,
    ) -> bool {
        let max = tmpl.get_max_simultaneous_of_type(self.world.superweapon_restriction);
        if max == 0 {
            return true;
        }
        let link_key = tmpl.get_max_simultaneous_link_key();
        let check_queues = tmpl.is_kind_of(KindOfType::Structure) == FALSE;
        let id = ThingTemplateId(u32::from(tmpl.get_template_id()));
        let mut count: UnsignedInt = 0;
        for slot in self.objects_of_player(team) {
            if self
                .world
                .objects
                .get(slot)
                .is_none_or(|o| o.is_effectively_dead() != FALSE)
            {
                continue;
            }
            if let Some(other) = self.object_template(slot) {
                if tmpl.is_equivalent_to(other) != FALSE
                    || (link_key != NameKeyType::INVALID
                        && link_key == other.get_max_simultaneous_link_key())
                {
                    count = count.wrapping_add(1);
                }
            }
            if check_queues {
                // The builder's module may be the caller, so it comes in as `own`.
                let queue = match own {
                    Some(own) if slot == builder => Some(own),
                    _ => self.production_module(slot),
                };
                if let Some(queue) = queue {
                    count = count.wrapping_add(queue.count_unit_type_in_queue(id));
                }
            }
        }
        count < max
    }

    fn allowed_to_build_template(&self, team: TeamId, tmpl: &ThingTemplate) -> bool {
        let structure = tmpl.is_kind_of(KindOfType::Structure) != FALSE;
        self.player_of_team(team).is_none_or(|p| {
            if structure {
                p.can_build_base
            } else {
                p.can_build_units
            }
        })
    }

    /// `Player::canBuild`.
    fn player_can_build(
        &self,
        team: TeamId,
        tmpl: &ThingTemplate,
        builder: ObjectSlot,
        own: Option<&dyn ProductionFacet>,
    ) -> bool {
        if !self.allowed_to_build_template(team, tmpl) {
            return false;
        }
        match tmpl.get_buildable(Some(&self.world.buildable_overrides)) {
            BuildableStatus::No => return false,
            BuildableStatus::IgnorePrerequisites => return true,
            BuildableStatus::OnlyByAi if self.player_is_human(team) != FALSE => return false,
            _ => {}
        }
        let view = PlayerView { ctx: self, team };
        let prerequisites_met = (0..tmpl.get_prereq_count() as usize)
            .filter_map(|i| tmpl.get_nth_prereq(i))
            .all(|pre| pre.is_satisfied(Some(&view)) != FALSE);
        prerequisites_met && self.can_build_more_of_type(team, tmpl, builder, own)
    }

    pub(crate) fn command_set_of(&self, owner: ObjectSlot) -> Option<&[CommandButton]> {
        let obj = self.world.objects.get(owner)?;
        let name = if obj.command_set_override.is_empty() {
            self.object_template(owner)?.friend_get_command_set_string()
        } else {
            &obj.command_set_override
        };
        self.rules.command_sets.find(name)
    }

    /// `BuildAssistant::isPossibleToMakeUnit`: the last matching button of the
    /// builder's command set wins, and it is the button's template that the
    /// player must be able to build.
    pub(crate) fn is_possible_to_make_unit(
        &self,
        builder: ObjectSlot,
        what: ThingTemplateId,
        own: Option<&dyn ProductionFacet>,
    ) -> bool {
        let (Some(obj), Some(tmpl)) = (self.world.objects.get(builder), self.template_by_id(what))
        else {
            return false;
        };
        let button_template = self.command_set_of(builder).and_then(|buttons| {
            buttons
                .iter()
                .filter(|b| b.command != CommandButtonType::Other)
                .filter_map(|b| self.rules.thing_factory.find_template(&b.object))
                .filter(|t| t.is_equivalent_to(tmpl) != FALSE)
                .last()
        });
        button_template.is_some_and(|t| self.player_can_build(obj.team_id, t, builder, own))
    }

    /// `BuildAssistant::canMakeUnit`. `own` is the builder's production module when the
    /// caller already holds it.
    pub(super) fn can_make_unit_for(
        &self,
        builder: ObjectSlot,
        what: ThingTemplateId,
        own: Option<&dyn ProductionFacet>,
    ) -> CanMakeType {
        let (Some(obj), Some(tmpl)) = (self.world.objects.get(builder), self.template_by_id(what))
        else {
            return CanMakeType::NoPrereq;
        };
        let script_blocks = OBJECT_STATUS_SCRIPT_DISABLED | OBJECT_STATUS_SCRIPT_UNPOWERED;
        if obj.script_status & script_blocks != 0 {
            return CanMakeType::FactoryIsDisabled;
        }
        let team = obj.team_id;
        let own = own.or_else(|| self.production_module(builder));
        if own.is_some_and(|p| p.special_power_construction_template() == Some(what)) {
            return CanMakeType::Ok;
        }
        if !self.can_build_more_of_type(team, tmpl, builder, own) {
            return CanMakeType::MaxedOutForPlayer;
        }
        if !self.is_possible_to_make_unit(builder, what, own) {
            return CanMakeType::NoPrereq;
        }
        if let (Some(queue), Some(ms)) = (own, self.production_slot(builder)) {
            let ctx = self.world.modules.meta(ms).context(ms);
            let can = queue.can_queue_create_unit(self, ctx, what);
            if can != CanMakeType::Ok {
                return can;
            }
        }
        if self.unit_cost_for(team, what) as UnsignedInt > self.player_money(team) {
            return CanMakeType::NoMoney;
        }
        CanMakeType::Ok
    }

    /// `ThingTemplate::calcCostToBuild`.
    pub(super) fn unit_cost_for(&self, team: TeamId, tmpl: ThingTemplateId) -> Int {
        let view = PlayerView { ctx: self, team };
        self.template_by_id(tmpl)
            .map_or(0, |t| t.calc_cost_to_build(&view))
    }

    /// `ThingTemplate::calcTimeToBuild`.
    pub(super) fn unit_time_for(&self, team: TeamId, tmpl: ThingTemplateId) -> Int {
        let view = PlayerView { ctx: self, team };
        let global = &self.rules.global_data;
        let tunables = ThingTemplateGlobalTunables {
            low_energy_penalty_modifier: global.low_energy_penalty_modifier,
            min_low_energy_production_speed: global.min_low_energy_production_speed,
            max_low_energy_production_speed: global.max_low_energy_production_speed,
            multiple_factory: global.multiple_factory,
        };
        self.template_by_id(tmpl)
            .map_or(0, |t| t.calc_time_to_build(&view, &tunables))
    }

    /// `Player::allowedToBuild`.
    pub(super) fn allowed_to_build_for(&self, team: TeamId, tmpl: ThingTemplateId) -> Bool {
        Bool::from(
            self.template_by_id(tmpl)
                .is_some_and(|t| self.allowed_to_build_template(team, t)),
        )
    }

    /// `Object::canProduceUpgrade`.
    pub(super) fn can_produce_upgrade_for(&self, owner: ObjectSlot, upgrade: NameKeyType) -> Bool {
        let names_it = |b: &CommandButton| {
            !b.upgrade.is_empty() && self.rules.name_keys.lookup_key(&b.upgrade) == Some(upgrade)
        };
        Bool::from(
            self.command_set_of(owner)
                .is_some_and(|buttons| buttons.iter().any(names_it)),
        )
    }

    /// `Object::affectedByUpgrade`.
    pub(super) fn affected_by_upgrade_for(&self, owner: ObjectSlot, upgrade: NameKeyType) -> Bool {
        let (Some(obj), Some(template)) = (
            self.world.objects.get(owner),
            self.rules.upgrade_center.find_upgrade_by_key(upgrade),
        ) else {
            return FALSE;
        };
        let empty = UpgradeMaskType::new();
        let mut mask = self
            .controlling_player(obj.team_id)
            .and_then(|p| self.world.players.player(p))
            .map(|p| p.upgrades_completed.clone())
            .unwrap_or_default();
        mask.clear_and_set(&empty, &obj.upgrades_completed);
        mask.clear_and_set(&empty, template.upgrade_mask());
        let affected = self
            .facet_slots(owner, FacetMask::UPGRADE)
            .into_iter()
            .any(|ms| {
                let ctx = self.world.modules.meta(ms).context(ms);
                self.world
                    .modules
                    .payload(ms)
                    .and_then(|m| m.as_upgrade())
                    .is_some_and(|u| u.would_upgrade(self, ctx, &mask) != FALSE)
            });
        Bool::from(affected)
    }

    /// Whether any `ParkingPlaceBehavior` wants a door reserved at queue time, and whether
    /// one of those has no space left.
    pub(super) fn parking_requirement_for(
        &self,
        owner: ObjectSlot,
        tmpl: ThingTemplateId,
    ) -> (Bool, Bool) {
        let (mut reserve, mut full) = (FALSE, FALSE);
        for ms in self.facet_slots(owner, FacetMask::PARKING_PLACE) {
            let payload = self.world.modules.payload(ms);
            let Some(parking) = payload.and_then(|m| m.as_parking_place()) else {
                continue;
            };
            if parking.should_reserve_door_when_queued(self, tmpl) != FALSE {
                reserve = TRUE;
                if parking.has_available_space_for(self, tmpl) == FALSE {
                    full = TRUE;
                }
            }
        }
        (reserve, full)
    }

    /// `Object::getObjectExitInterface`.
    fn exit_module(&self, owner: ObjectSlot) -> Option<ModuleSlot> {
        let obj = self.world.objects.get(owner)?;
        let ms = obj.exit_slot.or(obj.contain_slot)?;
        let has = self.world.modules.is_executing(ms)
            || self
                .world
                .modules
                .payload(ms)
                .is_some_and(|m| m.as_exit().is_some());
        has.then_some(ms)
    }

    pub(super) fn has_exit_interface(&self, owner: ObjectSlot) -> Bool {
        Bool::from(self.exit_module(owner).is_some())
    }

    pub(super) fn reserve_exit_door_of(
        &mut self,
        owner: ObjectSlot,
        tmpl: ThingTemplateId,
    ) -> ExitDoorType {
        self.exit_module(owner)
            .and_then(|ms| {
                self.dispatch_module(ms, |m, sim, ctx| {
                    m.as_exit_mut()
                        .map(|e| e.reserve_door_for_exit(sim, ctx, tmpl, None))
                })
                .ok()
                .flatten()
            })
            .unwrap_or(ExitDoorType::DoorNoneAvailable)
    }

    pub(super) fn unreserve_exit_door_of(&mut self, owner: ObjectSlot, door: ExitDoorType) {
        if let Some(ms) = self.exit_module(owner) {
            let _ = self.dispatch_module(ms, |m, _, _| {
                if let Some(e) = m.as_exit_mut() {
                    e.unreserve_door_for_exit(door);
                }
            });
        }
    }

    /// `ProductionUpdateInterface::setHoldDoorOpen`, also from inside the factory's own
    /// production update (an exit module reserving a hangar for the new aircraft).
    pub(super) fn hold_production_door(
        &mut self,
        owner: ObjectSlot,
        door: ExitDoorType,
        hold: Bool,
    ) {
        let Some(ms) = self.production_slot(owner) else {
            return;
        };
        if self.world.modules.is_executing(ms) {
            let op = DeferredSelfOp::ProductionHoldDoor {
                door: door.as_i32(),
                hold,
            };
            let _ = self.world.modules.push_deferred_op(ms, op);
        } else {
            let _ = self.dispatch_module(ms, |m, sim, _| {
                if let Some(p) = m.as_production_mut() {
                    p.set_hold_door_open(&*sim, door, hold);
                }
            });
        }
    }

    /// `getControllingPlayer()->getAcademyStats()` of a team.
    pub(super) fn academy(&mut self, team: TeamId) -> Option<&mut AcademyStats> {
        let player = self.player_index_for_team(team);
        self.world
            .players
            .player_mut(player)
            .map(|p| &mut p.academy)
    }

    /// `getControllingPlayer()->getScoreKeeper()` of a team.
    pub(super) fn score_keeper(&mut self, team: TeamId) -> Option<&mut ScoreKeeper> {
        let player = self.player_index_for_team(team);
        self.world
            .players
            .player_mut(player)
            .map(|p| &mut p.score_keeper)
    }

    /// What the score keeper counts an object by.
    fn score_desc(&self, owner: ObjectSlot) -> Option<ScoreTemplateDesc> {
        let t = self.object_template(owner)?;
        let id = u32::from(t.get_template_id());
        Some(ScoreTemplateDesc {
            id,
            name: t.get_name().to_owned(),
            kind_of: t.kindof.clone(),
            equivalent_root_id: t.reskinned_from_id.map_or(id, u32::from),
        })
    }

    /// `getScoreKeeper()->addObjectCaptured` of `Object::onCapture`.
    pub(super) fn record_object_captured(&mut self, owner: ObjectSlot, new_team: TeamId) {
        let Some(under_construction) = self
            .world
            .objects
            .get(owner)
            .map(|o| o.status.test(ObjectStatusType::UnderConstruction.as_i32()))
        else {
            return;
        };
        let Some(desc) = self.score_desc(owner) else {
            return;
        };
        let scoring = self.world.scoring_enabled;
        let controlling_player_idx = self.player_index_for_team(new_team);
        if let Some(score_keeper) = self.score_keeper(new_team) {
            score_keeper.add_object_captured(
            &ScoreObjectView {
                template: &desc,
                controlling_player_idx,
                under_construction,
            },
            scoring,
        );
        }
    }

    fn upgrade_name_and_mask(&self, upgrade: NameKeyType) -> Option<(String, UpgradeMaskType)> {
        self.rules
            .upgrade_center
            .find_upgrade_by_key(upgrade)
            .map(|u| (u.upgrade_name().to_owned(), u.upgrade_mask().clone()))
    }

    /// `Player::addUpgrade(.., UPGRADE_STATUS_IN_PRODUCTION)`.
    pub(super) fn add_upgrade_in_production(&mut self, team: TeamId, upgrade: NameKeyType) {
        if let (Some((name, mask)), Some(player_idx)) = (
            self.upgrade_name_and_mask(upgrade),
            self.controlling_player(team),
        ) {
            if let Some(player) = self.world.players.player_mut(player_idx) {
                player.add_upgrade(&name, &mask, UPGRADE_STATUS_IN_PRODUCTION);
            }
        }
    }

    /// `Player::removeUpgrade`.
    pub(super) fn remove_player_upgrade_of(&mut self, team: TeamId, upgrade: NameKeyType) {
        if let (Some((name, _)), Some(player_idx)) = (
            self.upgrade_name_and_mask(upgrade),
            self.controlling_player(team),
        ) {
            if let Some(player) = self.world.players.player_mut(player_idx) {
                player.remove_upgrade(&name);
            }
        }
    }

    /// The upgrade branch of `ProductionUpdate::update`, before the entry is removed.
    pub(super) fn complete_upgrade_of(&mut self, owner: ObjectSlot, upgrade: NameKeyType) {
        let Some((team, source)) = self.world.objects.get(owner).map(|o| (o.team_id, o.id)) else {
            return;
        };
        let Some(template) = self
            .rules
            .upgrade_center
            .find_upgrade_by_key(upgrade)
            .cloned()
        else {
            return;
        };
        let cost = template.calc_cost_to_build(None);
        if let Some(score_keeper) = self.score_keeper(team) {
            score_keeper.add_money_spent(cost);
        }
        let player = self.player_index_for_team(team);
        let name = template.upgrade_name().to_owned();
        self.world
            .completed_upgrades
            .push((player, name.clone(), source));
        if template.upgrade_type() == UpgradeType::Player {
            self.grant_player_upgrade(team, &name);
        } else {
            // `Object::giveUpgrade`.
            if let Some(obj) = self.world.objects.get_mut(owner) {
                let empty = UpgradeMaskType::new();
                obj.upgrades_completed
                    .clear_and_set(&empty, template.upgrade_mask());
            }
            self.update_upgrade_modules(owner);
        }
        if let Some(academy) = self.academy(team) {
            academy.record_upgrade(template.academy_classification_type(), FALSE);
        }
    }

    /// `ThingFactory::newObject`: a random build variation, then `initObject`'s upgrades.
    pub(super) fn factory_new_object_of(
        &mut self,
        tmpl: ThingTemplateId,
        team: TeamId,
    ) -> ObjectSlot {
        self.factory_new_object_with_status(tmpl, team, ObjectStatusMaskType::new())
    }

    /// `ThingFactory::newObject(tmpl, team, statusBits)`.
    pub fn factory_new_object_with_status(
        &mut self,
        mut tmpl: ThingTemplateId,
        team: TeamId,
        status: ObjectStatusMaskType,
    ) -> ObjectSlot {
        let variations = self
            .template_by_id(tmpl)
            .map_or_else(Vec::new, |t| t.get_build_variations().to_vec());
        if !variations.is_empty() {
            let which = self.rng_logic_int(0, variations.len() as Int - 1);
            let name = variations.get(which as usize);
            if let Some(id) = name.and_then(|n| self.rules.thing_factory.find_template_id(n)) {
                tmpl = id;
            }
        }
        let slot = self.spawn_object_with_status(tmpl, team, Coord3D::default(), 0.0, status);
        self.update_upgrade_modules(slot);
        slot
    }

    /// `ScoreKeeper::addObjectBuilt` and `addMoneySpent` of
    /// `Player::onStructureConstructionComplete`.
    pub(crate) fn record_structure_built(&mut self, owner: ObjectSlot) {
        let Some((team, under_construction)) = self.world.objects.get(owner).map(|o| {
            (
                o.team_id,
                o.status.test(ObjectStatusType::UnderConstruction.as_i32()),
            )
        }) else {
            return;
        };
        let Some(desc) = self.score_desc(owner) else {
            return;
        };
        let scoring = self.world.scoring_enabled;
        let player = self.player_index_for_team(team);
        if let Some(score_keeper) = self.score_keeper(team) {
            score_keeper.add_object_built(
            &ScoreObjectView {
                template: &desc,
                controlling_player_idx: player,
                under_construction,
            },
            scoring,
        );
        }
        let cost = self.unit_cost_for(team, ThingTemplateId(desc.id));
        if let Some(score_keeper) = self.score_keeper(team) {
            score_keeper.add_money_spent(cost);
        }
    }

    /// `Player::onStructureUndone`.
    pub(crate) fn record_structure_undone(&mut self, owner: ObjectSlot) {
        let Some((team, under_construction)) = self.world.objects.get(owner).map(|o| {
            (
                o.team_id,
                o.status.test(ObjectStatusType::UnderConstruction.as_i32()),
            )
        }) else {
            return;
        };
        let Some(desc) = self.score_desc(owner) else {
            return;
        };
        let scoring = self.world.scoring_enabled;
        let player = self.player_index_for_team(team);
        if let Some(score_keeper) = self.score_keeper(team) {
            score_keeper.remove_object_built(
            &ScoreObjectView {
                template: &desc,
                controlling_player_idx: player,
                under_construction,
            },
            scoring,
        );
        }
    }

    /// `Object::friend_adjustPowerForPlayer`, whatever the construction status.
    pub(crate) fn friend_adjust_power_for_player(&mut self, owner: ObjectSlot, incoming: Bool) {
        let Some(team) = self.world.objects.get(owner).map(|o| o.team_id) else {
            return;
        };
        let (Some(info), Some(player)) =
            (self.energy_object_info(owner), self.controlling_player(team))
        else {
            return;
        };
        if info.is_disabled != FALSE && info.energy_production > 0 {
            return;
        }
        self.change_energy(player, |energy, frame, told| {
            if incoming != FALSE {
                energy.object_entering_influence(Some(info), frame, told);
            } else {
                energy.object_leaving_influence(Some(info), frame, told);
            }
        });
    }

    /// `Player::onUnitCreated`. `factory` is `None` for starting network units
    /// (`GameLogic::placeNetworkBuildingsForPlayer`).
    pub(crate) fn unit_created(&mut self, factory: Option<ObjectSlot>, unit: ObjectSlot) {
        self.world.object_count_changed_frame = self.world.frame;
        let (team, factory_id) = match factory {
            Some(f) => match self.world.objects.get(f) {
                Some(o) => (o.team_id, o.id),
                None => return,
            },
            None => match self.world.objects.get(unit) {
                Some(o) => (o.team_id, zh_common::types::ObjectId::INVALID),
                None => return,
            },
        };
        let Some((unit_id, under_construction)) = self.world.objects.get(unit).map(|o| {
            (
                o.id,
                o.status.test(ObjectStatusType::UnderConstruction.as_i32()),
            )
        }) else {
            return;
        };
        let Some(desc) = self.score_desc(unit) else {
            return;
        };
        let scoring = self.world.scoring_enabled;
        let player = self.player_index_for_team(team);
        if let Some(score_keeper) = self.score_keeper(team) {
            score_keeper.add_object_built(
            &ScoreObjectView {
                template: &desc,
                controlling_player_idx: player,
                under_construction,
            },
            scoring,
        );
        }
        self.world.units_produced.push((factory_id, unit_id));
        if let Some(f) = factory {
            if self.world.ai_players.contains_key(&player) {
                crate::ai::ai_player::on_unit_produced_in_sim(self, player, f, unit);
            }
        }
    }

    /// `AcademyStats::recordProduction`.
    pub(super) fn record_production_of(&mut self, factory: ObjectSlot, unit: ObjectSlot) {
        use academy_kind_bits as bits;
        let Some(team) = self.world.objects.get(factory).map(|o| o.team_id) else {
            return;
        };
        let kinds = [
            (KindOfType::Dozer, bits::KINDOF_DOZER),
            (KindOfType::FsSupplyCenter, bits::KINDOF_FS_SUPPLY_CENTER),
            (KindOfType::Infantry, bits::KINDOF_INFANTRY),
            (KindOfType::Vehicle, bits::KINDOF_VEHICLE),
            (KindOfType::Harvester, bits::KINDOF_HARVESTER),
            (KindOfType::Hero, bits::KINDOF_HERO),
            (
                KindOfType::FsStrategyCenter,
                bits::KINDOF_FS_STRATEGY_CENTER,
            ),
            (KindOfType::MoneyHacker, bits::KINDOF_MONEY_HACKER),
            (KindOfType::FsBlackMarket, bits::KINDOF_FS_BLACK_MARKET),
            (
                KindOfType::FsSupplyDropzone,
                bits::KINDOF_FS_SUPPLY_DROPZONE,
            ),
            (KindOfType::FsBarracks, bits::KINDOF_FS_BARRACKS),
            (KindOfType::FsWarfactory, bits::KINDOF_FS_WARFACTORY),
            (KindOfType::FsAdvancedTech, bits::KINDOF_FS_ADVANCED_TECH),
            (KindOfType::Disguiser, bits::KINDOF_DISGUISER),
        ];
        let kind_mask = self.object_template(unit).map_or(0, |t| {
            kinds
                .iter()
                .filter(|(k, _)| t.is_kind_of(*k) != FALSE)
                .fold(0, |mask, (_, bit)| mask | (1u32 << bit))
        });
        let is_tunnel_contain = self
            .world
            .objects
            .get(unit)
            .and_then(|o| o.contain_slot)
            .and_then(|ms| self.world.modules.payload(ms))
            .and_then(|m| m.as_contain())
            .map_or(FALSE, |c| c.is_tunnel_contain());
        let frame = self.world.frame;
        let info = AcademyObjectInfo {
            kind_mask,
            is_tunnel_contain,
        };
        if let Some(academy) = self.academy(team) {
            academy.record_production(&info, frame);
        }
    }
}
