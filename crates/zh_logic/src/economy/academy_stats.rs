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
        todo!("PKT-116")
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
        todo!("PKT-116")
    }

    pub fn record_production(&mut self, obj: &AcademyObjectInfo, current_frame: UnsignedInt) {
        todo!("PKT-116")
    }

    pub fn record_upgrade(
        &mut self,
        classification: AcademyClassificationType,
        granted: Bool,
    ) {
        todo!("PKT-116")
    }

    pub fn record_special_power_used(&mut self, classification: AcademyClassificationType) {
        todo!("PKT-116")
    }

    pub fn record_income(&mut self, current_frame: UnsignedInt) {
        todo!("PKT-116")
    }

    #[inline]
    pub fn record_building_capture(&mut self) {
        self.structures_captured = self.structures_captured.wrapping_add(1);
    }

    #[inline]
    pub fn record_generals_points_spent(&mut self, points: Int) {
        self.generals_points_spent = self.generals_points_spent.wrapping_add(points as UnsignedInt);
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

    pub fn calculate_academy_advice(
        &mut self,
        info: &mut AcademyAdviceInfo,
        ctx: &AcademyPlayerContext<'_>,
        prng: &mut PrngState,
    ) -> Bool {
        let _ = (info, ctx, prng);
        todo!("PKT-116")
    }
}

impl Snapshot for AcademyStats {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let _ = xfer;
        todo!("PKT-116")
    }

    fn load_post_process(&mut self) {}
}
