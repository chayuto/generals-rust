//! Entry points for the engine's `Money.cpp` (`PKT-121`) and `Energy.cpp` (`PKT-118`).
//!
//! C++ keeps the members. Each call runs the ported type on a copy and writes the
//! result back. What the engine must be told (sounds, income, brownout) is
//! returned, not called, so C++ stores the new state before it acts. In both
//! files the original tells the engine as its last step, so the order holds.

use zh_common::types::{Bool, Int, FALSE, TRUE};
use zh_logic::economy::energy::{Energy, EnergyObjectInfo, EnergyOwnerCallbacks};
use zh_logic::economy::money::{Money, MoneyCallbacks, MoneyState, INCOME_BUCKET_COUNT};

// ---------------------------------------------------------------------------
// Money
// ---------------------------------------------------------------------------

/// Mirrors `ZhMoneyState` in `zh_ffi.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZhMoneyState {
    pub money: u32,
    pub player_index: i32,
    pub income_buckets: [u32; INCOME_BUCKET_COUNT],
    pub current_bucket: u32,
    pub cash_per_minute: u32,
}

pub const ZH_MONEY_WITHDRAW_SOUND: u32 = 1;
pub const ZH_MONEY_DEPOSIT_SOUND: u32 = 2;
pub const ZH_MONEY_RECORD_INCOME: u32 = 4;

#[derive(Default)]
struct Notices(u32);

impl MoneyCallbacks for Notices {
    fn trigger_withdraw_audio(&mut self, _player_index: Int) {
        self.0 |= ZH_MONEY_WITHDRAW_SOUND;
    }
    fn trigger_deposit_audio(&mut self, _player_index: Int) {
        self.0 |= ZH_MONEY_DEPOSIT_SOUND;
    }
    fn record_income(&mut self, _player_index: Int) {
        self.0 |= ZH_MONEY_RECORD_INCOME;
    }
}

/// Runs `op` on the money behind `state` and writes the result back.
unsafe fn with_money<R>(state: *mut ZhMoneyState, op: impl FnOnce(&mut Money) -> R) -> R {
    let s = unsafe { &mut *state };
    let mut money = Money::from_state(MoneyState {
        money: s.money,
        player_index: s.player_index,
        income_buckets: s.income_buckets,
        current_bucket: s.current_bucket,
        cash_per_minute: s.cash_per_minute,
    });
    let result = op(&mut money);
    let out = money.state();
    s.money = out.money;
    s.income_buckets = out.income_buckets;
    s.current_bucket = out.current_bucket;
    s.cash_per_minute = out.cash_per_minute;
    result
}

/// Returns the amount withdrawn; `notices` receives the `ZH_MONEY_*` bits.
///
/// # Safety
/// `state` and `notices` must be valid for the call.
#[no_mangle]
pub unsafe extern "C" fn zh_money_withdraw(
    state: *mut ZhMoneyState,
    amount: u32,
    play_sound: u8,
    notices: *mut u32,
) -> u32 {
    let mut told = Notices::default();
    let taken = unsafe {
        with_money(state, |m| {
            m.withdraw(amount, Bool::from(play_sound), &mut told)
        })
    };
    unsafe { *notices = told.0 };
    taken
}

/// Returns the `ZH_MONEY_*` bits.
///
/// # Safety
/// `state` must be valid for the call.
#[no_mangle]
pub unsafe extern "C" fn zh_money_deposit(
    state: *mut ZhMoneyState,
    amount: u32,
    play_sound: u8,
    track_income: u8,
) -> u32 {
    let mut told = Notices::default();
    unsafe {
        with_money(state, |m| {
            m.deposit(
                amount,
                Bool::from(play_sound),
                Bool::from(track_income),
                &mut told,
            )
        })
    };
    told.0
}

/// # Safety
/// `state` must be valid for the call.
#[no_mangle]
pub unsafe extern "C" fn zh_money_set_starting_cash(state: *mut ZhMoneyState, amount: u32) {
    unsafe { with_money(state, |m| m.set_starting_cash(amount)) }
}

/// # Safety
/// `state` must be valid for the call.
#[no_mangle]
pub unsafe extern "C" fn zh_money_update_income_bucket(state: *mut ZhMoneyState, frame: u32) {
    unsafe { with_money(state, |m| m.update_income_bucket(frame)) }
}

// ---------------------------------------------------------------------------
// Energy
// ---------------------------------------------------------------------------

/// Mirrors `ZhEnergyState` in `zh_ffi.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZhEnergyState {
    pub production: i32,
    pub consumption: i32,
    pub sabotaged_till_frame: u32,
    pub has_owner: u8,
}

/// Nothing to tell the owner. Otherwise the result is the brownout flag, 0 or 1.
pub const ZH_ENERGY_NO_NOTICE: i32 = -1;

struct Brownout(i32);

impl EnergyOwnerCallbacks for Brownout {
    fn on_power_brown_out_change(&mut self, now_brownout: Bool) {
        self.0 = i32::from(now_brownout != FALSE);
    }
}

fn energy(state: &ZhEnergyState) -> Energy {
    // The ported type only asks whether there is an owner, not which one.
    Energy::from_parts(
        state.production,
        state.consumption,
        state.sabotaged_till_frame,
        (state.has_owner != 0).then_some(0),
    )
}

unsafe fn with_energy(
    state: *mut ZhEnergyState,
    op: impl FnOnce(&mut Energy, &mut Brownout),
) -> i32 {
    let s = unsafe { &mut *state };
    let mut value = energy(s);
    let mut told = Brownout(ZH_ENERGY_NO_NOTICE);
    op(&mut value, &mut told);
    s.production = value.raw_production();
    s.consumption = value.get_consumption();
    told.0
}

fn object(production: i32, bonus: i32, disabled: u8) -> Option<EnergyObjectInfo> {
    Some(EnergyObjectInfo {
        energy_production: production,
        energy_bonus: bonus,
        is_disabled: if disabled != 0 { TRUE } else { FALSE },
    })
}

/// # Safety
/// `state` must be valid for the call; the same holds for every function below.
#[no_mangle]
pub unsafe extern "C" fn zh_energy_get_production(state: *const ZhEnergyState, frame: u32) -> i32 {
    energy(unsafe { &*state }).get_production(frame)
}

/// # Safety
/// See [`zh_energy_get_production`].
#[no_mangle]
pub unsafe extern "C" fn zh_energy_supply_ratio(state: *const ZhEnergyState, frame: u32) -> f32 {
    energy(unsafe { &*state }).get_energy_supply_ratio(frame)
}

/// # Safety
/// See [`zh_energy_get_production`].
#[no_mangle]
pub unsafe extern "C" fn zh_energy_has_sufficient_power(
    state: *const ZhEnergyState,
    frame: u32,
) -> u8 {
    u8::from(energy(unsafe { &*state }).has_sufficient_power(frame) != FALSE)
}

/// # Safety
/// See [`zh_energy_get_production`].
#[no_mangle]
pub unsafe extern "C" fn zh_energy_adjust_power(
    state: *mut ZhEnergyState,
    delta: Int,
    adding: u8,
    frame: u32,
) -> i32 {
    unsafe {
        with_energy(state, |e, told| {
            e.adjust_power(delta, Bool::from(adding), frame, told)
        })
    }
}

/// # Safety
/// See [`zh_energy_get_production`].
#[no_mangle]
pub unsafe extern "C" fn zh_energy_object_entering(
    state: *mut ZhEnergyState,
    production: i32,
    frame: u32,
) -> i32 {
    unsafe {
        with_energy(state, |e, told| {
            e.object_entering_influence(object(production, 0, 0), frame, told)
        })
    }
}

/// # Safety
/// See [`zh_energy_get_production`].
#[no_mangle]
pub unsafe extern "C" fn zh_energy_object_leaving(
    state: *mut ZhEnergyState,
    production: i32,
    frame: u32,
) -> i32 {
    unsafe {
        with_energy(state, |e, told| {
            e.object_leaving_influence(object(production, 0, 0), frame, told)
        })
    }
}

/// # Safety
/// See [`zh_energy_get_production`].
#[no_mangle]
pub unsafe extern "C" fn zh_energy_add_bonus(
    state: *mut ZhEnergyState,
    bonus: i32,
    frame: u32,
) -> i32 {
    unsafe {
        with_energy(state, |e, told| {
            e.add_power_bonus(object(0, bonus, 0), frame, told)
        })
    }
}

/// # Safety
/// See [`zh_energy_get_production`].
#[no_mangle]
pub unsafe extern "C" fn zh_energy_remove_bonus(
    state: *mut ZhEnergyState,
    bonus: i32,
    disabled: u8,
    frame: u32,
) -> i32 {
    unsafe {
        with_energy(state, |e, told| {
            e.remove_power_bonus(object(0, bonus, disabled), frame, told)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn money(amount: u32) -> ZhMoneyState {
        let mut state = ZhMoneyState {
            money: 0,
            player_index: 3,
            income_buckets: [0; INCOME_BUCKET_COUNT],
            current_bucket: 0,
            cash_per_minute: 0,
        };
        unsafe { zh_money_set_starting_cash(&mut state, amount) };
        state
    }

    #[test]
    fn money_state_round_trips_and_reports_notices() {
        let mut s = money(100);
        let mut told = 99;
        assert_eq!(unsafe { zh_money_withdraw(&mut s, 150, 1, &mut told) }, 100);
        assert_eq!((s.money, told), (0, ZH_MONEY_WITHDRAW_SOUND));
        assert_eq!(unsafe { zh_money_withdraw(&mut s, 5, 1, &mut told) }, 0);
        assert_eq!(told, 0);

        unsafe { zh_money_update_income_bucket(&mut s, 30 * 61) };
        assert_eq!(s.current_bucket, 1);
        let told = unsafe { zh_money_deposit(&mut s, 40, 0, 1) };
        assert_eq!(told, ZH_MONEY_RECORD_INCOME);
        assert_eq!(
            (s.money, s.income_buckets[1], s.cash_per_minute, s.player_index),
            (40, 40, 40, 3)
        );
    }

    #[test]
    fn energy_reports_brownout_only_with_an_owner() {
        let mut s = ZhEnergyState {
            production: 0,
            consumption: 0,
            sabotaged_till_frame: 0,
            has_owner: 1,
        };
        assert_eq!(unsafe { zh_energy_object_entering(&mut s, -5, 1) }, 1);
        assert_eq!(unsafe { zh_energy_object_entering(&mut s, 10, 1) }, 0);
        assert_eq!((s.production, s.consumption), (10, 5));
        assert_eq!(unsafe { zh_energy_supply_ratio(&s, 1) }, 2.0);
        assert_eq!(unsafe { zh_energy_adjust_power(&mut s, 0, 1, 1) }, -1);

        s.has_owner = 0;
        assert_eq!(unsafe { zh_energy_object_leaving(&mut s, 10, 1) }, -1);
        assert_eq!(s.production, 0);
        s.sabotaged_till_frame = 5;
        assert_eq!(unsafe { zh_energy_has_sufficient_power(&s, 4) }, 0);
    }
}
