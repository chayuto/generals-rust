#!/usr/bin/env python3
"""Automated self-tests for `scripts/pack_wave.py`."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import pack_wave

REPO_ROOT = SCRIPTS_DIR.parent


def create_fake_repo(root: Path) -> None:
    (root / "crates/zh_logic/src/sim").mkdir(parents=True, exist_ok=True)
    (root / "crates/zh_logic/src/modules").mkdir(parents=True, exist_ok=True)
    (root / "packets/phase5").mkdir(parents=True, exist_ok=True)
    (root / "tests/cpp_harness").mkdir(parents=True, exist_ok=True)

    (root / "crates/zh_logic/src/sim/mod.rs").write_text(
        "pub struct ObjectRecord {\n"
        "    pub id: ObjectId,\n"
        "}\n\n"
        "pub trait SimView {\n"
        "    fn frame(&self) -> UnsignedInt;\n"
        "}\n\n"
        "pub trait SimApi: SimView {\n"
        "    fn destroy_object(&mut self, slot: ObjectSlot);\n"
        "}\n\n"
        "impl SimContext {\n"
        "    fn HUGE_IMPL_BODY() { /* must not appear */ }\n"
        "}\n",
        encoding="utf-8",
    )

    (root / "crates/zh_logic/src/modules/module.rs").write_text(
        "impl ModuleSnapshotHeader {\n"
        "    pub fn xfer_die_module_header(xfer: &mut dyn Xfer) -> Result<(), XferError> { Ok(()) }\n"
        "}\n\n"
        "pub trait BehaviorModule: Send {\n"
        "    fn module_name_key(&self) -> NameKeyType;\n"
        "}\n\n"
        "pub trait DieFacet {\n"
        "    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo);\n"
        "}\n\n"
        "pub trait UpdateFacet {\n"
        "    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime;\n"
        "}\n",
        encoding="utf-8",
    )

    (root / "crates/zh_logic/src/modules/module_factory.rs").write_text(
        "pub const BUILTIN_ZH_MODULES: [BuiltinModuleEntry; 2] = [\n"
        '    BuiltinModuleEntry { name: "DestroyDie", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x002), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },\n'
        '    BuiltinModuleEntry { name: "AutoHealBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x085), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },\n'
        "];\n",
        encoding="utf-8",
    )

    (root / "tests/cpp_harness/oracle_wp51_world.h").write_text(
        "namespace oracle_wp51 {\n"
        "struct World {\n"
        "    Team* team_a = nullptr;\n"
        "    void load_ini(const std::string& text) {\n"
        "        // body stripped\n"
        "    }\n"
        "};\n"
        "}\n",
        encoding="utf-8",
    )

    (root / "packets/phase5/PKT-211_DestroyDie.md").write_text(
        "# Translation Packet `PKT-211`: `DestroyDie`\n"
        "## 1. Metadata\nPREAMBLE_NOISE\n"
        "## Compact Worker Cheat Sheet (Embedded in All Translation Packets)\nCHEAT_NOISE\n"
        "## 3. Reference C++ Source (`fbraz3/GeneralsX` at `84a7a20`)\n"
        "```cpp\n"
        "/*\n**\tCommand & Conquer Generals Zero Hour(tm)\n**\tCopyright 2025\n*/\n"
        "class DestroyDie : public DieModule {};\n"
        "```\n",
        encoding="utf-8",
    )


class TestPackWave(unittest.TestCase):
    def test_hermetic_extraction_strips_preambles_and_impls(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            create_fake_repo(root)
            out_path = root / "bundle.md"
            rc = pack_wave.main(
                [
                    "--repo-root",
                    str(root),
                    "--packets",
                    "PKT-211",
                    "--facet",
                    "die",
                    "--out",
                    str(out_path),
                ]
            )
            self.assertEqual(rc, 0)
            text = out_path.read_text(encoding="utf-8")
            self.assertIn("pub struct ObjectRecord", text)
            self.assertIn("pub trait SimView", text)
            self.assertIn("pub trait SimApi: SimView", text)
            self.assertIn("pub trait DieFacet", text)
            self.assertIn('name: "DestroyDie"', text)
            self.assertIn("class DestroyDie : public DieModule {};", text)
            self.assertIn("void load_ini(const std::string& text);", text)

            # Impl bodies, unrelated facets/entries, and repeated preambles are stripped.
            self.assertNotIn("HUGE_IMPL_BODY", text)
            self.assertNotIn("UpdateFacet", text)
            self.assertNotIn("AutoHealBehavior", text)
            self.assertNotIn("PREAMBLE_NOISE", text)
            self.assertNotIn("CHEAT_NOISE", text)
            self.assertNotIn("Command & Conquer Generals Zero Hour", text)

    def test_real_repo_wp52_die_bundle_stays_compact(self) -> None:
        bundle = pack_wave.build_wave_bundle(
            repo_root=REPO_ROOT,
            packets=[
                "PKT-205",
                "PKT-206",
                "PKT-207",
                "PKT-208",
                "PKT-211",
                "PKT-212",
                "PKT-213",
                "PKT-217",
                "PKT-220",
                "PKT-230",
                "PKT-243",
                "PKT-252",
            ],
            facet="die",
        )
        est_tokens = len(bundle.encode("utf-8")) // 4
        self.assertGreater(est_tokens, 5000)
        self.assertLess(est_tokens, 40000)
        self.assertIn("pub trait DieFacet", bundle)
        self.assertIn("FXListDie", bundle)

    def test_comma_separated_multi_facet_deduped(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            create_fake_repo(root)
            out_path = root / "multi.md"
            rc = pack_wave.main(
                [
                    "--repo-root",
                    str(root),
                    "--packets",
                    "PKT-211",
                    "--facet",
                    "die,update,die",
                    "--out",
                    str(out_path),
                ]
            )
            self.assertEqual(rc, 0)
            text = out_path.read_text(encoding="utf-8")
            self.assertIn("pub trait DieFacet", text)
            self.assertIn("pub trait UpdateFacet", text)
            self.assertEqual(text.count("pub trait DieFacet"), 1)


if __name__ == "__main__":
    unittest.main()
