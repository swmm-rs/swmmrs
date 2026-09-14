# Run `just` to list commands. Recipes run from the repository root.
set shell := ["sh", "-eu", "-c"]
set positional-arguments

# List available commands.
default:
    @just --list

# Install locked documentation dependencies (requires Node.js 22+ and uv).
docs-deps:
    npm --prefix docs/typescript-docs ci
    uv sync --locked

# Generate the TypeScript API reference from source comments.
tsdoc:
    npm --prefix docs/typescript-docs run build

# Generate the TypeScript reference and build the full documentation site.
docs: tsdoc
    uv run --locked zensical build --clean --strict

# Build the site and run documentation tooling and rendered-reference checks.
docs-check: docs
    node --test docs/typescript-docs/test.mjs
    uv run --locked python -m unittest discover -s scripts/tests -p 'test_docs_*.py'
    uv run --locked python docs/typescript-docs/check-site.py

# Preview docs with TypeDoc and Zensical watching; extra arguments go to Zensical.
docs-serve *args: tsdoc
    uv run --locked python scripts/serve-docs.py "$@"

# Build the CLI without release optimizations: target/debug/runswmmrs.
cli-debug:
    cargo build --locked -p runswmmrs --bin runswmmrs

# Build the optimized CLI: target/release/runswmmrs.
cli-release:
    cargo build --locked --release -p runswmmrs --bin runswmmrs

# Build a debug Python wheel without installing it: python/target/wheels/debug/.
python-debug:
    uv build python --wheel --out-dir python/target/wheels/debug --config-setting 'maturin.build-args=--profile dev'

# Build an optimized Python wheel: python/target/wheels/release/.
python-release:
    uv build python --wheel --out-dir python/target/wheels/release --config-setting 'maturin.build-args=--profile release'

# Install locked JavaScript package dependencies.
js-deps:
    npm --prefix js ci

# Build debug threaded/serial WASM, JavaScript, and declarations.
js-debug:
    npm --prefix js run build:wasm -- --dev --locked
    npm --prefix js run build:ts

# Build optimized threaded/serial WASM, JavaScript, and declarations.
js-release:
    npm --prefix js run build:wasm -- --release --locked
    npm --prefix js run build:ts
    node js/tests/release-assets.mjs
