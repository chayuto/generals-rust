#!/usr/bin/env python3
"""Self-test for `scripts/build_rust_engine.py` (no vendor tree needed)."""

from __future__ import annotations

import pathlib
import re
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import build_rust_engine as bre  # noqa: E402

SOURCE = "#include <a>\nBEGIN here\nold body\nEND here\ntail\n"
FFI = bre.REPO_ROOT / "crates/zh_ffi"


class BuildRustEngineTest(unittest.TestCase):
    def test_swap_region_keeps_the_surroundings(self) -> None:
        out = bre.swap_region(SOURCE, "BEGIN", "END", "NEW")
        self.assertEqual(out, "#include <a>\nNEW\n\nEND here\ntail\n")

    def test_swap_region_rejects_missing_or_repeated_anchors(self) -> None:
        for source in ("END here\n", SOURCE + "BEGIN again\n", "END\nBEGIN\n"):
            with self.assertRaises(bre.BuildError):
                bre.swap_region(source, "BEGIN", "END", "NEW")

    def test_compile_args_replace_input_and_output(self) -> None:
        out = bre.compile_args(
            ["c++", "-O2", "-o", "old.o", "-c", "old.cpp"],
            pathlib.Path("/w/new.cpp"),
            pathlib.Path("/w/new.o"),
            ["-I/inc"],
        )
        self.assertEqual(out, ["c++", "-O2", "-I/inc", "-o", "/w/new.o", "-c", "/w/new.cpp"])

    def test_link_args_put_objects_ahead_of_libraries(self) -> None:
        command = [":", "&&", "c++", "main.o", "-o", "game", "libz.a", "x/libzh_ffi.a", "&&", ":"]
        out = bre.link_args(command, [pathlib.Path("/w/new.o")], "game-rust")
        self.assertEqual(
            out, ["c++", "main.o", "-o", "game-rust", "/w/new.o", "libz.a", "x/libzh_ffi.a"]
        )

    def test_link_args_require_the_rust_library(self) -> None:
        with self.assertRaises(bre.BuildError):
            bre.link_args(["c++", "-o", "game", "libz.a"], [], "game-rust")

    def test_every_forwarder_calls_a_declared_and_exported_symbol(self) -> None:
        header = (FFI / "include/zh_ffi.h").read_text()
        lib = (FFI / "src/lib.rs").read_text()
        for name, prefix, count in (
            ("zh_ffi_random_value.inc", "zh_random_", 12),
            ("zh_ffi_experience_tracker.inc", "zh_exp_tracker_", 7),
        ):
            inc = (FFI / "include" / name).read_text()
            called = set(re.findall(rf"\b({prefix}[a-z_]+)\(", inc))
            self.assertEqual(len(called), count, (name, sorted(called)))
            for symbol in called:
                self.assertRegex(header, rf"\b{symbol}\(")
                self.assertRegex(lib, rf"fn {symbol}\(")

    def test_each_swap_has_a_fault_variant(self) -> None:
        for swap in bre.SWAPS:
            includes = [r for _, _, r in swap.regions if r.startswith("#include")]
            self.assertEqual(len(includes), 1, swap.name)
            inc = includes[0].split('"')[1]
            self.assertIn("ZH_SEAM_MUTATE", (FFI / "include" / inc).read_text(), swap.name)


if __name__ == "__main__":
    unittest.main()
