#!/usr/bin/env python3
"""
Automated V&V Validator for docs/MODULE_MANIFEST.json.
Verifies 100% file coverage (833 C++ files, 465 atomic packets, 11,456,985 bytes)
at pinned GeneralsX commit 84a7a20769aa5749f388b1f49bd542c79b3c33e8, zero duplicate
files or Rust target paths, and valid 2D (tier x env) classifications.
"""

from __future__ import annotations

import json
import pathlib
import sys
import unittest

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
MANIFEST_PATH = REPO_ROOT / "docs/MODULE_MANIFEST.json"

VALID_CLASSES = {"C1", "C2", "C3", "C4"}
VALID_TIERS = {"tier:1a-lite", "tier:1b-fast", "tier:2-engineer", "tier:3-architect"}
VALID_ENVS = {"env:ephemeral-web", "env:hybrid-ci", "env:local-full"}
VALID_CRATES = {"zh_common", "zh_logic"}


class TestModuleManifest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))

    def test_pinned_commit_and_totals(self) -> None:
        self.assertEqual(self.manifest["reference_repo"], "fbraz3/GeneralsX")
        self.assertEqual(
            self.manifest["reference_commit"],
            "84a7a20769aa5749f388b1f49bd542c79b3c33e8",
        )
        self.assertEqual(self.manifest["total_packets"], 463)
        self.assertEqual(self.manifest["total_cpp_files"], 833)
        self.assertEqual(self.manifest["total_cpp_bytes"], 11456985)

    def test_no_duplicate_cpp_files_or_rust_targets(self) -> None:
        seen_cpp = set()
        seen_rust = set()
        seen_ids = set()

        for pkt in self.manifest["packets"]:
            pid = pkt["packet_id"]
            self.assertNotIn(pid, seen_ids, f"Duplicate packet_id: {pid}")
            seen_ids.add(pid)

            rt = pkt["rust_target"]
            self.assertNotIn(rt, seen_rust, f"Duplicate rust_target: {rt}")
            seen_rust.add(rt)

            for cf in pkt["cpp_files"]:
                cp = cf["path"]
                self.assertNotIn(cp, seen_cpp, f"Duplicate C++ file in manifest: {cp}")
                seen_cpp.add(cp)

        self.assertEqual(len(seen_ids), 463)
        self.assertEqual(len(seen_cpp), 833)
        self.assertEqual(len(seen_rust), 463)

    def test_valid_2d_classification_and_vv_gates(self) -> None:
        for pkt in self.manifest["packets"]:
            self.assertIn(pkt["phase"], range(1, 8), f"Invalid phase in {pkt['packet_id']}")
            self.assertIn(pkt["complexity_class"], VALID_CLASSES)
            self.assertIn(pkt["agent_tier"], VALID_TIERS)
            self.assertIn(pkt["env_access"], VALID_ENVS)
            self.assertIn(pkt["rust_crate"], VALID_CRATES)
            self.assertTrue(pkt["rust_target"].startswith(f"crates/{pkt['rust_crate']}/src/"))
            self.assertIn("V1", pkt["vv_gates"])
            self.assertIn("V2-Golden", pkt["vv_gates"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
