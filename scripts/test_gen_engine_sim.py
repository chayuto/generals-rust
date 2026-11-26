#!/usr/bin/env python3
"""Self-test for `scripts/gen_engine_sim.py` (no vendor tree needed)."""

from __future__ import annotations

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gen_engine_sim as gen  # noqa: E402

SOURCE = """use crate::arena::{
    ObjectSlot,
};

pub trait SimView {
    /// Doc with a `fn decoy();` inside.
    fn frame(&self) -> UnsignedInt;
    fn grid(&self, cells: [Int; 4], at: &crate::math::Coord3D) -> Bool;
}

pub trait SimApi: SimView {
    fn many(
        &mut self,
        mut first: Int,
        second: std::vec::Vec<Int>, // trailing; comment
    );
}
"""


class GenEngineSimTest(unittest.TestCase):
    def test_signatures_and_argument_names(self) -> None:
        view = gen.methods(gen.trait_body(SOURCE, "SimView"))
        self.assertEqual([m[0] for m in view], ["frame", "grid"])
        self.assertEqual(view[1][2], ["cells", "at"])
        self.assertIn("&zh_logic::math::Coord3D", view[1][1])
        api = gen.methods(gen.trait_body(SOURCE, "SimApi"))
        self.assertEqual(api[0][2], ["first", "second"])

    def test_wired_methods_forward_and_others_abort(self) -> None:
        out = gen.render(SOURCE, ("frame", "many"))
        self.assertIn("self.engine_frame()", out)
        self.assertIn("self.engine_many(first, second)", out)
        self.assertIn('not_wired("grid")', out)
        self.assertIn("use zh_logic::arena::{", out)

    def test_unknown_wired_name_and_default_body_are_errors(self) -> None:
        with self.assertRaises(gen.GenError):
            gen.render(SOURCE, ("nope",))
        with self.assertRaises(gen.GenError):
            gen.methods("    fn has_default(&self) -> Int {\n        0\n    }\n")

    def test_committed_file_is_current(self) -> None:
        self.assertEqual(gen.main(["--check"]), 0)


if __name__ == "__main__":
    unittest.main()
