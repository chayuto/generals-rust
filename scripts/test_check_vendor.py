#!/usr/bin/env python3
"""Self-test for `check_vendor.py`, on a throwaway repository."""

import pathlib
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import check_vendor as cv  # noqa: E402


class CheckVendorTest(unittest.TestCase):
    def test_flags_an_edit_but_not_the_allowed_one(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = pathlib.Path(tmp)

            def git(*args: str) -> None:
                identity = ["-c", "user.name=t", "-c", "user.email=t@t", "-c", "core.hooksPath=/dev/null"]
                subprocess.run(["git", "-C", tmp, *identity, *args], check=True, capture_output=True)

            allowed = root / next(iter(cv.ALLOWED))
            allowed.parent.mkdir(parents=True)
            allowed.write_text("a\n")
            (root / "Weapon.cpp").write_text("body\n")
            git("init", "-q")
            git("add", ".")
            git("commit", "-qm", "pin")
            self.assertEqual(cv.main(root), 0)
            allowed.write_text("b\n")
            self.assertEqual(cv.changed(root), [])
            (root / "Weapon.cpp").write_text("\n")
            self.assertEqual(cv.changed(root), ["Weapon.cpp"])
            self.assertEqual(cv.main(root), 1)

    def test_passes_without_a_clone(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            self.assertEqual(cv.main(pathlib.Path(tmp)), 0)


if __name__ == "__main__":
    unittest.main()
