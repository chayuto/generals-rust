#!/usr/bin/env python3
"""Unit tests for scripts/apply_ffi_seam.py."""

import unittest
from apply_ffi_seam import SEAM_BEGIN, SEAM_END, apply_cmake_seam


class TestApplyFfiSeam(unittest.TestCase):
    def test_applies_and_is_idempotent(self) -> None:
        base = "add_library(z_gameengine STATIC)\n"
        first = apply_cmake_seam(base)
        self.assertIn(SEAM_BEGIN, first)
        self.assertIn(SEAM_END, first)
        self.assertIn("libzh_ffi.a", first)
        second = apply_cmake_seam(first)
        self.assertEqual(first, second)
        self.assertEqual(second.count(SEAM_BEGIN), 1)


if __name__ == "__main__":
    unittest.main(verbosity=2)
