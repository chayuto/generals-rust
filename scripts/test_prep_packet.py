#!/usr/bin/env python3
"""
Automated V&V Unit Test Suite for Tier 0 Context Slicer (scripts/prep_packet.py).
Verifies boilerplate stripping, invariant extraction (includes, Xfer calls,
PRNG calls, RETAIL_COMPATIBLE guards, INI FieldParse tables), and packet rendering.
"""

from __future__ import annotations

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import prep_packet as slicer  # noqa: E402


class TestPrepPacketSlicer(unittest.TestCase):
    def test_strip_cpp_boilerplate_removes_license_header(self) -> None:
        raw = """/*
** Command & Conquer Generals Zero Hour(tm)
** Copyright 2025 Electronic Arts Inc.
*/
////////////////////////////////////////////////////////////////////////////////
// (c) 2001-2003 Electronic Arts Inc.
////////////////////////////////////////////////////////////////////////////////

#pragma once
class Foo {};
"""
        stripped = slicer.strip_cpp_boilerplate(raw)
        self.assertTrue(stripped.startswith("#pragma once"))
        self.assertNotIn("Copyright 2025", stripped)

    def test_analyze_cpp_invariants_extracts_xfer_rng_and_retail_guards(self) -> None:
        sample_cpp = """
        #include "Common/Xfer.h"
        #include "GameLogic/ExperienceTracker.h"

        static const FieldParse myFieldParse[] = {
            { "Armor", ArmorTemplate::parseArmorCoefficients, nullptr, 0 },
            { nullptr, nullptr, nullptr, 0 }
        };

        void ExperienceTracker::crc( Xfer *xfer ) {
            xfer->xferInt( &m_currentExperience );
            xfer->xferUser( &m_currentLevel, sizeof( VeterancyLevel ) );
        #if !RETAIL_COMPATIBLE_CRC
            xfer->xferBool(&m_isTrainable);
        #endif
            Int r = GetGameLogicRandomValue(0, 10, __FILE__, __LINE__);
        }
        """
        inv = slicer.analyze_cpp_invariants({"ExperienceTracker.cpp": sample_cpp})
        self.assertEqual(inv["includes"], ["Common/Xfer.h", "GameLogic/ExperienceTracker.h"])
        self.assertEqual(len(inv["xfer_calls"]), 3)
        self.assertIn("xferInt( &m_currentExperience )", inv["xfer_calls"][0])
        self.assertEqual(len(inv["rng_calls"]), 1)
        self.assertEqual(inv["retail_guards"], ["!RETAIL_COMPATIBLE_CRC"])
        self.assertEqual(inv["ini_fields"], ['"Armor" => ArmorTemplate::parseArmorCoefficients'])


if __name__ == "__main__":
    unittest.main(verbosity=2)
