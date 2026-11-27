#!/usr/bin/env python3
"""
Fails when a tracked file in the reference clone `vendor/GeneralsX` has changed.

The clone is the oracle the port is compared with, so nothing may edit it. A worker
once overwrote `Weapon.cpp` with a helper script. Passes where there is no clone.

  python3 scripts/check_vendor.py
"""

from __future__ import annotations

import pathlib
import subprocess
import sys

VENDOR = pathlib.Path(__file__).resolve().parent.parent / "vendor/GeneralsX"
# Edited on purpose by `apply_ffi_seam.py`.
ALLOWED = frozenset({"GeneralsMD/Code/GameEngine/CMakeLists.txt"})


def changed(vendor: pathlib.Path) -> list[str]:
    """Tracked paths that differ from the checked-out commit, allowed ones left out."""
    result = subprocess.run(
        ["git", "-C", str(vendor), "diff", "--name-only", "HEAD"],
        capture_output=True,
        text=True,
        check=True,
    )
    return sorted(set(result.stdout.splitlines()) - ALLOWED)


def main(vendor: pathlib.Path = VENDOR) -> int:
    if not (vendor / ".git").exists():
        print("no reference clone here: nothing to check")
        return 0
    paths = changed(vendor)
    if paths:
        print("the reference clone has been edited:", *paths, sep="\n  ")
        print(f"restore with: git -C {vendor} checkout -- <path>")
        return 1
    print("reference clone is unchanged")
    return 0


if __name__ == "__main__":
    sys.exit(main())
