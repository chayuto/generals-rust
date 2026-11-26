// SPDX-License-Identifier: GPL-3.0-or-later
// Rust-backed replacements for the generator functions in the engine's RandomValue.cpp.
// `scripts/build_rust_engine.py` includes this in a copy of that file, in place of the
// original generators. It relies on that file's own includes for the engine types.

#include "zh_ffi.h"

#if RETAIL_COMPATIBLE_CRC
#error "libzh_ffi is built without the retail-crc feature; its random ranges would differ"
#endif

UnsignedInt GetGameLogicRandomSeed() { return zh_random_get_logic_base_seed(); }
UnsignedInt GetGameLogicRandomSeedCRC() { return zh_random_get_logic_seed_crc(); }

void InitRandom() { zh_random_init_time(); }
#ifdef ZH_SEAM_MUTATE
// Deliberate fault, used to prove that the replay check fails: one extra logic draw
// on the 100th call after seeding, which shifts the generator state the CRC covers.
static int zh_mutate_calls = 0;
#endif

void InitRandom(UnsignedInt seed)
{
#ifdef ZH_SEAM_MUTATE
	zh_mutate_calls = 0;
#endif
	zh_random_init_seed(seed);
}

Int GetGameAudioRandomValue(int lo, int hi, const char*, int)
{
	return zh_random_get_audio_int(lo, hi);
}

Real GetGameAudioRandomValueReal(Real lo, Real hi, const char*, int)
{
	return zh_random_get_audio_real(lo, hi);
}

Int GetGameClientRandomValue(int lo, int hi, const char*, int)
{
	return zh_random_get_client_int(lo, hi);
}

Real GetGameClientRandomValueReal(Real lo, Real hi, const char*, int)
{
	return zh_random_get_client_real(lo, hi);
}

Int GetGameLogicRandomValue(int lo, int hi, const char*, int)
{
#ifdef ZH_SEAM_MUTATE
	if (++zh_mutate_calls == 100)
		zh_random_get_logic_int(0, 1);
#endif
	return zh_random_get_logic_int(lo, hi);
}

Real GetGameLogicRandomValueReal(Real lo, Real hi, const char*, int)
{
	return zh_random_get_logic_real(lo, hi);
}

Int GetGameLogicRandomValueUnchanged(int lo, int hi, const char*, int)
{
	return zh_random_get_logic_int_unchanged(lo, hi);
}

Real GetGameLogicRandomValueRealUnchanged(Real lo, Real hi, const char*, int)
{
	return zh_random_get_logic_real_unchanged(lo, hi);
}
