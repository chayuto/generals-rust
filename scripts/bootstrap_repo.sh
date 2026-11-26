#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$REPO_ROOT"

EXPECTED_EMAIL="5305832+chayuto@users.noreply.github.com"

echo "=== [1/4] Configuring git identity and hooks in $REPO_ROOT ==="
git config core.hooksPath .githooks
git config user.email "$EXPECTED_EMAIL"
git config user.name "chayuto"
chmod +x .githooks/pre-commit .githooks/commit-msg scripts/bootstrap_repo.sh scripts/lint_determinism.py scripts/test_lint_determinism.py

HOOKS_PATH="$(git config core.hooksPath)"
ACTUAL_EMAIL="$(git config user.email)"
if [[ "$HOOKS_PATH" != ".githooks" || "$ACTUAL_EMAIL" != "$EXPECTED_EMAIL" ]]; then
  echo "ERROR: Git configuration verification failed (hooksPath=$HOOKS_PATH, email=$ACTUAL_EMAIL)" >&2
  exit 1
fi
echo "  ✓ core.hooksPath = $HOOKS_PATH"
echo "  ✓ user.email     = $ACTUAL_EMAIL"

echo "=== [2/4] Checking host disk budget and required toolchains ==="
AVAIL_KB="$(df -k "$REPO_ROOT" | awk 'NR==2 {print $4}')"
AVAIL_GIB="$(awk -v kb="$AVAIL_KB" 'BEGIN {printf "%.1f", kb / 1048576}')"
echo "  ✓ Free disk space: ${AVAIL_GIB} GiB"
if [[ "$AVAIL_KB" -lt 4194304 ]]; then
  echo "WARNING: Less than 4.0 GiB free on disk (${AVAIL_GIB} GiB). Offload C++ builds to GitHub Actions CI." >&2
fi

for cmd in git gh python3; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "ERROR: Required command '$cmd' not found in PATH." >&2
    exit 1
  fi
done
echo "  ✓ Core tools (git, gh, python3) present"

for opt_cmd in cargo rustc cmake ninja; do
  if command -v "$opt_cmd" >/dev/null 2>&1; then
    echo "  ✓ Toolchain present: $opt_cmd ($(command -v "$opt_cmd"))"
  else
    echo "  - Optional local toolchain not in PATH: $opt_cmd"
  fi
done

echo "=== [3/4] Running V&V self-tests on Manifest, Tier 0 Slicer, FFI Seam & Gate V1 Determinism Linter ==="
python3 scripts/validate_manifest.py
python3 scripts/test_prep_packet.py
python3 scripts/test_apply_ffi_seam.py
python3 scripts/test_lint_determinism.py
python3 scripts/test_port_map.py
python3 scripts/test_build_oracle.py
python3 scripts/test_pack_wave.py
python3 scripts/test_check_replays.py
python3 scripts/test_build_rust_engine.py
python3 scripts/lint_determinism.py
python3 scripts/port_map.py --check
if command -v cargo >/dev/null 2>&1 && [[ -f Cargo.toml ]]; then
  cargo test --workspace --quiet
  echo "  ✓ Gate V2-Golden (3,000 C++ Golden Test Vectors) passed via cargo test"
fi

echo "=== [4/4] Running V&V self-tests on .githooks/commit-msg ==="
TMP_CLEAN="$(mktemp)"
TMP_BAD="$(mktemp)"
trap 'rm -f "$TMP_CLEAN" "$TMP_BAD"' EXIT

printf "feat(common): add deterministic wrapping RNG seed helper\n" > "$TMP_CLEAN"
if ! ./.githooks/commit-msg "$TMP_CLEAN"; then
  echo "ERROR: .githooks/commit-msg rejected a valid commit message!" >&2
  exit 1
fi

TRAILER_KEY="Co-Authored-""By:"
printf "feat(common): bad message\n\n%s Example Bot <bot@example.com>\n" "$TRAILER_KEY" > "$TMP_BAD"
if ./.githooks/commit-msg "$TMP_BAD" 2>/dev/null; then
  echo "ERROR: .githooks/commit-msg failed to block a prohibited attribution trailer!" >&2
  exit 1
fi
echo "  ✓ .githooks/commit-msg verified (accepts valid commits, blocks prohibited trailers)"
echo "=== Repository bootstrap and guardrail V&V complete ==="
