#!/usr/bin/env python3
"""Self-test for `swap_candidates.py` (no engine needed)."""

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import swap_candidates as sc  # noqa: E402

MODULE = """
use crate::sim::SimApi;
mod_marker!();
fn helper(sim: &mut dyn SimApi, owner: u32) { deeper(&mut *sim, owner); }
fn deeper(sim: &mut dyn SimApi, owner: u32) { sim.object_mut(owner); }
fn unused(sim: &mut dyn SimApi) { sim.rules(); }
pub struct Leaf;
impl Leaf {
    // sim.rules() in a comment, and "sim.rules()" in a string, are not calls.
    fn go(&mut self, sim: &mut dyn SimApi) { sim.frame(); let _ = "sim.rules()"; }
}
pub struct Deep;
impl UpdateFacet for Deep {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: u32) { self.step(sim); helper(sim, ctx); }
}
impl Deep {
    fn step(&mut self, view: &dyn SimView) { view.frame(); }
}
pub struct Inner;
impl Inner { fn go(&self, sim: &mut dyn SimApi) { sim.rules(); } }
pub struct Wrapper { pub inner: Inner }
impl Wrapper { fn run(&self, sim: &mut dyn SimApi) { self.inner.go(sim); } }
trait Shared { fn shared(&self, sim: &mut dyn SimApi) { sim.object_mut(0); } }
pub struct Inherits;
impl Shared for Inherits {}
pub struct FromMacro;
impl_die!(FromMacro);
macro_rules! impl_die {
    ($t:ty) => {
        impl DieFacet for $t {
            fn die(&mut self, sim: &mut dyn SimApi) { sim.rules(); }
        }
    };
}
#[cfg(test)]
mod tests {
    impl Leaf { fn t(sim: &mut dyn SimApi) { sim.rules(); } }
}
"""
METHODS = frozenset({"frame", "object_mut", "rules"})


class SwapCandidatesTest(unittest.TestCase):
    def setUp(self) -> None:
        self.program = sc.Program({"m.rs": MODULE}, METHODS)

    def test_follows_helpers_and_ignores_text_and_tests(self) -> None:
        self.assertEqual(self.program.reached("Leaf")[0], {"frame"})
        self.assertEqual(self.program.reached("Deep")[0], {"frame", "object_mut"})

    def test_follows_fields_and_trait_defaults(self) -> None:
        self.assertEqual(self.program.reached("Wrapper")[0], {"rules"})
        self.assertEqual(self.program.reached("Inherits")[0], {"object_mut"})

    def test_macro_generated_functions_belong_to_the_type(self) -> None:
        self.assertEqual(self.program.reached("FromMacro")[0], {"rules"})

    def test_the_real_traits_list_their_reference_returns(self) -> None:
        every, references = sc.trait_methods(sc.gen_engine_sim.TRAITS_SOURCE.read_text())
        self.assertGreater(len(every), 400)
        self.assertLessEqual({"object", "object_mut", "rules", "module_data", "client"}, references)
        self.assertNotIn("frame", references)

    def test_known_modules_in_the_real_tree(self) -> None:
        program, references = sc.load()
        # Runs in the engine today, so nothing it reaches may be a reference.
        methods, _ = program.reached("LifetimeUpdate")
        self.assertLessEqual({"set_wake_frame", "rng_logic_int", "attempt_damage"}, methods)
        self.assertEqual(methods & references, set())
        # A body does its work through the `ActiveBody` it wraps.
        self.assertGreater(len(program.reached("StructureBody")[0]), 5)

    def test_render_splits_free_from_blocked(self) -> None:
        row = {"replays": 1, "methods": [], "services": [], "dispatched": []}
        text = sc.render(
            [dict(row, name="A", instances=3, data=[]), dict(row, name="B", instances=2, data=["rules"])],
            ["C"],
        )
        self.assertIn("with no data method: 1.", text)
        self.assertIn("| `rules` | 1 | 2 | 1 |", text)
        self.assertLess(text.index("| `A` |"), text.index("## Blocked"))
        self.assertGreater(text.index("| `B` |"), text.index("## Blocked"))


if __name__ == "__main__":
    unittest.main()
