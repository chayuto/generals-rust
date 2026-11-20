#!/usr/bin/env python3
"""
Idempotently wires the `libzh_ffi.a` C-ABI staticlib and `crates/zh_ffi/include`
into `vendor/GeneralsX/GeneralsMD/Code/GameEngine/CMakeLists.txt`.
"""

from __future__ import annotations

import argparse
import pathlib
import sys

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
DEFAULT_CMAKE_PATH = (
    REPO_ROOT / "vendor/GeneralsX/GeneralsMD/Code/GameEngine/CMakeLists.txt"
)

SEAM_BEGIN = "# --- BEGIN GENERALS_RUST_FFI_SEAM ---"
SEAM_END = "# --- END GENERALS_RUST_FFI_SEAM ---"

SEAM_BLOCK = f"""{SEAM_BEGIN}
get_filename_component(ZH_RUST_ROOT "${{CMAKE_SOURCE_DIR}}/../.." ABSOLUTE)
target_include_directories(z_gameengine PUBLIC "${{ZH_RUST_ROOT}}/crates/zh_ffi/include")
if(WIN32)
    target_link_libraries(z_gameengine PUBLIC "${{ZH_RUST_ROOT}}/target/release/zh_ffi.lib" ws2_32 userenv bcrypt ntdll)
else()
    target_link_libraries(z_gameengine PUBLIC "${{ZH_RUST_ROOT}}/target/release/libzh_ffi.a")
endif()
{SEAM_END}
"""


def apply_cmake_seam(cmake_text: str) -> str:
    if SEAM_BEGIN in cmake_text and SEAM_END in cmake_text:
        start = cmake_text.index(SEAM_BEGIN)
        end = cmake_text.index(SEAM_END) + len(SEAM_END)
        updated = cmake_text[:start].rstrip() + "\n\n" + SEAM_BLOCK
        tail = cmake_text[end:].lstrip("\n")
        if tail:
            updated += "\n" + tail
        return updated
    return cmake_text.rstrip() + "\n\n" + SEAM_BLOCK


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Wire libzh_ffi.a into GeneralsX GameEngine CMakeLists.txt."
    )
    parser.add_argument(
        "--cmake-file",
        type=pathlib.Path,
        default=DEFAULT_CMAKE_PATH,
        help="Path to GeneralsMD/Code/GameEngine/CMakeLists.txt",
    )
    args = parser.parse_args(argv)

    if not args.cmake_file.is_file():
        print(f"ERROR: CMake file not found: {args.cmake_file}", file=sys.stderr)
        return 1

    original = args.cmake_file.read_text(encoding="utf-8")
    updated = apply_cmake_seam(original)
    args.cmake_file.write_text(updated, encoding="utf-8")
    print(f"Applied generals-rust FFI CMake seam to {args.cmake_file}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
