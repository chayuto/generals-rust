//! `WP-3.1`: Deterministic economy and player leaf trackers
//! (`AcademyStats`, `Money`, `Energy`, `Handicap`, and `MissionStats`).

pub mod academy_stats;
pub mod energy;
pub mod handicap;
pub mod mission_stats;
pub mod money;

pub const MAX_PLAYER_COUNT: usize = 16;

pub use academy_stats::{
    AcademyAdviceInfo, AcademyObjectInfo, AcademyPlayerContext, AcademyStats,
    FRAMES_BETWEEN_UPDATES, MAX_ADVICE_TIPS,
};
pub use energy::{Energy, EnergyObjectInfo, EnergyOwnerCallbacks, NoopEnergyCallbacks};
pub use handicap::{Handicap, HandicapType, ThingType, HANDICAP_TYPE_COUNT, THING_TYPE_COUNT};
pub use mission_stats::MissionStats;
pub use money::{
    Money, MoneyCallbacks, NoopMoneyCallbacks, INCOME_BUCKET_COUNT, LOGICFRAMES_PER_SECOND,
};
