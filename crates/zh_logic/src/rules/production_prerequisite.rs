//! `PKT-131`: Unit/structure & science prerequisite evaluator
//! (`GeneralsMD/Code/GameEngine/Include/Common/ProductionPrerequisite.h` &
//! `GeneralsMD/Code/GameEngine/Source/Common/RTS/ProductionPrerequisite.cpp`).

use zh_common::types::{Bool, Int, FALSE, TRUE};

use crate::rules::science::{ScienceType, ScienceVec};

pub const UNIT_OR_WITH_PREV: Int = 0x01;
pub const MAX_PREREQ: usize = 32;

/// Opaque handle representing a resolved `const ThingTemplate*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThingTemplateRef(pub usize);

/// Registry adapter replacing `TheThingFactory->findTemplate(name)` and display names.
pub trait ThingTemplateRegistry {
    fn find_template(&self, name: &str) -> Option<ThingTemplateRef>;
    fn display_name_utf16(&self, tmpl: ThingTemplateRef) -> &[u16];
}

/// Player query adapter replacing `Player::hasScience` & `Player::countObjectsByThingTemplate`.
pub trait PrereqPlayerView {
    fn has_science(&self, st: ScienceType) -> bool;
    fn count_objects_by_thing_template(
        &self,
        tmpls: &[Option<ThingTemplateRef>],
        counts: &mut [Int],
    );
}

/// Text adapter for localized strings needed by [`ProductionPrerequisite::get_requires_list`].
pub trait PrereqTextLookup {
    fn or_requirement_utf16(&mut self) -> Vec<u16>;
    fn generals_promotion_utf16(&mut self) -> Vec<u16>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PrereqUnitRec {
    unit: Option<ThingTemplateRef>,
    flags: Int,
    name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProductionPrerequisite {
    prereq_units: Vec<PrereqUnitRec>,
    prereq_sciences: ScienceVec,
}

impl ProductionPrerequisite {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(&mut self) {
        self.prereq_units.clear();
        self.prereq_sciences.clear();
    }

    pub fn reset_sciences(&mut self) {
        self.prereq_sciences.clear();
    }

    pub fn add_science_prereq(&mut self, science: ScienceType) {
        self.prereq_sciences.push(science);
    }

    pub fn reset_units(&mut self) {
        self.prereq_units.clear();
    }

    pub fn add_unit_prereq(&mut self, unit: &str, or_unit_with_previous: Bool) {
        self.prereq_units.push(PrereqUnitRec {
            name: unit.to_owned(),
            flags: if or_unit_with_previous != FALSE {
                UNIT_OR_WITH_PREV
            } else {
                0
            },
            unit: None,
        });
    }

    pub fn add_unit_prereq_list(&mut self, units: &[&str]) {
        let mut or_with_previous: Bool = FALSE;
        for &u in units {
            self.add_unit_prereq(u, or_with_previous);
            or_with_previous = TRUE;
        }
    }

    pub fn resolve_names(&mut self, factory: &dyn ThingTemplateRegistry) {
        for rec in &mut self.prereq_units {
            if !rec.name.is_empty() {
                rec.unit = factory.find_template(&rec.name);
                rec.name.clear();
            }
        }
    }

    fn calc_num_prereq_units_owned(
        &self,
        player: &dyn PrereqPlayerView,
        counts: &mut [Int; MAX_PREREQ],
    ) -> usize {
        let cnt = self.prereq_units.len().min(MAX_PREREQ);
        let mut tmpls = [None; MAX_PREREQ];
        for (i, slot) in tmpls.iter_mut().take(cnt).enumerate() {
            *slot = self.prereq_units[i].unit;
        }
        player.count_objects_by_thing_template(&tmpls[..cnt], &mut counts[..cnt]);
        cnt
    }

    pub fn get_all_possible_build_facility_templates(
        &self,
        out: &mut [Option<ThingTemplateRef>],
    ) -> Int {
        let mut count = 0usize;
        for (i, rec) in self.prereq_units.iter().enumerate() {
            if i > 0 && (rec.flags & UNIT_OR_WITH_PREV) == 0 {
                break;
            }
            if count >= out.len() {
                break;
            }
            out[count] = rec.unit;
            count += 1;
        }
        count as Int
    }

    pub fn get_existing_build_facility_template(
        &self,
        player: &dyn PrereqPlayerView,
    ) -> Option<ThingTemplateRef> {
        if self.prereq_units.is_empty() {
            return None;
        }
        let mut own_count = [0; MAX_PREREQ];
        let cnt = self.calc_num_prereq_units_owned(player, &mut own_count);
        for i in 0..cnt {
            if i > 0 && (self.prereq_units[i].flags & UNIT_OR_WITH_PREV) == 0 {
                break;
            }
            if own_count[i] != 0 {
                return self.prereq_units[i].unit;
            }
        }
        None
    }

    pub fn is_satisfied(&self, player: Option<&dyn PrereqPlayerView>) -> Bool {
        let Some(player) = player else {
            return FALSE;
        };

        for &sci in &self.prereq_sciences {
            if !player.has_science(sci) {
                return FALSE;
            }
        }

        let mut own_count = [0; MAX_PREREQ];
        let cnt = self.calc_num_prereq_units_owned(player, &mut own_count);

        for i in 1..cnt {
            if (self.prereq_units[i].flags & UNIT_OR_WITH_PREV) != 0 {
                own_count[i] = own_count[i].wrapping_add(own_count[i - 1]);
                own_count[i - 1] = -1;
            }
        }

        for count in own_count.iter().take(cnt) {
            if *count == -1 {
                continue;
            }
            if *count == 0 {
                return FALSE;
            }
        }
        TRUE
    }

    pub fn get_requires_list(
        &self,
        player: Option<&dyn PrereqPlayerView>,
        factory: &dyn ThingTemplateRegistry,
        text: &mut dyn PrereqTextLookup,
    ) -> Vec<u16> {
        let Some(player) = player else {
            return Vec::new();
        };

        let mut requires_list: Vec<u16> = Vec::new();
        let mut own_count = [0; MAX_PREREQ];
        let cnt = self.calc_num_prereq_units_owned(player, &mut own_count);

        let mut or_requirements = [false; MAX_PREREQ];
        for i in 1..cnt {
            if (self.prereq_units[i].flags & UNIT_OR_WITH_PREV) != 0 {
                or_requirements[i] = true;
                own_count[i] = own_count[i].wrapping_add(own_count[i - 1]);
                own_count[i - 1] = -1;
            }
        }

        let mut unit_name: Vec<u16>;
        let mut first_requirement = true;
        for i in 0..cnt {
            if own_count[i] == 0 {
                if or_requirements[i] {
                    if let Some(prev_tmpl) = self.prereq_units[i - 1].unit {
                        unit_name = factory.display_name_utf16(prev_tmpl).to_vec();
                        unit_name.push(b' ' as u16);
                        unit_name.extend_from_slice(&text.or_requirement_utf16());
                        unit_name.push(b' ' as u16);
                        requires_list.extend_from_slice(&unit_name);
                    }
                }

                if let Some(cur_tmpl) = self.prereq_units[i].unit {
                    unit_name = factory.display_name_utf16(cur_tmpl).to_vec();
                    if first_requirement {
                        first_requirement = false;
                    } else {
                        unit_name.push(b'\n' as u16);
                    }
                    requires_list.extend_from_slice(&unit_name);
                }
            }
        }

        let has_sciences = self
            .prereq_sciences
            .iter()
            .all(|&sci| player.has_science(sci));

        if !has_sciences {
            // Mirrors C++ lines 302-308 literally (even the trailing write to unitName instead of requiresList).
            let _ = first_requirement;
            requires_list.extend_from_slice(&text.generals_promotion_utf16());
        }

        requires_list
    }
}
