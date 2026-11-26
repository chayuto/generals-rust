//! `PKT-121`: Player cash balance & 60-second circular income rate tracker
//! (`GeneralsMD/Code/GameEngine/Source/Common/RTS/Money.cpp`).

use zh_common::types::{Bool, Int, UnsignedInt, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError, Snapshot};

pub const INCOME_BUCKET_COUNT: usize = 60;
pub const LOGICFRAMES_PER_SECOND: UnsignedInt = 30;

/// Host/player callbacks emitted by [`Money`] mutations.
pub trait MoneyCallbacks {
    fn trigger_withdraw_audio(&mut self, _player_index: Int) {}
    fn trigger_deposit_audio(&mut self, _player_index: Int) {}
    fn record_income(&mut self, _player_index: Int) {}
}

/// No-op implementation of [`MoneyCallbacks`] for headless or isolated callers.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopMoneyCallbacks;

impl MoneyCallbacks for NoopMoneyCallbacks {}

/// The members of C++ `Money`, for a caller that keeps them itself (`zh_ffi`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoneyState {
    pub money: UnsignedInt,
    pub player_index: Int,
    pub income_buckets: [UnsignedInt; INCOME_BUCKET_COUNT],
    pub current_bucket: UnsignedInt,
    pub cash_per_minute: UnsignedInt,
}

/// Player money balance and rolling 60-second income tracker matching C++ `Money`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    money: UnsignedInt,
    player_index: Int,
    income_buckets: [UnsignedInt; INCOME_BUCKET_COUNT],
    current_bucket: UnsignedInt,
    cash_per_minute: UnsignedInt,
}

impl Default for Money {
    fn default() -> Self {
        Self::new()
    }
}

impl Money {
    pub fn new() -> Self {
        let mut out = Self {
            money: 0,
            player_index: 0,
            income_buckets: [0; INCOME_BUCKET_COUNT],
            current_bucket: 0,
            cash_per_minute: 0,
        };
        out.init();
        out
    }

    pub fn from_state(state: MoneyState) -> Self {
        Self {
            money: state.money,
            player_index: state.player_index,
            income_buckets: state.income_buckets,
            current_bucket: state.current_bucket,
            cash_per_minute: state.cash_per_minute,
        }
    }

    pub fn state(&self) -> MoneyState {
        MoneyState {
            money: self.money,
            player_index: self.player_index,
            income_buckets: self.income_buckets,
            current_bucket: self.current_bucket,
            cash_per_minute: self.cash_per_minute,
        }
    }

    pub fn init(&mut self) {
        self.set_starting_cash(0);
    }

    #[inline]
    pub fn count_money(&self) -> UnsignedInt {
        self.money
    }

    /// Withdraws up to `amount_to_withdraw` (cannot go negative) and returns the actual amount.
    pub fn withdraw(
        &mut self,
        amount_to_withdraw: UnsignedInt,
        play_sound: Bool,
        cb: &mut dyn MoneyCallbacks,
    ) -> UnsignedInt {
        let actual = amount_to_withdraw.min(self.money);
        if actual == 0 {
            return 0;
        }
        if play_sound != FALSE {
            cb.trigger_withdraw_audio(self.player_index);
        }
        self.money -= actual;
        actual
    }

    pub fn deposit(
        &mut self,
        amount_to_deposit: UnsignedInt,
        play_sound: Bool,
        track_income: Bool,
        cb: &mut dyn MoneyCallbacks,
    ) {
        if amount_to_deposit == 0 {
            return;
        }
        if play_sound != FALSE {
            cb.trigger_deposit_audio(self.player_index);
        }
        if track_income != FALSE {
            let bucket = (self.current_bucket as usize) % INCOME_BUCKET_COUNT;
            self.income_buckets[bucket] = self.income_buckets[bucket].wrapping_add(amount_to_deposit);
            self.cash_per_minute = self.cash_per_minute.wrapping_add(amount_to_deposit);
        }
        self.money = self.money.wrapping_add(amount_to_deposit);
        if amount_to_deposit > 0 {
            cb.record_income(self.player_index);
        }
    }

    pub fn set_starting_cash(&mut self, amount: UnsignedInt) {
        self.money = amount;
        self.income_buckets = [0; INCOME_BUCKET_COUNT];
        self.current_bucket = 0;
        self.cash_per_minute = 0;
    }

    pub fn update_income_bucket(&mut self, current_frame: UnsignedInt) {
        let next_bucket = (current_frame / LOGICFRAMES_PER_SECOND) % (INCOME_BUCKET_COUNT as UnsignedInt);
        if next_bucket != self.current_bucket {
            self.cash_per_minute = self
                .cash_per_minute
                .wrapping_sub(self.income_buckets[next_bucket as usize]);
            self.current_bucket = next_bucket;
            self.income_buckets[next_bucket as usize] = 0;
        }
    }

    #[inline]
    pub fn get_cash_per_minute(&self) -> UnsignedInt {
        self.cash_per_minute
    }

    #[inline]
    pub fn set_player_index(&mut self, index: Int) {
        self.player_index = index;
    }

    #[inline]
    pub fn player_index(&self) -> Int {
        self.player_index
    }

    #[inline]
    pub fn amount_equal(&self, that: &Self) -> Bool {
        if self.money == that.money {
            TRUE
        } else {
            FALSE
        }
    }
}

impl Snapshot for Money {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let current_version: XferVersion = if cfg!(feature = "retail-xfer-save") {
            1
        } else {
            2
        };
        let mut version = current_version;
        xfer.xfer_version(&mut version, current_version)?;
        xfer.xfer_unsigned_int(&mut self.money)?;

        if version <= 1 {
            self.set_starting_cash(self.money);
        } else {
            let mut raw = [0u8; INCOME_BUCKET_COUNT * 4];
            for (i, &val) in self.income_buckets.iter().enumerate() {
                raw[i * 4..i * 4 + 4].copy_from_slice(&val.to_le_bytes());
            }
            xfer.xfer_user(&mut raw)?;
            for (i, slot) in self.income_buckets.iter_mut().enumerate() {
                *slot = u32::from_le_bytes([
                    raw[i * 4],
                    raw[i * 4 + 1],
                    raw[i * 4 + 2],
                    raw[i * 4 + 3],
                ]);
            }
            xfer.xfer_unsigned_int(&mut self.current_bucket)?;
            self.cash_per_minute = self
                .income_buckets
                .iter()
                .fold(0u32, |acc, &v| acc.wrapping_add(v));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zh_common::xfer::{Xfer, XferLoad, XferSave};

    #[cfg(feature = "retail-xfer-save")]
    #[test]
    fn retail_xfer_save_uses_version_1() {
        let mut m = Money::new();
        let mut cb = NoopMoneyCallbacks;
        m.deposit(2500, FALSE, TRUE, &mut cb);
        let mut save = XferSave::new();
        save.xfer_snapshot(&mut m).expect("save v1");
        assert_eq!(save.as_bytes(), &[1, 0xc4, 0x09, 0x00, 0x00]);

        let mut loaded = Money::new();
        let mut load = XferLoad::new(save.as_bytes());
        load.xfer_snapshot(&mut loaded).expect("load v1");
        assert_eq!(loaded.count_money(), 2500);
        assert_eq!(loaded.get_cash_per_minute(), 0);
    }

    #[cfg(not(feature = "retail-xfer-save"))]
    #[test]
    fn default_xfer_save_roundtrips_income_buckets() {
        let mut m = Money::new();
        let mut cb = NoopMoneyCallbacks;
        m.deposit(1500, FALSE, TRUE, &mut cb);
        m.update_income_bucket(60);
        m.deposit(300, FALSE, TRUE, &mut cb);
        let mut save = XferSave::new();
        save.xfer_snapshot(&mut m).expect("save v2");

        let mut loaded = Money::new();
        let mut load = XferLoad::new(save.as_bytes());
        load.xfer_snapshot(&mut loaded).expect("load v2");
        assert_eq!(loaded.count_money(), 1800);
        assert_eq!(loaded.get_cash_per_minute(), 1800);
    }
}
