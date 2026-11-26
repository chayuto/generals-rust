// SPDX-License-Identifier: GPL-3.0-or-later
// Rust-backed replacements for the behaviour of the engine's Energy.cpp.
// `scripts/build_rust_engine.py` includes this in a copy of that file, in place of the
// original functions. The members stay in C++, so `xfer` is unchanged.

#include "zh_ffi.h"

#if RETAIL_COMPATIBLE_CRC
#error "libzh_ffi is built without the retail-crc feature; remove_power_bonus would differ"
#endif

// The original reads the frame only when there is an owner to tell.
#define ZH_ENERGY_STATE(state) \
	ZhEnergyState state = { m_energyProduction, m_energyConsumption, \
		m_powerSabotagedTillFrame, static_cast<uint8_t>(m_owner != nullptr) }

// Stores the new numbers first: the owner reads them while it is being told.
#define ZH_ENERGY_APPLY(call) \
	do { \
		ZH_ENERGY_STATE(state); \
		const UnsignedInt frame = m_owner ? TheGameLogic->getFrame() : 0; \
		const int32_t brownout = call; \
		m_energyProduction = state.production; \
		m_energyConsumption = state.consumption; \
		if (brownout != ZH_ENERGY_NO_NOTICE) \
			m_owner->onPowerBrownOutChange(brownout != 0); \
	} while (0)

Int Energy::getProduction() const
{
	ZH_ENERGY_STATE(state);
	return zh_energy_get_production(&state, TheGameLogic->getFrame());
}

Real Energy::getEnergySupplyRatio() const
{
	ZH_ENERGY_STATE(state);
	return zh_energy_supply_ratio(&state, TheGameLogic->getFrame());
}

Bool Energy::hasSufficientPower() const
{
	ZH_ENERGY_STATE(state);
	return zh_energy_has_sufficient_power(&state, TheGameLogic->getFrame()) != 0;
}

void Energy::adjustPower(Int powerDelta, Bool adding)
{
	ZH_ENERGY_APPLY(zh_energy_adjust_power(&state, powerDelta, adding, frame));
}

void Energy::objectEnteringInfluence( Object *obj )
{
	if( obj == nullptr )
		return;
	Int energy = obj->getTemplate()->getEnergyProduction();
#ifdef ZH_SEAM_MUTATE
	// Deliberate fault, used to prove that the replay check fails: power plants give less.
	if (energy > 0)
		energy /= 2;
#endif
	ZH_ENERGY_APPLY(zh_energy_object_entering(&state, energy, frame));
}

void Energy::objectLeavingInfluence( Object *obj )
{
	if( obj == nullptr )
		return;
	const Int energy = obj->getTemplate()->getEnergyProduction();
	ZH_ENERGY_APPLY(zh_energy_object_leaving(&state, energy, frame));
}

void Energy::addPowerBonus( Object *obj )
{
	if( obj == nullptr )
		return;
	const Int bonus = obj->getTemplate()->getEnergyBonus();
	ZH_ENERGY_APPLY(zh_energy_add_bonus(&state, bonus, frame));
}

void Energy::removePowerBonus( Object *obj )
{
	if( obj == nullptr )
		return;
	const Int bonus = obj->getTemplate()->getEnergyBonus();
	const Bool disabled = obj->isDisabled();
	ZH_ENERGY_APPLY(zh_energy_remove_bonus(&state, bonus, disabled, frame));
}

#undef ZH_ENERGY_STATE
#undef ZH_ENERGY_APPLY
