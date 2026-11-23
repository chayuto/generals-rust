#!/usr/bin/env python3
"""
Automated V&V Unit Test Suite for Tier 0 Context Slicer (scripts/prep_packet.py).
Verifies boilerplate stripping, invariant extraction (includes, Xfer calls,
PRNG calls, RETAIL_COMPATIBLE guards, INI FieldParse tables), and packet rendering.
"""

from __future__ import annotations

import pathlib
import sys
import tempfile
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

    def test_resolve_include_paths_finds_generalsmd_and_core_headers(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            vendor_root = pathlib.Path(tmp) / "vendor/GeneralsX"
            foo_h = vendor_root / "GeneralsMD/Code/GameEngine/Include/GameLogic/Foo.h"
            bar_h = vendor_root / "Core/GameEngine/Include/Common/Bar.h"
            local_h = vendor_root / "GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Local.h"
            for p in (foo_h, bar_h, local_h):
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_text("#pragma once\n", encoding="utf-8")

            resolved = slicer.resolve_include_paths(
                ["GameLogic/Foo.h", "Common/Bar.h", "Local.h", "Missing/Baz.h"],
                ["GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Foo.cpp"],
                vendor_root=vendor_root,
            )
            self.assertEqual(
                resolved["GameLogic/Foo.h"],
                "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Foo.h",
            )
            self.assertEqual(
                resolved["Common/Bar.h"],
                "vendor/GeneralsX/Core/GameEngine/Include/Common/Bar.h",
            )
            self.assertEqual(
                resolved["Local.h"],
                "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Local.h",
            )
            self.assertIsNone(resolved["Missing/Baz.h"])

    def test_build_rust_symbol_index_extracts_public_types(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = pathlib.Path(tmp)
            types_rs = repo_root / "crates/zh_common/src/types.rs"
            arena_rs = repo_root / "crates/zh_logic/src/arena/mod.rs"
            types_rs.parent.mkdir(parents=True, exist_ok=True)
            arena_rs.parent.mkdir(parents=True, exist_ok=True)
            types_rs.write_text(
                "pub type Int = i32;\npub struct ObjectId(pub i32);\npub const TRUE: u8 = 1;\n",
                encoding="utf-8",
            )
            arena_rs.write_text(
                "pub struct ObjectSlot(pub u32);\npub enum TakeError { Dead, Reentered }\n",
                encoding="utf-8",
            )

            pkt = {
                "packet_id": "PKT-163",
                "module_name": "Module",
                "phase": 5,
                "work_package": "WP-5.0",
                "complexity_class": "C4",
                "agent_tier": "tier:3-architect",
                "env_access": "env:local-full",
                "ffi_pattern": "Pattern 3",
                "rust_crate": "zh_logic",
                "rust_target": "crates/zh_logic/src/arena/sample.rs",
                "vv_gates": ["V1", "V2-Golden"],
            }
            md = slicer.render_packet_markdown(
                pkt,
                {"Foo.h": '#include "Common/Xfer.h"\n'},
                "cheatsheet",
                vendor_root=repo_root / "missing_vendor",
                repo_root=repo_root,
            )
            self.assertIn("## 2.5 Existing Rust Symbol Index", md)
            self.assertIn("- `zh_logic::arena`: `ObjectSlot`, `TakeError`", md)
            self.assertIn("- `zh_common::types`: `Int`, `ObjectId`, `TRUE`", md)


if __name__ == "__main__":
    unittest.main(verbosity=2)
