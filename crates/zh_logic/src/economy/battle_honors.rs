//! `PKT-117`: Battle honors bitmask and challenge mask constants
//! (`GeneralsMD/Code/GameEngine/Include/Common/BattleHonors.h`).

use zh_common::types::{Int, UnsignedInt};

pub const BATTLE_HONOR_LADDER_CHAMP: UnsignedInt = 0x0000_0001;
pub const BATTLE_HONOR_STREAK: UnsignedInt = 0x0000_0002;
pub const BATTLE_HONOR_LOYALTY_USA: UnsignedInt = 0x0000_0020;
pub const BATTLE_HONOR_LOYALTY_CHINA: UnsignedInt = 0x0000_0040;
pub const BATTLE_HONOR_BATTLE_TANK: UnsignedInt = 0x0000_0080;
pub const BATTLE_HONOR_AIR_WING: UnsignedInt = 0x0000_0100;
pub const BATTLE_HONOR_LOYALTY_GLA: UnsignedInt = 0x0000_0200;
pub const BATTLE_HONOR_ENDURANCE: UnsignedInt = 0x0000_0400;
pub const BATTLE_HONOR_CAMPAIGN_USA: UnsignedInt = 0x0000_0800;
pub const BATTLE_HONOR_CAMPAIGN_CHINA: UnsignedInt = 0x0000_1000;
pub const BATTLE_HONOR_CAMPAIGN_GLA: UnsignedInt = 0x0000_2000;
pub const BATTLE_HONOR_BLITZ5: UnsignedInt = 0x0000_4000;
pub const BATTLE_HONOR_BLITZ10: UnsignedInt = 0x0000_8000;
pub const BATTLE_HONOR_FAIR_PLAY: UnsignedInt = 0x0001_0000;
pub const BATTLE_HONOR_APOCALYPSE: UnsignedInt = 0x0002_0000;
pub const BATTLE_HONOR_OFFICERSCLUB: UnsignedInt = 0x0004_0000;
pub const BATTLE_HONOR_DOMINATION: UnsignedInt = 0x0008_0000;
pub const BATTLE_HONOR_CHALLENGE_MODE: UnsignedInt = 0x0010_0000;
pub const BATTLE_HONOR_ULTIMATE: UnsignedInt = 0x0020_0000;
pub const BATTLE_HONOR_GLOBAL_GENERAL: UnsignedInt = 0x0040_0000;
pub const BATTLE_HONOR_DOMINATION_ONLINE: UnsignedInt = 0x0080_0000;
pub const BATTLE_HONOR_STREAK_ONLINE: UnsignedInt = 0x0100_0000;
pub const BATTLE_HONOR_CHALLENGE: UnsignedInt = 0x0200_0000;
pub const BATTLE_HONOR_NOT_GAINED: UnsignedInt = 0x0800_0000;

/// Bitwise OR of all active `BATTLE_HONOR_*` bits defined in `BattleHonors.h`.
pub const BATTLE_HONOR_ALL_MASK: UnsignedInt = BATTLE_HONOR_LADDER_CHAMP
    | BATTLE_HONOR_STREAK
    | BATTLE_HONOR_LOYALTY_USA
    | BATTLE_HONOR_LOYALTY_CHINA
    | BATTLE_HONOR_BATTLE_TANK
    | BATTLE_HONOR_AIR_WING
    | BATTLE_HONOR_LOYALTY_GLA
    | BATTLE_HONOR_ENDURANCE
    | BATTLE_HONOR_CAMPAIGN_USA
    | BATTLE_HONOR_CAMPAIGN_CHINA
    | BATTLE_HONOR_CAMPAIGN_GLA
    | BATTLE_HONOR_BLITZ5
    | BATTLE_HONOR_BLITZ10
    | BATTLE_HONOR_FAIR_PLAY
    | BATTLE_HONOR_APOCALYPSE
    | BATTLE_HONOR_OFFICERSCLUB
    | BATTLE_HONOR_DOMINATION
    | BATTLE_HONOR_CHALLENGE_MODE
    | BATTLE_HONOR_ULTIMATE
    | BATTLE_HONOR_GLOBAL_GENERAL
    | BATTLE_HONOR_DOMINATION_ONLINE
    | BATTLE_HONOR_STREAK_ONLINE
    | BATTLE_HONOR_CHALLENGE
    | BATTLE_HONOR_NOT_GAINED;

pub const BH_CHALLENGE_MASK_1: UnsignedInt = 0x0001;
pub const BH_CHALLENGE_MASK_2: UnsignedInt = 0x0002;
pub const BH_CHALLENGE_MASK_3: UnsignedInt = 0x0004;
pub const BH_CHALLENGE_MASK_4: UnsignedInt = 0x0008;
pub const BH_CHALLENGE_MASK_5: UnsignedInt = 0x0010;
pub const BH_CHALLENGE_MASK_6: UnsignedInt = 0x0020;
pub const BH_CHALLENGE_MASK_7: UnsignedInt = 0x0040;

pub const BH_CHALLENGE_ALL_MASK: UnsignedInt = BH_CHALLENGE_MASK_1
    | BH_CHALLENGE_MASK_2
    | BH_CHALLENGE_MASK_3
    | BH_CHALLENGE_MASK_4
    | BH_CHALLENGE_MASK_5
    | BH_CHALLENGE_MASK_6
    | BH_CHALLENGE_MASK_7;

pub const MAX_BATTLE_HONOR_COLUMNS: Int = 4;
pub const MAX_BATTLE_HONOR_IMAGE_WIDTH: Int = 40;
pub const MAX_BATTLE_HONOR_IMAGE_HEIGHT: Int = 41;

/// Strongly-typed wrapper around the 32-bit `BattleHonors` bitfield.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BattleHonorFlags(pub UnsignedInt);

impl BattleHonorFlags {
    #[inline]
    pub const fn new(bits: UnsignedInt) -> Self {
        Self(bits)
    }

    #[inline]
    pub const fn bits(self) -> UnsignedInt {
        self.0
    }

    #[inline]
    pub const fn has(self, mask: UnsignedInt) -> bool {
        (self.0 & mask) == mask
    }

    #[inline]
    pub const fn has_any(self, mask: UnsignedInt) -> bool {
        (self.0 & mask) != 0
    }

    #[inline]
    pub fn grant(&mut self, mask: UnsignedInt) {
        self.0 |= mask;
    }

    #[inline]
    pub const fn mask_defined(self) -> Self {
        Self(self.0 & BATTLE_HONOR_ALL_MASK)
    }
}
