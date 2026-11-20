#!/usr/bin/env python3
"""
Automated V&V Unit Test Suite for Gate V1 (scripts/lint_determinism.py).
Verifies that every banned determinism pattern (DET001..DET007) is caught
and that compliant deterministic Rust code and explicit exemptions pass cleanly.
"""

from __future__ import annotations

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import lint_determinism as linter  # noqa: E402


class TestDeterminismLinter(unittest.TestCase):
    def setUp(self) -> None:
        self.dummy_path = pathlib.Path("crates/zh_logic/src/dummy.rs")

    def test_clean_deterministic_code_passes(self) -> None:
        clean_rs = """
        use std::collections::{BTreeMap, BTreeSet};

        pub fn step_exp(exp: i32, gain: i32, scalar: f32) -> i32 {
            let scaled = ((gain as f32) * scalar) as i32;
            exp.wrapping_add(scaled)
        }

        pub fn calc_wave(x: f32, y: f32) -> f32 {
            let d = gm_sqrtf(x * x + y * y);
            let a = gm_atan2f(y, x);
            gm_sinf(a) * gm_floorf(d)
        }
        """
        violations = linter.lint_source_text(clean_rs, self.dummy_path)
        self.assertEqual(violations, [])

    def test_det001_rejects_hashmap_and_hashset(self) -> None:
        bad_rs = """
        use std::collections::HashMap;
        let mut s = HashSet::new();
        """
        violations = linter.lint_source_text(bad_rs, self.dummy_path)
        codes = [v.code for v in violations]
        self.assertEqual(codes, ["DET001", "DET001"])

    def test_det002_rejects_binary_heap(self) -> None:
        bad_rs = "let mut q: BinaryHeap<u32> = BinaryHeap::new();"
        violations = linter.lint_source_text(bad_rs, self.dummy_path)
        self.assertTrue(any(v.code == "DET002" for v in violations))

    def test_det003_rejects_std_float_transcendentals_and_rounding(self) -> None:
        samples = [
            "let a = angle.sin();",
            "let b = f32::cos(angle);",
            "let c = dist.sqrt();",
            "let d = val.floor();",
            "let e = val.ceil();",
            "let f = y.atan2(x);",
            "let g = base.powf(2.5);",
        ]
        for line in samples:
            violations = linter.lint_source_text(line, self.dummy_path)
            self.assertEqual(
                len(violations),
                1,
                msg=f"Expected DET003 violation on `{line}`, got {violations}",
            )
            self.assertEqual(violations[0].code, "DET003")

    def test_det004_rejects_mul_add_fma(self) -> None:
        bad_rs = "let rval = (rnd * mult).mul_add(delta, lo);"
        violations = linter.lint_source_text(bad_rs, self.dummy_path)
        self.assertEqual([v.code for v in violations], ["DET004"])

    def test_det005_rejects_pointer_keyed_btreemap(self) -> None:
        bad_rs = "let map: BTreeMap<*const Object, i32> = BTreeMap::new();"
        violations = linter.lint_source_text(bad_rs, self.dummy_path)
        self.assertEqual([v.code for v in violations], ["DET005"])

    def test_det006_rejects_usize_in_xfer(self) -> None:
        bad_rs = "xfer.xfer_user(&mut (len as usize));"
        violations = linter.lint_source_text(bad_rs, self.dummy_path)
        self.assertEqual([v.code for v in violations], ["DET006"])

    def test_exemption_comment_suppresses_rule(self) -> None:
        exempt_rs = (
            "let table: HashMap<u32, u32> = HashMap::new(); "
            "// determinism-ok: read-only CLI lookup table never iterated"
        )
        violations = linter.lint_source_text(exempt_rs, self.dummy_path)
        self.assertEqual(violations, [])


if __name__ == "__main__":
    unittest.main(verbosity=2)
