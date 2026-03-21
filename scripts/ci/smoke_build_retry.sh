#!/usr/bin/env bash
# Smoke build with retry for CI resilience.
#
# Attempts `cargo build --release --locked` up to CI_SMOKE_BUILD_ATTEMPTS times
# (default: 3). Uses the release-fast profile when available, otherwise falls
# back to --release.
#
# Environment:
#   CI_SMOKE_BUILD_ATTEMPTS  Max attempts (default: 3)
#   CARGO_BUILD_JOBS         Forwarded to cargo (optional)

set -euo pipefail

MAX_ATTEMPTS="${CI_SMOKE_BUILD_ATTEMPTS:-3}"
PROFILE="release-fast"

# Detect whether the release-fast profile exists in Cargo.toml
if ! grep -q "\[profile\.${PROFILE}\]" Cargo.toml 2>/dev/null; then
  PROFILE="release"
fi

BUILD_CMD=(cargo build --profile "$PROFILE" --locked)

attempt=1
while [ "$attempt" -le "$MAX_ATTEMPTS" ]; do
  echo "::group::Build attempt ${attempt}/${MAX_ATTEMPTS}"
  if "${BUILD_CMD[@]}"; then
    echo "::endgroup::"
    echo "Build succeeded on attempt ${attempt}."
    exit 0
  fi
  echo "::endgroup::"
  echo "::warning::Build attempt ${attempt} failed."
  attempt=$((attempt + 1))
  if [ "$attempt" -le "$MAX_ATTEMPTS" ]; then
    echo "Retrying in 5 seconds..."
    sleep 5
  fi
done

echo "::error::Build failed after ${MAX_ATTEMPTS} attempts."
exit 1
