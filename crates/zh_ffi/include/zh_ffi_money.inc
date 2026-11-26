// SPDX-License-Identifier: GPL-3.0-or-later
// Rust-backed replacements for the behaviour of the engine's Money.cpp.
// `scripts/build_rust_engine.py` includes this in a copy of that file, in place of the
// original functions. The members stay in C++, so `xfer` is unchanged.

#include "zh_ffi.h"

#if defined(RTS_DEBUG)
#error "the replaced Money::withdraw has a debug-only free-build check"
#endif

static_assert(LOGICFRAMES_PER_SECOND == 30, "the Rust tracker assumes 30 logic frames a second");

namespace
{

ZhMoneyState zhMoneyState(UnsignedInt money, Int playerIndex, const UnsignedInt (&buckets)[60],
	UnsignedInt currentBucket, UnsignedInt cashPerMinute)
{
	ZhMoneyState state;
	state.money = money;
	state.player_index = playerIndex;
	std::copy(buckets, buckets + 60, state.income_buckets);
	state.current_bucket = currentBucket;
	state.cash_per_minute = cashPerMinute;
	return state;
}

} // namespace

#define ZH_MONEY_LOAD(state) \
	ZhMoneyState state = zhMoneyState(m_money, m_playerIndex, m_incomeBuckets, \
		m_currentBucket, m_cashPerMinute)

#define ZH_MONEY_STORE(state) \
	do { \
		m_money = state.money; \
		std::copy(state.income_buckets, state.income_buckets + 60, m_incomeBuckets); \
		m_currentBucket = state.current_bucket; \
		m_cashPerMinute = state.cash_per_minute; \
	} while (0)

UnsignedInt Money::withdraw(UnsignedInt amountToWithdraw, Bool playSound)
{
	ZH_MONEY_LOAD(state);
	uint32_t notices = 0;
	const UnsignedInt taken = zh_money_withdraw(&state, amountToWithdraw, playSound, &notices);
	// The original plays the sound before it subtracts; the sound does not read the money.
	if (notices & ZH_MONEY_WITHDRAW_SOUND)
		triggerAudioEvent(TheAudio->getMiscAudio()->m_moneyWithdrawSound);
	ZH_MONEY_STORE(state);
	return taken;
}

void Money::deposit(UnsignedInt amountToDeposit, Bool playSound, Bool trackIncome)
{
#ifdef ZH_SEAM_MUTATE
	// Deliberate fault, used to prove that the replay check fails: income is halved.
	amountToDeposit /= 2;
#endif
	ZH_MONEY_LOAD(state);
	const uint32_t notices = zh_money_deposit(&state, amountToDeposit, playSound, trackIncome);
	if (notices & ZH_MONEY_DEPOSIT_SOUND)
		triggerAudioEvent(TheAudio->getMiscAudio()->m_moneyDepositSound);
	ZH_MONEY_STORE(state);
	if (notices & ZH_MONEY_RECORD_INCOME)
	{
		Player *player = ThePlayerList->getNthPlayer( m_playerIndex );
		if( player )
		{
			player->getAcademyStats()->recordIncome();
		}
	}
}

void Money::setStartingCash(UnsignedInt amount)
{
	ZH_MONEY_LOAD(state);
	zh_money_set_starting_cash(&state, amount);
	ZH_MONEY_STORE(state);
}

void Money::updateIncomeBucket()
{
	ZH_MONEY_LOAD(state);
	zh_money_update_income_bucket(&state, TheGameLogic->getFrame());
	ZH_MONEY_STORE(state);
}

UnsignedInt Money::getCashPerMinute() const
{
	return m_cashPerMinute;
}

#undef ZH_MONEY_LOAD
#undef ZH_MONEY_STORE
