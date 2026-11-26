// SPDX-License-Identifier: GPL-3.0-or-later
// Not part of the game. `scripts/build_rust_engine.py --witness` inserts this at the
// top of `Player::crc`, in a stock build and a Rust build alike, because the stock
// CRC leaves money and energy out and would not notice a wrong replacement.
{
	UnsignedInt money = m_money.countMoney();
	UnsignedInt income = m_money.getCashPerMinute();
	Int production = m_energy.getProduction();
	Int consumption = m_energy.getConsumption();
	Real ratio = m_energy.getEnergySupplyRatio();
	Bool powered = m_energy.hasSufficientPower();
	xfer->xferUnsignedInt( &money );
	xfer->xferUnsignedInt( &income );
	xfer->xferInt( &production );
	xfer->xferInt( &consumption );
	xfer->xferReal( &ratio );
	xfer->xferBool( &powered );
}
