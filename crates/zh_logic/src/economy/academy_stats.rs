//! `PKT-116`: Player academy statistics, advice evaluator & snapshot tracker
//! (`Core/GameEngine/Include/Common/AcademyStats.h`,
//! `Core/GameEngine/Source/Common/RTS/AcademyStats.cpp`).

#![allow(unused_variables, dead_code)]

use crate::rules::AcademyClassificationType;
use zh_common::random_value::PrngState;
use zh_common::types::{Bool, Int, UnsignedInt};
use zh_common::xfer::{Snapshot, Xfer, XferError};

pub const MAX_ADVICE_TIPS: usize = 1;
pub const FRAMES_BETWEEN_UPDATES: UnsignedInt = 30;
pub const DEFAULT_SUPPLY_CENTER_COST: UnsignedInt = 1000;

/// KindOf bit positions used by `AcademyStats` (`KindOfType` in C++).
pub mod academy_kind_bits {
    use zh_common::types::UnsignedInt;

    pub const KINDOF_DOZER: UnsignedInt = 1;
    pub const KINDOF_FS_SUPPLY_CENTER: UnsignedInt = 2;
    pub const KINDOF_COMMANDCENTER: UnsignedInt = 3;
    pub const KINDOF_INFANTRY: UnsignedInt = 4;
    pub const KINDOF_VEHICLE: UnsignedInt = 5;
    pub const KINDOF_HARVESTER: UnsignedInt = 6;
    pub const KINDOF_HERO: UnsignedInt = 7;
    pub const KINDOF_FS_STRATEGY_CENTER: UnsignedInt = 8;
    pub const KINDOF_MONEY_HACKER: UnsignedInt = 9;
    pub const KINDOF_FS_BLACK_MARKET: UnsignedInt = 10;
    pub const KINDOF_FS_SUPPLY_DROPZONE: UnsignedInt = 11;
    pub const KINDOF_FS_BARRACKS: UnsignedInt = 12;
    pub const KINDOF_FS_WARFACTORY: UnsignedInt = 13;
    pub const KINDOF_FS_ADVANCED_TECH: UnsignedInt = 14;
    pub const KINDOF_DISGUISER: UnsignedInt = 15;

    #[inline]
    pub const fn has_kind(mask: UnsignedInt, kind_bit: UnsignedInt) -> bool {
        (mask & (1u32 << kind_bit)) != 0
    }
}

/// Advice output buffer populated by [`AcademyStats::calculate_academy_advice`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AcademyAdviceInfo {
    pub advice: [String; MAX_ADVICE_TIPS],
    pub num_tips: UnsignedInt,
}

/// Lightweight object properties inspected by [`AcademyStats::update`] and [`AcademyStats::record_production`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AcademyObjectInfo {
    pub kind_mask: UnsignedInt,
    pub is_tunnel_contain: Bool,
}

/// Player & global context passed to [`AcademyStats`] methods in place of C++ global singletons.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AcademyPlayerContext<'a> {
    /// Player template base side (`Some("USA")`, `Some("China")`, `Some("GLA")`, etc., or `None` if no template).
    pub base_side: Option<&'a str>,
    /// Current `TheGameLogic->getFrame()`.
    pub current_frame: UnsignedInt,
    /// Player current cash (`m_player->getMoney()->countMoney()`).
    pub current_money: UnsignedInt,
    /// Player current power status (`m_player->getEnergy()->hasSufficientPower()`).
    pub has_sufficient_power: Bool,
    /// Objects owned by the player (`m_player->iterateObjects`).
    pub objects: &'a [AcademyObjectInfo],
    /// Optional supply center build cost if discovered via dozer command set (`None` defaults to `1000`).
    pub dozer_supply_center_cost: Option<UnsignedInt>,
    /// `TheGlobalData->m_useAlternateMouse`.
    pub use_alternate_mouse: Bool,
    /// `ThePlayerList->getLocalPlayer()->getAcademyStats()->getMines()`.
    pub local_player_mines: UnsignedInt,
    /// `ThePlayerList->getNeutralPlayer()->getAcademyStats()->getVehiclesSniped()`.
    pub neutral_player_vehicles_sniped: UnsignedInt,
}

/// Deterministic player gameplay statistics & advice evaluator (`PKT-116`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AcademyStats {
    pub next_update_frame: UnsignedInt,
    pub first_update: Bool,
    pub unknown_side: Bool,

    // Tier 1 (Basic advice)
    pub spent_cash_before_building_supply_center: Bool,
    pub supply_centers_built: UnsignedInt,
    pub supply_center_cost: UnsignedInt,
    pub researched_radar: Bool,
    pub peons_built: UnsignedInt,
    pub structures_captured: UnsignedInt,
    pub generals_points_spent: UnsignedInt,
    pub special_powers_used: UnsignedInt,
    pub structures_garrisoned: UnsignedInt,
    pub idle_building_units_max_frames: UnsignedInt,
    pub last_unit_built_frame: UnsignedInt,
    pub drag_select_units: UnsignedInt,
    pub upgrades_purchased: UnsignedInt,
    pub power_out_max_frames: UnsignedInt,
    pub oldest_power_out_frame: UnsignedInt,
    pub had_power_last_check: Bool,
    pub gatherers_built: UnsignedInt,
    pub heroes_built: UnsignedInt,

    // Tier 2 (Intermediate advice)
    pub had_a_strategy_center: Bool,
    pub chose_a_strategy_for_center: Bool,
    pub units_entered_tunnel_network: UnsignedInt,
    pub had_a_tunnel_network: Bool,
    pub control_groups_used: UnsignedInt,
    pub secondary_income_units_built: UnsignedInt,
    pub cleared_garrisoned_buildings: UnsignedInt,
    pub salvage_collected: UnsignedInt,
    pub guard_ability_used_count: UnsignedInt,

    // Tier 3 (Advanced advice)
    pub double_click_attack_move_orders_given: UnsignedInt,
    pub built_barracks_within_five_minutes: Bool,
    pub built_war_factory_within_ten_minutes: Bool,
    pub built_tech_structure_within_fifteen_minutes: Bool,
    pub last_income_frame: UnsignedInt,
    pub max_frames_between_income: UnsignedInt,
    pub mines: UnsignedInt,
    pub mines_cleared: UnsignedInt,
    pub vehicles_recovered: UnsignedInt,
    pub vehicles_sniped: UnsignedInt,
    pub disguisable_vehicles_built: UnsignedInt,
    pub vehicles_disguised: UnsignedInt,
    pub firestorms_created: UnsignedInt,
}

impl AcademyStats {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(&mut self, ctx: &AcademyPlayerContext<'_>) {
        self.next_update_frame = ctx.current_frame.wrapping_add(FRAMES_BETWEEN_UPDATES);
        self.first_update = zh_common::types::TRUE;
        self.unknown_side = if matches!(ctx.base_side, Some(side) if side.eq_ignore_ascii_case("USA")
            || side.eq_ignore_ascii_case("China")
            || side.eq_ignore_ascii_case("GLA"))
        {
            zh_common::types::FALSE
        } else {
            zh_common::types::TRUE
        };
        self.spent_cash_before_building_supply_center = zh_common::types::FALSE;
        self.supply_centers_built = 0;
        self.supply_center_cost = ctx
            .dozer_supply_center_cost
            .unwrap_or(DEFAULT_SUPPLY_CENTER_COST);
        self.researched_radar = zh_common::types::FALSE;
        self.had_power_last_check = zh_common::types::FALSE;
        self.had_a_strategy_center = zh_common::types::FALSE;
        self.chose_a_strategy_for_center = zh_common::types::FALSE;
        self.had_a_tunnel_network = zh_common::types::FALSE;
        self.built_barracks_within_five_minutes = zh_common::types::FALSE;
        self.built_war_factory_within_ten_minutes = zh_common::types::FALSE;
        self.built_tech_structure_within_fifteen_minutes = zh_common::types::FALSE;

        self.peons_built = 0;
        self.structures_captured = 0;
        self.generals_points_spent = 0;
        self.special_powers_used = 0;
        self.structures_garrisoned = 0;
        self.idle_building_units_max_frames = 0;
        self.last_unit_built_frame = 0;
        self.drag_select_units = 0;
        self.upgrades_purchased = 0;
        self.power_out_max_frames = 0;
        self.oldest_power_out_frame = 0;
        self.gatherers_built = 0;
        self.heroes_built = 0;
        self.units_entered_tunnel_network = 0;
        self.control_groups_used = 0;
        self.secondary_income_units_built = 0;
        self.cleared_garrisoned_buildings = 0;
        self.salvage_collected = 0;
        self.guard_ability_used_count = 0;
        self.double_click_attack_move_orders_given = 0;
        self.last_income_frame = 0;
        self.max_frames_between_income = 0;
        self.mines = 0;
        self.mines_cleared = 0;
        self.vehicles_recovered = 0;
        self.vehicles_sniped = 0;
        self.disguisable_vehicles_built = 0;
        self.vehicles_disguised = 0;
        self.firestorms_created = 0;
    }

    #[inline]
    pub fn is_first_update(&self) -> Bool {
        self.first_update
    }

    #[inline]
    pub fn set_first_update(&mut self, set: Bool) {
        self.first_update = set;
    }

    pub fn update(&mut self, ctx: &AcademyPlayerContext<'_>) {
        if self.unknown_side != zh_common::types::FALSE {
            return;
        }

        let now = ctx.current_frame;
        if self.next_update_frame >= now {
            self.next_update_frame = now.wrapping_add(FRAMES_BETWEEN_UPDATES);
            if self.is_first_update() != zh_common::types::FALSE {
                for obj in ctx.objects {
                    self.record_production(obj, now);
                }
            }

            if self.supply_centers_built == 0
                && self.spent_cash_before_building_supply_center == zh_common::types::FALSE
                && ctx.current_money < self.supply_center_cost
            {
                self.spent_cash_before_building_supply_center = zh_common::types::TRUE;
            }

            let has_power = ctx.has_sufficient_power;
            if has_power != self.had_power_last_check {
                if has_power == zh_common::types::FALSE {
                    self.oldest_power_out_frame = now;
                } else {
                    let frames = now.wrapping_sub(self.oldest_power_out_frame);
                    if frames > self.power_out_max_frames {
                        self.power_out_max_frames = frames;
                    }
                }
                self.had_power_last_check = has_power;
            }

            if self.is_first_update() != zh_common::types::FALSE {
                self.set_first_update(zh_common::types::FALSE);
            }
        }
    }

    pub fn record_production(&mut self, obj: &AcademyObjectInfo, current_frame: UnsignedInt) {
        use academy_kind_bits::*;

        if has_kind(obj.kind_mask, KINDOF_FS_SUPPLY_CENTER) {
            self.supply_centers_built = self.supply_centers_built.wrapping_add(1);
        }
        if has_kind(obj.kind_mask, KINDOF_DOZER) {
            self.peons_built = self.peons_built.wrapping_add(1);
        }
        if (has_kind(obj.kind_mask, KINDOF_INFANTRY) || has_kind(obj.kind_mask, KINDOF_VEHICLE))
            && !has_kind(obj.kind_mask, KINDOF_DOZER)
            && !has_kind(obj.kind_mask, KINDOF_HARVESTER)
        {
            let idle = current_frame.wrapping_sub(self.last_unit_built_frame);
            if idle > self.idle_building_units_max_frames {
                self.idle_building_units_max_frames = idle;
            }
            self.last_unit_built_frame = current_frame;
        }
        if has_kind(obj.kind_mask, KINDOF_HARVESTER) {
            self.gatherers_built = self.gatherers_built.wrapping_add(1);
        }
        if has_kind(obj.kind_mask, KINDOF_HERO) {
            self.heroes_built = self.heroes_built.wrapping_add(1);
        }
        if has_kind(obj.kind_mask, KINDOF_FS_STRATEGY_CENTER) {
            self.had_a_strategy_center = zh_common::types::TRUE;
        }
        if obj.is_tunnel_contain != zh_common::types::FALSE {
            self.had_a_tunnel_network = zh_common::types::TRUE;
        }
        if has_kind(obj.kind_mask, KINDOF_MONEY_HACKER)
            || has_kind(obj.kind_mask, KINDOF_FS_BLACK_MARKET)
            || has_kind(obj.kind_mask, KINDOF_FS_SUPPLY_DROPZONE)
        {
            self.secondary_income_units_built = self.secondary_income_units_built.wrapping_add(1);
        }
        if has_kind(obj.kind_mask, KINDOF_FS_BARRACKS) && current_frame <= 9000 {
            self.built_barracks_within_five_minutes = zh_common::types::TRUE;
        }
        if has_kind(obj.kind_mask, KINDOF_FS_WARFACTORY) && current_frame <= 18000 {
            self.built_war_factory_within_ten_minutes = zh_common::types::TRUE;
        }
        if has_kind(obj.kind_mask, KINDOF_FS_ADVANCED_TECH) && current_frame <= 27000 {
            self.built_tech_structure_within_fifteen_minutes = zh_common::types::TRUE;
        }
        if has_kind(obj.kind_mask, KINDOF_DISGUISER) {
            self.disguisable_vehicles_built = self.disguisable_vehicles_built.wrapping_add(1);
        }
    }

    pub fn record_upgrade(&mut self, classification: AcademyClassificationType, granted: Bool) {
        if classification == AcademyClassificationType::UpgradeRadar {
            self.researched_radar = zh_common::types::TRUE;
        }
        if granted == zh_common::types::FALSE {
            self.upgrades_purchased = self.upgrades_purchased.wrapping_add(1);
        }
    }

    pub fn record_special_power_used(&mut self, classification: AcademyClassificationType) {
        if classification == AcademyClassificationType::Superpower {
            self.special_powers_used = self.special_powers_used.wrapping_add(1);
        }
    }

    pub fn record_income(&mut self, current_frame: UnsignedInt) {
        let delta = self.last_income_frame.wrapping_sub(current_frame);
        if delta > self.max_frames_between_income {
            self.max_frames_between_income = delta;
        }
        self.last_income_frame = current_frame;
    }

    #[inline]
    pub fn record_building_capture(&mut self) {
        self.structures_captured = self.structures_captured.wrapping_add(1);
    }

    #[inline]
    pub fn record_generals_points_spent(&mut self, points: Int) {
        self.generals_points_spent = self
            .generals_points_spent
            .wrapping_add(points as UnsignedInt);
    }

    #[inline]
    pub fn record_building_garrisoned(&mut self) {
        self.structures_garrisoned = self.structures_garrisoned.wrapping_add(1);
    }

    #[inline]
    pub fn record_drag_selection(&mut self) {
        self.drag_select_units = self.drag_select_units.wrapping_add(1);
    }

    #[inline]
    pub fn record_strategy_center(&mut self) {
        self.had_a_strategy_center = zh_common::types::TRUE;
    }

    #[inline]
    pub fn record_battle_plan_selected(&mut self) {
        self.chose_a_strategy_for_center = zh_common::types::TRUE;
    }

    #[inline]
    pub fn record_unit_entered_tunnel_network(&mut self) {
        self.units_entered_tunnel_network = self.units_entered_tunnel_network.wrapping_add(1);
    }

    #[inline]
    pub fn record_control_groups_used(&mut self) {
        self.control_groups_used = self.control_groups_used.wrapping_add(1);
    }

    #[inline]
    pub fn record_cleared_garrisoned_building(&mut self) {
        self.cleared_garrisoned_buildings = self.cleared_garrisoned_buildings.wrapping_add(1);
    }

    #[inline]
    pub fn record_vehicle_disguised(&mut self) {
        self.vehicles_disguised = self.vehicles_disguised.wrapping_add(1);
    }

    #[inline]
    pub fn record_firestorm_created(&mut self) {
        self.firestorms_created = self.firestorms_created.wrapping_add(1);
    }

    #[inline]
    pub fn record_guard_ability_used(&mut self) {
        self.guard_ability_used_count = self.guard_ability_used_count.wrapping_add(1);
    }

    #[inline]
    pub fn record_salvage_collected(&mut self) {
        self.salvage_collected = self.salvage_collected.wrapping_add(1);
    }

    #[inline]
    pub fn record_double_click_attack_move_order_given(&mut self) {
        self.double_click_attack_move_orders_given =
            self.double_click_attack_move_orders_given.wrapping_add(1);
    }

    #[inline]
    pub fn record_mine_cleared(&mut self) {
        self.mines_cleared = self.mines_cleared.wrapping_add(1);
    }

    #[inline]
    pub fn record_vehicle_sniped(&mut self) {
        self.vehicles_sniped = self.vehicles_sniped.wrapping_add(1);
    }

    #[inline]
    pub fn get_vehicles_sniped(&self) -> UnsignedInt {
        self.vehicles_sniped
    }

    #[inline]
    pub fn record_mine(&mut self) {
        self.mines = self.mines.wrapping_add(1);
    }

    #[inline]
    pub fn get_mines(&self) -> UnsignedInt {
        self.mines
    }

    #[inline]
    pub fn had_a_supply_center(&self) -> Bool {
        Bool::from(self.supply_centers_built > 0)
    }

    fn evaluate_tier1_advice(
        &mut self,
        info: &mut AcademyAdviceInfo,
        ctx: &AcademyPlayerContext<'_>,
        prng: &mut PrngState,
        num_available_tips: Int,
    ) {
        let choosing = num_available_tips != -1;
        let mut num_available_tips = num_available_tips;
        let mut available_tips: Int = 0;

        macro_rules! tip {
            ($matched:expr, $key:literal) => {{
                if $matched {
                    available_tips = available_tips.wrapping_add(1);
                    let hi = num_available_tips.wrapping_sub(1);
                    let rand = prng.get_game_client_random_value(0, hi);
                    let limit = (MAX_ADVICE_TIPS as Int).wrapping_sub(info.num_tips as Int);
                    if choosing && rand < limit {
                        if info.num_tips < MAX_ADVICE_TIPS as UnsignedInt {
                            info.advice[info.num_tips as usize].push_str($key);
                            info.num_tips = info.num_tips.wrapping_add(1);
                        }
                    }
                    num_available_tips = num_available_tips.wrapping_sub(1);
                }
            }};
        }

        let now = ctx.current_frame;
        tip!(
            self.spent_cash_before_building_supply_center != 0,
            "ACADEMY:BuildSupplyCenterEarlier"
        );
        tip!(self.researched_radar == 0, "ACADEMY:TryBuildingRadar");
        tip!(self.peons_built < 2, "ACADEMY:BuildMorePeons");
        tip!(
            self.structures_captured == 0,
            "ACADEMY:TryCapturingStructures"
        );
        tip!(
            self.generals_points_spent == 0,
            "ACADEMY:SpendGeneralsPoints"
        );
        tip!(
            self.special_powers_used == 0,
            "ACADEMY:TryUsingSuperweapons"
        );
        tip!(
            self.structures_garrisoned == 0,
            "ACADEMY:TryGarrisoningAStructure"
        );

        let idle_frames = now.wrapping_sub(self.last_unit_built_frame);
        if idle_frames > self.idle_building_units_max_frames {
            self.idle_building_units_max_frames = idle_frames;
        }
        tip!(
            self.idle_building_units_max_frames > 300 * 30 || self.last_unit_built_frame == 0,
            "ACADEMY:IdleBuildingUnits"
        );
        tip!(self.drag_select_units == 0, "ACADEMY:TryDragSelectingUnits");
        tip!(self.upgrades_purchased == 0, "ACADEMY:ResearchUpgrades");

        if self.had_power_last_check == zh_common::types::FALSE {
            let frames = now.wrapping_sub(self.oldest_power_out_frame);
            if frames > self.power_out_max_frames {
                self.power_out_max_frames = frames;
            }
        }
        tip!(
            self.power_out_max_frames > 600 * 30,
            "ACADEMY:RanOutOfPower"
        );
        tip!(self.gatherers_built == 0, "ACADEMY:BuildMoreGatherers");
        tip!(self.heroes_built == 0, "ACADEMY:BuildAHero");
        let _ = num_available_tips;

        if !choosing && available_tips > 0 {
            self.evaluate_tier1_advice(info, ctx, prng, available_tips);
        }
    }

    fn evaluate_tier2_advice(
        &mut self,
        info: &mut AcademyAdviceInfo,
        ctx: &AcademyPlayerContext<'_>,
        prng: &mut PrngState,
        num_available_tips: Int,
    ) {
        let choosing = num_available_tips != -1;
        let mut num_available_tips = num_available_tips;
        let mut available_tips: Int = 0;

        macro_rules! tip {
            ($matched:expr, $key:literal) => {{
                if $matched {
                    available_tips = available_tips.wrapping_add(1);
                    let hi = num_available_tips.wrapping_sub(1);
                    let rand = prng.get_game_client_random_value(0, hi);
                    let limit = (MAX_ADVICE_TIPS as Int).wrapping_sub(info.num_tips as Int);
                    if choosing && rand < limit {
                        if info.num_tips < MAX_ADVICE_TIPS as UnsignedInt {
                            info.advice[info.num_tips as usize].push_str($key);
                            info.num_tips = info.num_tips.wrapping_add(1);
                        }
                    }
                    num_available_tips = num_available_tips.wrapping_sub(1);
                }
            }};
        }

        tip!(
            self.had_a_strategy_center != 0 && self.chose_a_strategy_for_center == 0,
            "ACADEMY:PickStrategyCenterPlan"
        );
        tip!(
            self.had_a_tunnel_network != 0 && self.units_entered_tunnel_network == 0,
            "ACADEMY:UseTunnelNetwork"
        );
        tip!(self.control_groups_used == 0, "ACADEMY:UseControlGroups");
        tip!(
            self.secondary_income_units_built == 0,
            "ACADEMY:UseSecondaryIncomeMethods"
        );
        tip!(
            self.cleared_garrisoned_buildings == 0,
            "ACADEMY:ClearBuildings"
        );
        tip!(
            ctx.base_side
                .is_some_and(|side| side.eq_ignore_ascii_case("GLA"))
                && self.salvage_collected == 0,
            "ACADEMY:PickUpSalvage"
        );
        tip!(
            self.guard_ability_used_count == 0,
            "ACADEMY:UseGuardAbility"
        );
        tip!(
            self.supply_centers_built < 2,
            "ACADEMY:MultipleSupplyCenters"
        );
        let _ = num_available_tips;

        if !choosing && available_tips > 0 {
            self.evaluate_tier2_advice(info, ctx, prng, available_tips);
        }
    }

    fn evaluate_tier3_advice(
        &mut self,
        info: &mut AcademyAdviceInfo,
        ctx: &AcademyPlayerContext<'_>,
        prng: &mut PrngState,
        num_available_tips: Int,
    ) {
        let choosing = num_available_tips != -1;
        let mut num_available_tips = num_available_tips;
        let mut available_tips: Int = 0;

        macro_rules! tip {
            ($matched:expr, $key:literal) => {{
                if $matched {
                    available_tips = available_tips.wrapping_add(1);
                    let hi = num_available_tips.wrapping_sub(1);
                    let rand = prng.get_game_client_random_value(0, hi);
                    let limit = (MAX_ADVICE_TIPS as Int).wrapping_sub(info.num_tips as Int);
                    if choosing && rand < limit {
                        if info.num_tips < MAX_ADVICE_TIPS as UnsignedInt {
                            info.advice[info.num_tips as usize].push_str($key);
                            info.num_tips = info.num_tips.wrapping_add(1);
                        }
                    }
                    num_available_tips = num_available_tips.wrapping_sub(1);
                }
            }};
        }

        let now = ctx.current_frame;
        tip!(
            ctx.use_alternate_mouse == zh_common::types::FALSE,
            "ACADEMY:AlternateMouseInterface"
        );
        tip!(
            self.double_click_attack_move_orders_given == 0,
            "ACADEMY:DoubleClickAttackMoveGuard"
        );
        tip!(
            self.built_barracks_within_five_minutes == 0,
            "ACADEMY:BuildBarracksSooner"
        );
        tip!(
            self.built_war_factory_within_ten_minutes == 0,
            "ACADEMY:BuildWarFactorySooner"
        );
        tip!(
            self.built_tech_structure_within_fifteen_minutes == 0,
            "ACADEMY:BuildTechStructureSooner"
        );

        let delta = self.last_income_frame.wrapping_sub(now);
        if delta > self.max_frames_between_income {
            self.max_frames_between_income = delta;
        }
        tip!(
            self.max_frames_between_income > 30 * 120 || self.last_income_frame == 0,
            "ACADEMY:NoIncome"
        );
        tip!(
            ctx.local_player_mines > 0 && self.mines_cleared == 0,
            "ACADEMY:NoIncome"
        );
        tip!(
            ctx.neutral_player_vehicles_sniped > 0 && self.vehicles_recovered == 0,
            "ACADEMY:UnmannedVehicles"
        );
        tip!(
            self.disguisable_vehicles_built > 0 && self.vehicles_disguised == 0,
            "ACADEMY:DisguisedUnits"
        );
        tip!(
            ctx.base_side
                .is_some_and(|side| side.eq_ignore_ascii_case("China"))
                && self.firestorms_created == 0,
            "ACADEMY:Firestorm"
        );
        let _ = num_available_tips;

        if !choosing && available_tips > 0 {
            self.evaluate_tier3_advice(info, ctx, prng, available_tips);
        }
    }

    pub fn calculate_academy_advice(
        &mut self,
        info: &mut AcademyAdviceInfo,
        ctx: &AcademyPlayerContext<'_>,
        prng: &mut PrngState,
    ) -> Bool {
        if self.unknown_side != zh_common::types::FALSE {
            return zh_common::types::FALSE;
        }

        info.num_tips = 0;
        info.advice[0] = "\n\n".to_string();
        self.evaluate_tier1_advice(info, ctx, prng, -1);
        if info.num_tips < MAX_ADVICE_TIPS as UnsignedInt {
            self.evaluate_tier2_advice(info, ctx, prng, -1);
        }
        if info.num_tips < MAX_ADVICE_TIPS as UnsignedInt {
            self.evaluate_tier3_advice(info, ctx, prng, -1);
        }

        Bool::from(info.num_tips != 0)
    }
}

impl Snapshot for AcademyStats {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        use zh_common::types::XferVersion;

        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        xfer.xfer_unsigned_int(&mut self.next_update_frame)?;
        xfer.xfer_bool(&mut self.first_update)?;
        xfer.xfer_bool(&mut self.unknown_side)?;
        xfer.xfer_bool(&mut self.spent_cash_before_building_supply_center)?;
        xfer.xfer_unsigned_int(&mut self.supply_centers_built)?;
        xfer.xfer_unsigned_int(&mut self.supply_center_cost)?;
        xfer.xfer_bool(&mut self.researched_radar)?;
        xfer.xfer_unsigned_int(&mut self.peons_built)?;
        xfer.xfer_unsigned_int(&mut self.structures_captured)?;
        xfer.xfer_unsigned_int(&mut self.generals_points_spent)?;
        xfer.xfer_unsigned_int(&mut self.special_powers_used)?;
        xfer.xfer_unsigned_int(&mut self.structures_garrisoned)?;
        xfer.xfer_unsigned_int(&mut self.idle_building_units_max_frames)?;
        xfer.xfer_unsigned_int(&mut self.last_unit_built_frame)?;
        xfer.xfer_unsigned_int(&mut self.drag_select_units)?;
        xfer.xfer_unsigned_int(&mut self.upgrades_purchased)?;
        xfer.xfer_unsigned_int(&mut self.power_out_max_frames)?;
        xfer.xfer_unsigned_int(&mut self.oldest_power_out_frame)?;
        xfer.xfer_bool(&mut self.had_power_last_check)?;
        xfer.xfer_unsigned_int(&mut self.gatherers_built)?;
        xfer.xfer_unsigned_int(&mut self.heroes_built)?;
        xfer.xfer_bool(&mut self.had_a_strategy_center)?;
        xfer.xfer_bool(&mut self.chose_a_strategy_for_center)?;
        xfer.xfer_unsigned_int(&mut self.units_entered_tunnel_network)?;
        xfer.xfer_bool(&mut self.had_a_tunnel_network)?;
        xfer.xfer_unsigned_int(&mut self.control_groups_used)?;
        xfer.xfer_unsigned_int(&mut self.secondary_income_units_built)?;
        xfer.xfer_unsigned_int(&mut self.cleared_garrisoned_buildings)?;
        xfer.xfer_unsigned_int(&mut self.salvage_collected)?;
        xfer.xfer_unsigned_int(&mut self.guard_ability_used_count)?;
        xfer.xfer_unsigned_int(&mut self.double_click_attack_move_orders_given)?;
        xfer.xfer_bool(&mut self.built_barracks_within_five_minutes)?;
        xfer.xfer_bool(&mut self.built_war_factory_within_ten_minutes)?;
        xfer.xfer_bool(&mut self.built_tech_structure_within_fifteen_minutes)?;
        xfer.xfer_unsigned_int(&mut self.last_income_frame)?;
        xfer.xfer_unsigned_int(&mut self.max_frames_between_income)?;
        xfer.xfer_unsigned_int(&mut self.mines)?;
        xfer.xfer_unsigned_int(&mut self.mines_cleared)?;
        xfer.xfer_unsigned_int(&mut self.vehicles_recovered)?;
        xfer.xfer_unsigned_int(&mut self.vehicles_sniped)?;
        xfer.xfer_unsigned_int(&mut self.disguisable_vehicles_built)?;
        xfer.xfer_unsigned_int(&mut self.vehicles_disguised)?;
        xfer.xfer_unsigned_int(&mut self.firestorms_created)?;
        Ok(())
    }

    fn load_post_process(&mut self) {}
}
