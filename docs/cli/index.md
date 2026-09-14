# Command-line runner

`runswmmrs` executes one SWMM input file through the complete native Rust lifecycle and writes a text report and binary output file.

!!! warning
    The solver is pre-release. Compare critical results against a trusted EPA SWMM baseline before relying on it.

!!! info "Solver source access"
    The native solver source is private. Access starts with a conversation about
    contributing; read the [governance and contribution model](../governance.md).

## Install

<!-- markdownlint-disable MD046 -->

=== "macOS and Linux"

    ```sh
    curl -LsSf https://raw.githubusercontent.com/swmm-rs/swmmrs/main/scripts/install.sh | sh
    ```

=== "Windows"

    ```powershell
    powershell -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/swmm-rs/swmmrs/main/scripts/install.ps1 | iex"
    ```

<!-- markdownlint-enable MD046 -->

See the [installation guide](install.md) for additional installation options.

## Run a model

```sh
runswmmrs model.inp model.rpt model.out
```

The three paths are required and positional:

1. EPA SWMM input file
2. text report destination
3. binary output destination

The runner opens and validates the input, starts routing with saved results, advances to completion, ends the run, writes the detailed report, and closes all files.

## Help and version

```sh
runswmmrs --help
runswmmrs --version
```

The version command reports the embedded SWMM engine version rather than the Cargo package or GitHub release version.

## Exit behavior

| Exit code | Meaning |
| --- | --- |
| `0` | Simulation completed, or help/version output completed. |
| `1` | Arguments were invalid or a solver lifecycle operation failed. |

Solver failures are written to standard error as `error: <diagnostic>`. The runner preserves the first lifecycle error when cleanup also fails.

## Output ownership

Use distinct output paths for concurrent or repeated jobs. A successful run writes the detailed `.rpt` report and retained `.out` binary artifact requested on the command line.

The CLI produces but does not query the completed `.out` file. Use the Python [`OutputReader`](../guides/read-output.md) for validated post-run extraction, or use the [Python bindings](../python/index.md) to collect Live Views, snapshots, and statistics while routing.
