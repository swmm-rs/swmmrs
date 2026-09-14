# Contributing

Public contributions are welcome for documentation, Python and command-line
interfaces, the standalone output reader, packaging, installer behavior,
validation models, benchmark methods, and reproducible discrepancy reports.
Original contributions to the public repository are accepted under Apache-2.0.

The private solver source is excluded from that license for now. Its privacy is
meant to encourage communication and contribution before access, not to create
an exclusive club. If project interest grows enough, the solver will be
released under Apache-2.0. See the full [development and trust
model](../governance.md).

## Core solver contributions

The solver repository is shared with people who genuinely intend to contribute.
That can mean validation, investigation, review, documentation, tooling, public
interfaces, benchmarks, or solver code. A completed public contribution is
helpful but is not a formal prerequisite.

To start the conversation, email [admin@swmm.rs](mailto:admin@swmm.rs) with your
GitHub username, relevant experience, and the work you want to pursue.

Do not include private credentials or solver source in public issues, pull
requests, logs, or artifacts.

## Pull requests

Open pull requests against `main`. Repository tests use private dependencies, so
the workflow targets the protected `maintainer-approved` GitHub environment
before checking out those dependencies and running the platform matrix. Repository
administrators must configure that environment with required reviewers. Maintainers
approve the workflow after reviewing the proposed changes.

Keep changes focused and include tests or validation appropriate to the behavior
being changed. Preserve EPA SWMM behavior, floating-point evaluation order, and
deterministic results unless an intentional difference is clearly documented.

## Development commands

The root `justfile` provides the shared local and CI task interface. Install
[Just](https://just.systems/) through your package manager or with uv:

```bash
uv tool install rust-just==1.58.0
just --list
```

Recipes run from the repository root, including when invoked from a subdirectory.
They require a POSIX shell (`sh`), also used by the JavaScript WASM build script.
Native builds require the Rust toolchain and authorized solver submodules.
Python wheels additionally require uv; JavaScript builds require Node.js 22+,
`wasm-pack`, and the toolchain in `js/rust-toolchain.toml`.

| Project | Debug build | Release build | Output |
| --- | --- | --- | --- |
| CLI | `just cli-debug` | `just cli-release` | `target/debug/runswmmrs` or `target/release/runswmmrs` (`.exe` on Windows) |
| Python | `just python-debug` | `just python-release` | Wheels in `python/target/wheels/debug/` or `python/target/wheels/release/` |
| JavaScript / TypeScript | `just js-debug` | `just js-release` | Threaded WASM in `js/dist/`, serial WASM in `js/dist/serial/`, JS and declarations in `js/lib/` |

Run `just js-deps` before the first JavaScript build, or after its npm lockfile
changes. Both JS profiles build WASM first, then compile TypeScript. The latest
profile replaces the generated JS package assets at the same paths. Python
builds create wheels without installing into or changing an existing environment;
its debug recipe explicitly selects Cargo's `dev` profile, rather than the
build backend's default optimized wheel profile. These are local builds, not
cross-platform release or publishing commands.

## Documentation and installer checks

Documentation needs Just, uv, and Node.js 22+, but no solver source:

```bash
just docs-deps       # Once, or after dependency lockfiles change.
just docs-check      # Generate TypeScript docs, build the site, and run checks.
./scripts/test-install.sh
```

Use `just docs` for a strict site build without the additional checks, or
`just tsdoc` to regenerate only the TypeScript reference. CI runs the same
`just docs-deps docs-check` pipeline. Bare `zensical build` still only consumes
previously generated Markdown; use the Just recipes to keep it current.

### TypeScript documentation

The TypeScript API reference is generated from source-authored comments.
Put member descriptions, parameter details, defaults, units, return values,
errors, valid lifecycle states, and short examples in TSDoc comments under
`js/src/swmmrs/`. Signatures come from the declarations. Put workflow explanations
and substantial examples in the JavaScript user guide, not in API Markdown.
Use `{@link Symbol}` for API links and `{@inheritDoc Type.property}` when a patch
or snapshot field shares an existing description.

The `docs/typescript-docs/*.ts` entry points select public exports by family;
they must not contain duplicate declarations or API descriptions. Each API page
includes generated Markdown at its existing site URL. When adding an export,
select it in exactly one entry point; when adding a family, register its entry
point in `typedoc.json`, add a thin API include page, and update navigation.

Do not edit or commit `.generated/` output. Checks compare the selected symbols
against the package-root exports, require source contracts, validate generated
cross-page anchors, and check links throughout the rendered JavaScript docs.
Generation fails on unresolved links or TypeScript errors.

For local preview with automatic source-comment regeneration:

```bash
just docs-serve
# Optional Zensical arguments:
just docs-serve --dev-addr 127.0.0.1:8001
```

This generates the TypeScript reference before starting the preview, then runs
TypeDoc's watcher alongside Zensical. Source changes regenerate Markdown, which
triggers the site reload. Ctrl-C stops both processes; if either exits, the
other is stopped too. A missing generated include is a build error.
`npm --prefix js run test:docs` checks both guide examples and TSDoc examples
against the built public package declarations as part of the JavaScript checks.
Write each example as a standalone module, with imports and `declare` bindings
when needed.

The docs toolchain is isolated from the package's TypeScript 7 compiler because
TypeDoc currently supports TypeScript through 6. The reference uses TypeDoc's
Markdown renderer: `mkdocstrings-typescript` 0.1.0 could not collect the full
public API's indexed-access types, and its comment renderer omitted block-tag
contracts such as parameter and return descriptions. This choice can be
revisited without moving documentation out of the source.

The PowerShell installer test is available on Windows:

```powershell
.\scripts\test-install.ps1
```

## Full workspace tests

After maintainer access is granted, initialize the private dependencies and run
the complete workspace suite:

```bash
git submodule update --init --recursive
cargo test --workspace --exclude swmmrs-parallel --locked --all-features
```
