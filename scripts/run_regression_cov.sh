#!/bin/sh
# Maintainer-only cross-module regression coverage; requires authorized solver sources.

set -eu

SCRIPT_DIR=$(CDPATH= cd "$(dirname "$0")" && pwd)
ROOT=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
SOLVER="$ROOT/crates/solver"
TARGET_DIR="$ROOT/crates/run/target"
COVERAGE_DATA="$TARGET_DIR/regression-coverage-data"

if [ ! -f "$SOLVER/Cargo.toml" ]; then
    printf '%s\n' "run_regression_cov: crates/solver/Cargo.toml is absent; this maintainer-only check requires the authorized solver checkout" >&2
    exit 1
fi

command -v cargo >/dev/null 2>&1 || {
    printf '%s\n' "run_regression_cov: cargo is required" >&2
    exit 1
}
command -v grcov >/dev/null 2>&1 || {
    printf '%s\n' "run_regression_cov: grcov is required" >&2
    exit 1
}

mkdir -p "$COVERAGE_DATA"
rm -f "$COVERAGE_DATA"/*.profraw
rm -rf "$TARGET_DIR/debug/coverage"

export CARGO_TARGET_DIR="$TARGET_DIR"
export RUSTFLAGS="-Cinstrument-coverage${RUSTFLAGS:+ $RUSTFLAGS}"
export LLVM_PROFILE_FILE="$COVERAGE_DATA/swmmrs-%p-%m.profraw"

cargo test --manifest-path "$ROOT/crates/run/Cargo.toml" --locked --bin runswmmrs regression_suite::

grcov "$COVERAGE_DATA" \
  --source-dir "$ROOT" \
  --binary-path "$TARGET_DIR/debug" \
  --keep-only 'crates/solver/src/**' \
  --output-types html \
  --branch \
  --ignore-not-existing \
  --output-path "$TARGET_DIR/debug/coverage"
