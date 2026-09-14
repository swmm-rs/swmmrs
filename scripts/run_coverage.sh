#!/bin/sh
# Run solver-owned Rust coverage in an authorized checkout.

set -eu

SCRIPT_DIR=$(CDPATH= cd "$(dirname "$0")" && pwd)
ROOT=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
SOLVER="$ROOT/crates/solver"
TARGET_DIR="$SOLVER/target"
COVERAGE_DATA="$TARGET_DIR/coverage-data"

if [ ! -f "$SOLVER/Cargo.toml" ]; then
    printf '%s\n' "run_coverage: crates/solver/Cargo.toml is absent; initialize the private solver checkout first" >&2
    exit 1
fi

command -v cargo >/dev/null 2>&1 || {
    printf '%s\n' "run_coverage: cargo is required" >&2
    exit 1
}
command -v grcov >/dev/null 2>&1 || {
    printf '%s\n' "run_coverage: grcov is required" >&2
    exit 1
}

mkdir -p "$COVERAGE_DATA"
rm -f "$COVERAGE_DATA"/*.profraw
rm -rf "$TARGET_DIR/debug/coverage"

export CARGO_TARGET_DIR="$TARGET_DIR"
export RUSTFLAGS="-Cinstrument-coverage${RUSTFLAGS:+ $RUSTFLAGS}"
export LLVM_PROFILE_FILE="$COVERAGE_DATA/swmmrs-%p-%m.profraw"

cargo test --manifest-path "$SOLVER/Cargo.toml" --locked --lib

grcov "$COVERAGE_DATA" \
  --source-dir "$ROOT" \
  --binary-path "$TARGET_DIR/debug" \
  --keep-only 'crates/solver/src/**' \
  --output-types html \
  --branch \
  --ignore-not-existing \
  --output-path "$TARGET_DIR/debug/coverage"
