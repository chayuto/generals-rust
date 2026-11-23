//! `PKT-122`: Per-player score and build/destroy/capture/loss counters
//! (`GeneralsMD/Code/GameEngine/Source/Common/RTS/ScoreKeeper.cpp`).

use std::collections::BTreeMap;

use super::MAX_PLAYER_COUNT;
use crate::rules::kind_of::{KindOfMaskType, KindOfType};
use zh_common::types::{Bool, Int, UnsignedInt, UnsignedShort, XferVersion, FALSE};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

/// Lightweight template descriptor used by [`ScoreKeeper`] to evaluate KindOf
/// masks, equivalence, and deterministic template ordering without raw pointers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreTemplateDesc {
    pub id: UnsignedInt,
    pub name: String,
    pub kind_of: KindOfMaskType,
    pub equivalent_root_id: UnsignedInt,
}

impl ScoreTemplateDesc {
    pub fn is_kind_of(&self, k: KindOfType) -> bool {
        let idx = k as Int;
        idx >= 0 && self.kind_of.test(idx) != FALSE
    }

    pub fn is_kind_of_multi(
        &self,
        must_be_set: &KindOfMaskType,
        must_be_clear: &KindOfMaskType,
    ) -> bool {
        self.kind_of.test_for_all(must_be_set) != FALSE
            && self.kind_of.test_for_none(must_be_clear) != FALSE
    }

    pub fn is_equivalent_to(&self, other: &Self) -> bool {
        self.equivalent_root_id == other.equivalent_root_id
    }
}

/// Object snapshot view passed to [`ScoreKeeper`] mutation methods.
#[derive(Debug, Clone)]
pub struct ScoreObjectView<'a> {
    pub template: &'a ScoreTemplateDesc,
    pub controlling_player_idx: Int,
    pub under_construction: Bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreCountEntry {
    pub name: String,
    pub kind_of: KindOfMaskType,
    pub equivalent_root_id: UnsignedInt,
    pub count: Int,
}

pub type ObjectCountMap = BTreeMap<UnsignedInt, ScoreCountEntry>;

/// Per-player match score and object accounting (`ScoreKeeper.h/.cpp`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreKeeper {
    pub total_money_earned: Int,
    pub total_money_spent: Int,
    pub total_units_destroyed: [Int; MAX_PLAYER_COUNT],
    pub total_units_built: Int,
    pub total_units_lost: Int,
    pub total_buildings_destroyed: [Int; MAX_PLAYER_COUNT],
    pub total_buildings_built: Int,
    pub total_buildings_lost: Int,
    pub total_tech_buildings_captured: Int,
    pub total_faction_buildings_captured: Int,
    pub current_score: Int,
    pub my_player_idx: Int,
    pub objects_built: ObjectCountMap,
    pub objects_destroyed: [ObjectCountMap; MAX_PLAYER_COUNT],
    pub objects_lost: ObjectCountMap,
    pub objects_captured: ObjectCountMap,
}

impl Default for ScoreKeeper {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoreKeeper {
    pub fn new() -> Self {
        let mut out = Self {
            total_money_earned: 0,
            total_money_spent: 0,
            total_units_destroyed: [0; MAX_PLAYER_COUNT],
            total_units_built: 0,
            total_units_lost: 0,
            total_buildings_destroyed: [0; MAX_PLAYER_COUNT],
            total_buildings_built: 0,
            total_buildings_lost: 0,
            total_tech_buildings_captured: 0,
            total_faction_buildings_captured: 0,
            current_score: 0,
            my_player_idx: 0,
            objects_built: BTreeMap::new(),
            objects_destroyed: std::array::from_fn(|_| BTreeMap::new()),
            objects_lost: BTreeMap::new(),
            objects_captured: BTreeMap::new(),
        };
        out.reset(0);
        out
    }

    pub fn reset(&mut self, player_idx: Int) {
        self.total_money_earned = 0;
        self.total_money_spent = 0;
        self.total_units_lost = 0;
        self.total_units_built = 0;
        self.total_buildings_lost = 0;
        self.total_buildings_built = 0;
        self.total_faction_buildings_captured = 0;
        self.total_tech_buildings_captured = 0;
        self.current_score = 0;
        self.objects_built.clear();
        self.objects_captured.clear();
        self.objects_lost.clear();
        for i in 0..MAX_PLAYER_COUNT {
            self.objects_destroyed[i].clear();
            self.total_buildings_destroyed[i] = 0;
            self.total_units_destroyed[i] = 0;
        }
        self.my_player_idx = player_idx;
    }

    #[inline]
    pub fn add_money_spent(&mut self, money: Int) {
        self.total_money_spent = self.total_money_spent.wrapping_add(money);
    }

    #[inline]
    pub fn add_money_earned(&mut self, money: Int) {
        self.total_money_earned = self.total_money_earned.wrapping_add(money);
    }

    pub fn add_object_built(&mut self, obj: &ScoreObjectView<'_>, scoring_enabled: Bool) {
        if scoring_enabled == FALSE {
            return;
        }
        let mut add_to_count = false;
        let tmpl = obj.template;
        if tmpl.is_kind_of_multi(&scoring_building_mask(), &KindOfMaskType::default())
            || tmpl.is_kind_of_multi(&scoring_building_create_mask(), &KindOfMaskType::default())
        {
            self.total_buildings_built = self.total_buildings_built.wrapping_add(1);
            add_to_count = true;
        } else if (tmpl.is_kind_of(KindOfType::Infantry) || tmpl.is_kind_of(KindOfType::Vehicle))
            && (tmpl.is_kind_of(KindOfType::Score) || tmpl.is_kind_of(KindOfType::ScoreCreate))
        {
            self.total_units_built = self.total_units_built.wrapping_add(1);
            add_to_count = true;
        }

        if add_to_count {
            adjust_count_map(&mut self.objects_built, tmpl, 1);
        }
    }

    pub fn remove_object_built(&mut self, obj: &ScoreObjectView<'_>, scoring_enabled: Bool) {
        if scoring_enabled == FALSE {
            return;
        }
        let mut remove_from_count = false;
        let tmpl = obj.template;
        if tmpl.is_kind_of_multi(&scoring_building_mask(), &KindOfMaskType::default())
            || tmpl.is_kind_of_multi(&scoring_building_create_mask(), &KindOfMaskType::default())
        {
            self.total_buildings_built = self.total_buildings_built.wrapping_sub(1);
            remove_from_count = true;
        } else if (tmpl.is_kind_of(KindOfType::Infantry) || tmpl.is_kind_of(KindOfType::Vehicle))
            && (tmpl.is_kind_of(KindOfType::Score) || tmpl.is_kind_of(KindOfType::ScoreCreate))
        {
            self.total_units_built = self.total_units_built.wrapping_sub(1);
            remove_from_count = true;
        }

        if remove_from_count {
            adjust_count_map(&mut self.objects_built, tmpl, -1);
        }
    }

    pub fn add_object_captured(&mut self, obj: &ScoreObjectView<'_>, scoring_enabled: Bool) {
        if scoring_enabled == FALSE {
            return;
        }
        let mut add_to_count = false;
        let tmpl = obj.template;
        if tmpl.is_kind_of(KindOfType::Structure) {
            if tmpl.is_kind_of(KindOfType::Score) {
                self.total_faction_buildings_captured =
                    self.total_faction_buildings_captured.wrapping_add(1);
            } else {
                self.total_tech_buildings_captured =
                    self.total_tech_buildings_captured.wrapping_add(1);
            }
            add_to_count = true;
        }

        if add_to_count {
            adjust_count_map(&mut self.objects_captured, tmpl, 1);
        }
    }

    pub fn add_object_destroyed(&mut self, obj: &ScoreObjectView<'_>, scoring_enabled: Bool) {
        if scoring_enabled == FALSE {
            return;
        }
        let player_idx = obj.controlling_player_idx;
        if player_idx < 0 || (player_idx as usize) >= MAX_PLAYER_COUNT {
            return;
        }
        let pidx = player_idx as usize;
        let tmpl = obj.template;
        let under_construction = obj.under_construction != FALSE;

        let mut add_to_count = false;
        if tmpl.is_kind_of_multi(&scoring_building_mask(), &KindOfMaskType::default())
            || tmpl.is_kind_of_multi(&scoring_building_destroy_mask(), &KindOfMaskType::default())
        {
            if !under_construction {
                self.total_buildings_destroyed[pidx] =
                    self.total_buildings_destroyed[pidx].wrapping_add(1);
                add_to_count = true;
            }
        } else if (tmpl.is_kind_of(KindOfType::Infantry) || tmpl.is_kind_of(KindOfType::Vehicle))
            && (tmpl.is_kind_of(KindOfType::Score) || tmpl.is_kind_of(KindOfType::ScoreDestroy))
            && !under_construction
        {
            self.total_units_destroyed[pidx] = self.total_units_destroyed[pidx].wrapping_add(1);
            add_to_count = true;
        }

        if add_to_count {
            adjust_count_map(&mut self.objects_destroyed[pidx], tmpl, 1);
        }
    }

    pub fn add_object_lost(&mut self, obj: &ScoreObjectView<'_>, scoring_enabled: Bool) {
        if scoring_enabled == FALSE {
            return;
        }
        let tmpl = obj.template;
        let under_construction = obj.under_construction != FALSE;

        let mut add_to_count = false;
        if tmpl.is_kind_of_multi(&scoring_building_mask(), &KindOfMaskType::default())
            || tmpl.is_kind_of_multi(&scoring_building_destroy_mask(), &KindOfMaskType::default())
        {
            if !under_construction {
                self.total_buildings_lost = self.total_buildings_lost.wrapping_add(1);
                add_to_count = true;
            }
        } else if (tmpl.is_kind_of(KindOfType::Infantry) || tmpl.is_kind_of(KindOfType::Vehicle))
            && (tmpl.is_kind_of(KindOfType::Score) || tmpl.is_kind_of(KindOfType::ScoreDestroy))
            && !under_construction
        {
            self.total_units_lost = self.total_units_lost.wrapping_add(1);
            add_to_count = true;
        }

        if add_to_count {
            adjust_count_map(&mut self.objects_lost, tmpl, 1);
        }
    }

    pub fn calculate_score(&mut self) -> Int {
        let mut score: Int = 0;
        score = score.wrapping_add(self.total_units_built.wrapping_mul(100));
        score = score.wrapping_add(self.total_money_earned);
        score = score.wrapping_add(self.total_buildings_built.wrapping_mul(100));
        for i in 0..MAX_PLAYER_COUNT {
            if (i as Int) == self.my_player_idx {
                continue;
            }
            score = score.wrapping_add(self.total_units_destroyed[i].wrapping_mul(100));
            score = score.wrapping_add(self.total_buildings_destroyed[i].wrapping_mul(100));
        }
        self.current_score = score;
        self.current_score
    }

    pub fn get_total_units_built_masked(
        &self,
        valid_mask: &KindOfMaskType,
        invalid_mask: &KindOfMaskType,
    ) -> Int {
        let mut count: Int = 0;
        for entry in self.objects_built.values() {
            if entry.kind_of.test_for_all(valid_mask) != FALSE
                && entry.kind_of.test_for_none(invalid_mask) != FALSE
            {
                count = count.wrapping_add(entry.count);
            }
        }
        count
    }

    /// Note: C++ `ScoreKeeper::getTotalObjectsBuilt(const ThingTemplate *)` increments `++count`
    /// per matching map entry rather than summing `it->second`.
    pub fn get_total_objects_built(&self, tmpl: &ScoreTemplateDesc) -> Int {
        let mut count: Int = 0;
        for entry in self.objects_built.values() {
            if entry.equivalent_root_id == tmpl.equivalent_root_id {
                count = count.wrapping_add(1);
            }
        }
        count
    }

    pub fn get_total_units_destroyed(&self) -> Int {
        let mut count: Int = 0;
        for i in 0..MAX_PLAYER_COUNT {
            count = count.wrapping_add(self.total_units_destroyed[i]);
        }
        count
    }

    pub fn get_total_buildings_destroyed(&self) -> Int {
        let mut count: Int = 0;
        for i in 0..MAX_PLAYER_COUNT {
            count = count.wrapping_add(self.total_buildings_destroyed[i]);
        }
        count
    }

    /// `Xfer` with an explicit template name resolver (matching C++ `TheThingFactory->findTemplate`).
    pub fn xfer_with_resolver<F>(
        &mut self,
        xfer: &mut dyn Xfer,
        mut resolve: F,
    ) -> Result<(), XferError>
    where
        F: FnMut(&str) -> Option<ScoreTemplateDesc>,
    {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;

        xfer.xfer_int(&mut self.total_money_earned)?;
        xfer.xfer_int(&mut self.total_money_spent)?;
        xfer_i32_array(xfer, &mut self.total_units_destroyed)?;
        xfer.xfer_int(&mut self.total_units_built)?;
        xfer.xfer_int(&mut self.total_units_lost)?;
        xfer_i32_array(xfer, &mut self.total_buildings_destroyed)?;
        xfer.xfer_int(&mut self.total_buildings_built)?;
        xfer.xfer_int(&mut self.total_buildings_lost)?;
        xfer.xfer_int(&mut self.total_tech_buildings_captured)?;
        xfer.xfer_int(&mut self.total_faction_buildings_captured)?;
        xfer.xfer_int(&mut self.current_score)?;
        xfer.xfer_int(&mut self.my_player_idx)?;

        xfer_object_count_map(xfer, &mut self.objects_built, &mut resolve)?;

        let mut destroyed_array_size: UnsignedShort = MAX_PLAYER_COUNT as UnsignedShort;
        xfer.xfer_unsigned_short(&mut destroyed_array_size)?;
        if destroyed_array_size as usize != MAX_PLAYER_COUNT {
            return Err(XferError::InvalidData);
        }
        for i in 0..MAX_PLAYER_COUNT {
            xfer_object_count_map(xfer, &mut self.objects_destroyed[i], &mut resolve)?;
        }

        xfer_object_count_map(xfer, &mut self.objects_lost, &mut resolve)?;
        xfer_object_count_map(xfer, &mut self.objects_captured, &mut resolve)?;

        Ok(())
    }
}

impl Snapshot for ScoreKeeper {
    fn crc(&self, _xfer: &mut dyn Xfer) {
        // C++ ScoreKeeper::crc is empty.
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut next_synthetic_id: UnsignedInt = 0;
        self.xfer_with_resolver(xfer, |name| {
            let id = next_synthetic_id;
            next_synthetic_id = next_synthetic_id.wrapping_add(1);
            Some(ScoreTemplateDesc {
                id,
                name: name.to_string(),
                kind_of: KindOfMaskType::default(),
                equivalent_root_id: id,
            })
        })
    }
}

fn adjust_count_map(map: &mut ObjectCountMap, tmpl: &ScoreTemplateDesc, delta: Int) {
    if let Some(entry) = map.get_mut(&tmpl.id) {
        entry.count = entry.count.wrapping_add(delta);
    } else {
        map.insert(
            tmpl.id,
            ScoreCountEntry {
                name: tmpl.name.clone(),
                kind_of: tmpl.kind_of.clone(),
                equivalent_root_id: tmpl.equivalent_root_id,
                count: delta,
            },
        );
    }
}

fn xfer_object_count_map<F>(
    xfer: &mut dyn Xfer,
    map: &mut ObjectCountMap,
    resolve: &mut F,
) -> Result<(), XferError>
where
    F: FnMut(&str) -> Option<ScoreTemplateDesc>,
{
    let mut version: XferVersion = 1;
    xfer.xfer_version(&mut version, 1)?;

    let mut map_size: UnsignedShort = map.len() as UnsignedShort;
    xfer.xfer_unsigned_short(&mut map_size)?;

    if xfer.mode() == XferMode::Save || xfer.mode() == XferMode::Crc {
        for entry in map.values() {
            let mut name = entry.name.clone();
            xfer.xfer_ascii_string(&mut name)?;
            let mut count = entry.count;
            xfer.xfer_int(&mut count)?;
        }
    } else {
        map.clear();
        for _ in 0..map_size {
            let mut name = String::new();
            xfer.xfer_ascii_string(&mut name)?;
            let desc = resolve(&name).ok_or(XferError::InvalidData)?;
            let mut count: Int = 0;
            xfer.xfer_int(&mut count)?;
            map.insert(
                desc.id,
                ScoreCountEntry {
                    name: desc.name,
                    kind_of: desc.kind_of,
                    equivalent_root_id: desc.equivalent_root_id,
                    count,
                },
            );
        }
    }

    Ok(())
}

fn xfer_i32_array<const N: usize>(xfer: &mut dyn Xfer, arr: &mut [Int; N]) -> Result<(), XferError> {
    let mut buf = vec![0u8; N * 4];
    for (i, &val) in arr.iter().enumerate() {
        buf[i * 4..i * 4 + 4].copy_from_slice(&val.to_le_bytes());
    }
    xfer.xfer_user(&mut buf)?;
    for (i, slot) in arr.iter_mut().enumerate() {
        *slot = i32::from_le_bytes([
            buf[i * 4],
            buf[i * 4 + 1],
            buf[i * 4 + 2],
            buf[i * 4 + 3],
        ]);
    }
    Ok(())
}

#[inline]
fn scoring_building_mask() -> KindOfMaskType {
    let mut m = KindOfMaskType::default();
    m.set(KindOfType::Structure as Int, 1);
    m.set(KindOfType::Score as Int, 1);
    m
}

#[inline]
fn scoring_building_create_mask() -> KindOfMaskType {
    let mut m = KindOfMaskType::default();
    m.set(KindOfType::Structure as Int, 1);
    m.set(KindOfType::ScoreCreate as Int, 1);
    m
}

#[inline]
fn scoring_building_destroy_mask() -> KindOfMaskType {
    let mut m = KindOfMaskType::default();
    m.set(KindOfType::Structure as Int, 1);
    m.set(KindOfType::ScoreDestroy as Int, 1);
    m
}
