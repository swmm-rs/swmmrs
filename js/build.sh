#!/bin/sh
set -eu
cd "$(dirname "$0")"
command -v wasm-pack >/dev/null || { echo "error: wasm-pack not found; run cargo install wasm-pack --locked" >&2; exit 1; }
# Remap embedded panic locations without exposing the dependency's source layout.
parallel_source=$(
  set -- ../crates/solver/thirdparty/swmmrs-parallel/rust/*/browser.rs
  if [ "$#" -ne 1 ] || [ ! -f "$1" ]; then
    echo "error: expected exactly one parallel browser backend" >&2
    exit 1
  fi
  cd "$(dirname "$1")"
  pwd
)
remap="--remap-path-prefix=$parallel_source=swmmrs-parallel"
# Remove obsolete snippets from previous builds before packaging.
rm -rf dist/snippets dist/serial/snippets
wasm-pack build --target web --out-dir dist --out-name swmmrs "$@" --config "target.wasm32-unknown-unknown.rustflags=[\"$remap\"]"
cp wasm.npmignore dist/.npmignore

# Keep serial build flags and Cargo artifacts separate from the shared-memory build.
(
  unset CARGO_ENCODED_RUSTFLAGS
  export RUSTFLAGS="-C target-feature=-atomics,+bulk-memory,+mutable-globals -C link-arg=--max-memory=2147483648 -C link-arg=-zstack-size=8388608 $remap"
  export CARGO_TARGET_DIR=target/wasm-serial
  wasm-pack build --target web --out-dir dist/serial --out-name swmmrs "$@"
)
cp wasm.npmignore dist/serial/.npmignore
